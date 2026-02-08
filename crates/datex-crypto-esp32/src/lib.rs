#![no_std]
#![feature(thread_local)]

#[cfg(test)]
extern crate std;

extern crate alloc;

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    ptr::NonNull,
    result::Result,
    sync::atomic::{AtomicPtr, Ordering},
};
use datex_crypto_facade::{
    crypto::{Crypto, CryptoResult},
    error::CryptoError,
};
use spin::Mutex;
use static_cell::StaticCell;

use esp_hal::rng::Rng;

static CELL: StaticCell<Mutex<Rng>> = StaticCell::new();
static PTR: AtomicPtr<Mutex<Rng>> = AtomicPtr::new(core::ptr::null_mut());

pub fn init_rng(rng: Rng) {
    let m: &'static mut Mutex<Rng> = CELL.init(Mutex::new(rng));
    PTR.store(m as *mut _, Ordering::Release);
}

pub fn rng() -> spin::MutexGuard<'static, Rng> {
    let p = PTR.load(Ordering::Acquire);
    let m = NonNull::new(p).expect("RNG not initialized");
    unsafe { m.as_ref() }.lock()
}

#[derive(Debug, Clone)]
pub struct CryptoEsp32;

impl Crypto for CryptoEsp32 {
    fn create_uuid() -> String {
        // TODO: use uuid crate?
        let mut bytes = [0u8; 16];
        rng().read(&mut bytes);

        // set version to 4 -- random
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        // set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]),
            u16::from_be_bytes([bytes[8], bytes[9]]),
            u64::from_be_bytes([
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15], 0, 0
            ]) >> 16
        )
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        rng().read(&mut bytes);
        bytes
    }

    fn hash_sha256<'a>(to_digest: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        todo!()
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        todo!()
    }
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 64], CryptoError>> + 'a>> {
        todo!()
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<bool, CryptoError>> + 'a>> {
        todo!()
    }

    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }

    fn key_upwrap<'a>(
        kek_bytes: &'a [u8; 32],
        rb: &'a [u8; 32],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 40], CryptoError>> + 'a>> {
        todo!()
    }

    fn key_unwrap<'a>(
        kek_bytes: &'a [u8; 32],
        cipher: &'a [u8; 40],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 32], CryptoError>> + 'a>> {
        todo!()
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }

    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        todo!()
    }

    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        todo!()
    }
}
