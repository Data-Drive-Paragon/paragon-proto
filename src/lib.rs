#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), not(test)))]
use core::panic::PanicInfo;

#[cfg(all(not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub use heapless;
pub use x25519_dalek as x25519;
pub use chacha20poly1305;
pub use aes_gcm;
pub use rand_core;

pub mod types;
pub mod validation;
pub mod crypto;
pub mod serialization;

pub use types::*;
pub use validation::*;
pub use crypto::*;
pub use serialization::*;
