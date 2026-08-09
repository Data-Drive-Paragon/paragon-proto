use crate::types::KeyPair;
use x25519_dalek::{PublicKey, StaticSecret};
use chacha20poly1305::{
    aead::AeadInPlace,
    ChaCha20Poly1305, Key, KeyInit, Nonce, Tag,
};
use rand_core::{CryptoRng, RngCore};

const TAG_LEN: usize = 16;


pub type FillRandomFn = extern "C" fn(buf: *mut u8, buf_len: usize) -> u8;

struct ExternalRng {
    fill: FillRandomFn,
}

impl RngCore for ExternalRng {
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf);
        u32::from_le_bytes(buf)
    }

    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let ok = (self.fill)(dest.as_mut_ptr(), dest.len());
        if ok != 0 {
            panic!("external RNG callback failed");
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        let ok = (self.fill)(dest.as_mut_ptr(), dest.len());
        if ok == 0 {
            Ok(())
        } else {
            Err(rand_core::Error::from(
                core::num::NonZeroU32::new(ok as u32).unwrap_or(core::num::NonZeroU32::new(1).unwrap())
            ))
        }
    }
}


impl CryptoRng for ExternalRng {}

#[no_mangle]
pub extern "C" fn crypto_generate_keypair(fill_random: FillRandomFn) -> KeyPair {
    let mut rng = ExternalRng { fill: fill_random };
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = PublicKey::from(&secret);

    KeyPair {
        public: public.to_bytes(),
        private: secret.to_bytes(),
    }
}

#[no_mangle]
pub extern "C" fn crypto_derive_shared_secret(
    private_key: [u8; 32],
    public_key: [u8; 32],
) -> [u8; 32] {
    let secret = StaticSecret::from(private_key);
    let public = PublicKey::from(public_key);
    secret.diffie_hellman(&public).to_bytes()
}


#[no_mangle]
pub extern "C" fn crypto_encrypt_chacha20poly1305(
    key: [u8; 32],
    nonce: [u8; 12],
    plaintext: *const u8,
    plaintext_len: usize,
    ciphertext: *mut u8,
    ciphertext_buf_len: usize,
    tag_out: *mut u8, // ровно TAG_LEN (16) байт
) -> u8 {
    if plaintext.is_null() || ciphertext.is_null() || tag_out.is_null() {
        return 1;
    }
    if ciphertext_buf_len < plaintext_len {
        return 2; // буфер вывода слишком мал
    }

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce);

    let plain = unsafe { core::slice::from_raw_parts(plaintext, plaintext_len) };
    let out = unsafe { core::slice::from_raw_parts_mut(ciphertext, plaintext_len) };
    out.copy_from_slice(plain);

    match cipher.encrypt_in_place_detached(nonce, /* aad = */ &[], out) {
        Ok(tag) => {
            let tag_dst = unsafe { core::slice::from_raw_parts_mut(tag_out, TAG_LEN) };
            tag_dst.copy_from_slice(tag.as_slice());
            0
        }
        Err(_) => 3,
    }
}


#[no_mangle]
pub extern "C" fn crypto_decrypt_chacha20poly1305(
    key: [u8; 32],
    nonce: [u8; 12],
    ciphertext: *const u8,
    ciphertext_len: usize,
    tag: *const u8, 
    plaintext: *mut u8,
    plaintext_buf_len: usize,
) -> u8 {
    if ciphertext.is_null() || tag.is_null() || plaintext.is_null() {
        return 1;
    }
    if plaintext_buf_len < ciphertext_len {
        return 2;
    }

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce);
    let tag_slice = unsafe { core::slice::from_raw_parts(tag, TAG_LEN) };
    let tag = Tag::from_slice(tag_slice);

    let cin = unsafe { core::slice::from_raw_parts(ciphertext, ciphertext_len) };
    let out = unsafe { core::slice::from_raw_parts_mut(plaintext, ciphertext_len) };
    out.copy_from_slice(cin);

    match cipher.decrypt_in_place_detached(nonce, /* aad = */ &[], out, tag) {
        Ok(()) => 0,
        Err(_) => 4, 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn dummy_fill_random(buf: *mut u8, buf_len: usize) -> u8 {
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, buf_len) };
        for (i, b) in slice.iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(42);
        }
        0
    }

    extern "C" fn dummy_fill_random_2(buf: *mut u8, buf_len: usize) -> u8 {
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, buf_len) };
        for (i, b) in slice.iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(99);
        }
        0
    }

    #[test]
    fn test_keypair_and_shared_secret() {
        let kp1 = crypto_generate_keypair(dummy_fill_random);
        let kp2 = crypto_generate_keypair(dummy_fill_random_2);

        let shared1 = crypto_derive_shared_secret(kp1.private, kp2.public);
        let shared2 = crypto_derive_shared_secret(kp2.private, kp1.public);
        assert_eq!(shared1, shared2);
    }

    #[test]
    fn test_chacha20poly1305_encryption() {
        let key = [0x42u8; 32];
        let nonce = [0x11u8; 12];
        let plaintext = b"Hello, Paragon Proto!";
        let mut ciphertext = [0u8; 64];
        let mut tag = [0u8; 16];

        let res = crypto_encrypt_chacha20poly1305(
            key,
            nonce,
            plaintext.as_ptr(),
            plaintext.len(),
            ciphertext.as_mut_ptr(),
            ciphertext.len(),
            tag.as_mut_ptr(),
        );
        assert_eq!(res, 0);

        let mut decrypted = [0u8; 64];
        let dec_res = crypto_decrypt_chacha20poly1305(
            key,
            nonce,
            ciphertext.as_ptr(),
            plaintext.len(),
            tag.as_ptr(),
            decrypted.as_mut_ptr(),
            decrypted.len(),
        );
        assert_eq!(dec_res, 0);
        assert_eq!(&decrypted[..plaintext.len()], plaintext);
    }
}