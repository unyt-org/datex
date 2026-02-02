#![no_std]
#![feature(thread_local)]

extern crate alloc;

use core::future::Future;
use alloc::format;
use esp_hal::rng::Rng;
use core::result::Result;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use core::pin::Pin;
use datex_crypto_facade::crypto::{Crypto, CryptoResult};
use datex_crypto_facade::error::CryptoError;

#[thread_local]
static mut RNG: Rng = Rng::new();

pub fn get_global_rng() -> &'static mut Rng {
    unsafe {
        &mut RNG
    }
}

#[derive(Debug, Clone)]
pub struct EspCrypto;

impl Crypto for EspCrypto {
    fn create_uuid() -> String {
        // TODO: use uuid crate?
        let mut bytes = [0u8; 16];
        get_global_rng().read(&mut bytes);

        // set version to 4 -- random
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        // set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                u16::from_be_bytes([bytes[4], bytes[5]]),
                u16::from_be_bytes([bytes[6], bytes[7]]),
                u16::from_be_bytes([bytes[8], bytes[9]]),
                u64::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], 0, 0]) >> 16
        )
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        get_global_rng().read(&mut bytes);
        bytes
    }

    fn hash_sha256<'a>(to_digest: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        todo!()
    }

    fn hkdf_sha256<'a>(ikm: &'a [u8], salt: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        todo!()
    }

    fn gen_ed25519() -> Pin<Box<dyn Future<Output=Result<(Vec<u8>, Vec<u8>), CryptoError>> + 'static>> {
        todo!()
    }

    fn sig_ed25519<'a>(pri_key: &'a [u8], data: &'a [u8]) -> Pin<Box<dyn Future<Output=Result<[u8; 64], CryptoError>> + 'a>> {
        todo!()
    }

    fn ver_ed25519<'a>(pub_key: &'a [u8], sig: &'a [u8], data: &'a [u8]) -> Pin<Box<dyn Future<Output=Result<bool, CryptoError>> + 'a>> {
        todo!()
    }

    fn aes_ctr_encrypt<'a>(key: &'a [u8; 32], iv: &'a [u8; 16], plaintext: &'a [u8]) -> Pin<Box<dyn Future<Output=Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }

    fn aes_ctr_decrypt<'a>(key: &'a [u8; 32], iv: &'a [u8; 16], cipher: &'a [u8]) -> Pin<Box<dyn Future<Output=Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }

    fn key_upwrap<'a>(kek_bytes: &'a [u8; 32], rb: &'a [u8; 32]) -> Pin<Box<dyn Future<Output=Result<[u8; 40], CryptoError>> + 'a>> {
        todo!()
    }

    fn key_unwrap<'a>(kek_bytes: &'a [u8; 32], cipher: &'a [u8; 40]) -> Pin<Box<dyn Future<Output=Result<[u8; 32], CryptoError>> + 'a>> {
        todo!()
    }

    fn gen_x25519() -> Pin<Box<dyn Future<Output=Result<([u8; 44], [u8; 48]), CryptoError>>>> {
        todo!()
    }

    fn derive_x25519<'a>(pri_key: &'a [u8; 48], peer_pub: &'a [u8; 44]) -> Pin<Box<dyn Future<Output=Result<Vec<u8>, CryptoError>> + 'a>> {
        todo!()
    }
}