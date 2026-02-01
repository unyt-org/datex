use crate::crypto::crypto::{CryptoError, CryptoResult, CryptoTrait};
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use core::prelude::rust_2024::*;

static UUID_COUNTER: OnceLock<AtomicU64> = OnceLock::new();

fn init_counter() -> &'static AtomicU64 {
    UUID_COUNTER.get_or_init(|| AtomicU64::new(1))
}
fn generate_pseudo_uuid() -> String {
    let counter = init_counter();
    let count = counter.fetch_add(1, Ordering::Relaxed);

    // Encode counter into last segment, keeping UUID-like structure
    format!("00000000-0000-0000-0000-{count:012x}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct CryptoMock;
impl CryptoTrait for CryptoMock {
    fn create_uuid(&self) -> String {
        generate_pseudo_uuid()
    }

    fn random_bytes(&self, length: usize) -> Vec<u8> {
        unimplemented!()
    }

    fn hash_sha256<'a>(
        &'a self,
        to_digest: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn hkdf_sha256<'a>(
        &'a self,
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }
    // EdDSA keygen
    fn gen_ed25519<'a>(&'a self) -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        unimplemented!()
    }

    // EdDSA signature
    fn sig_ed25519<'a>(
        &'a self,
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, [u8; 64]> {
        unimplemented!()
    }

    // EdDSA verification of signature
    fn ver_ed25519<'a>(
        &'a self,
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, bool> {
        unimplemented!()
    }

    // AES CTR
    fn aes_ctr_encrypt<'a>(
        &'a self,
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    fn aes_ctr_decrypt<'a>(
        &'a self,
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        ciphertext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    // AES KW
    fn key_upwrap<'a>(
        &'a self,
        kek_bytes: &'a [u8; 32],
        rb: &'a [u8; 32],
    ) -> CryptoResult<'a, [u8; 40]> {
        unimplemented!()
    }

    fn key_unwrap<'a>(
        &'a self,
        kek_bytes: &'a [u8; 32],
        cipher: &'a [u8; 40],
    ) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    // Generate encryption keypair
    fn gen_x25519<'a>(&'a self) -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        unimplemented!()
    }

    // Derive shared secret on x255109
    fn derive_x25519<'a>(
        &'a self,
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }
}
