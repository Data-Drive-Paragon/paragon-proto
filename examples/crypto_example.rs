use paragon_proto::{
    crypto_generate_keypair, crypto_derive_shared_secret,
    crypto_encrypt_chacha20poly1305, crypto_decrypt_chacha20poly1305
};

extern "C" fn sample_fill_random(buf: *mut u8, buf_len: usize) -> u8 {
    let slice = unsafe { core::slice::from_raw_parts_mut(buf, buf_len) };
    for (i, b) in slice.iter_mut().enumerate() {
        *b = ((i * 17 + 5) % 256) as u8;
    }
    0
}

fn main() {
    println!("=== Paragon Proto Crypto Example ===");

    // 1. Generate Keypairs
    let keypair_alice = crypto_generate_keypair(sample_fill_random);
    let keypair_bob = crypto_generate_keypair(sample_fill_random);

    println!("Alice public key: {:02X?}", &keypair_alice.public[..8]);
    println!("Bob public key: {:02X?}", &keypair_bob.public[..8]);

    // 2. Derive Shared Secrets via Diffie-Hellman
    let shared_alice = crypto_derive_shared_secret(keypair_alice.private, keypair_bob.public);
    let shared_bob = crypto_derive_shared_secret(keypair_bob.private, keypair_alice.public);

    assert_eq!(shared_alice, shared_bob);
    println!("Shared secret successfully derived and matched!");

    // 3. Encrypt & Decrypt using ChaCha20Poly1305
    let key = shared_alice;
    let nonce = [0x75u8; 12];
    let plaintext = b"Confidential payload via Paragon Proto crypto module!";
    
    let mut ciphertext = [0u8; 128];
    let mut tag = [0u8; 16];

    let enc_res = crypto_encrypt_chacha20poly1305(
        key,
        nonce,
        plaintext.as_ptr(),
        plaintext.len(),
        ciphertext.as_mut_ptr(),
        ciphertext.len(),
        tag.as_mut_ptr(),
    );

    if enc_res == 0 {
        println!("Encryption successful!");
        println!("Ciphertext length: {}", plaintext.len());
    } else {
        println!("Encryption failed with error code: {}", enc_res);
        return;
    }

    let mut decrypted = [0u8; 128];
    let dec_res = crypto_decrypt_chacha20poly1305(
        key,
        nonce,
        ciphertext.as_ptr(),
        plaintext.len(),
        tag.as_ptr(),
        decrypted.as_mut_ptr(),
        decrypted.len(),
    );

    if dec_res == 0 {
        println!("Decryption successful!");
        let decrypted_str = core::str::from_utf8(&decrypted[..plaintext.len()]).unwrap();
        println!("Decrypted plaintext: '{}'", decrypted_str);
    } else {
        println!("Decryption failed with error code: {}", dec_res);
    }
}
