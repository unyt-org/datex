use crate::prelude::*;
use core::{
    cell::OnceCell,
    sync::atomic::{AtomicU32, Ordering},
};
use datex_crypto_facade::crypto::{Crypto, CryptoResult};

pub struct CryptoStub;

#[thread_local]
static UUID_COUNTER: OnceCell<AtomicU32> = OnceCell::new();

fn generate_pseudo_uuid() -> String {
    let counter = UUID_COUNTER.get_or_init(|| AtomicU32::new(1));
    let count = counter.fetch_add(1, Ordering::Relaxed);

    // Encode counter into last segment, keeping UUID-like structure
    format!("00000000-0000-0000-0000-{count:012x}")
}

impl Crypto for CryptoStub {
    fn create_uuid() -> String {
        generate_pseudo_uuid()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        vec![0u8; length]
    }

    fn hash_sha256<'a>(_to_digest: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn hkdf_sha256<'a>(
        _ikm: &'a [u8],
        _salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        unimplemented!()
    }

    fn sig_ed25519<'a>(
        _pri_key: &'a [u8],
        _data: &'a [u8],
    ) -> CryptoResult<'a, [u8; 64]> {
        unimplemented!()
    }

    fn ver_ed25519<'a>(
        _pub_key: &'a [u8],
        _sig: &'a [u8],
        _data: &'a [u8],
    ) -> CryptoResult<'a, bool> {
        unimplemented!()
    }

    fn aes_ctr_encrypt<'a>(
        _key: &'a [u8; 32],
        _iv: &'a [u8; 16],
        _plaintext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    fn aes_ctr_decrypt<'a>(
        _key: &'a [u8; 32],
        _iv: &'a [u8; 16],
        _cipher: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    fn key_upwrap<'a>(
        _kek_bytes: &'a [u8; 32],
        _rb: &'a [u8; 32],
    ) -> CryptoResult<'a, [u8; 40]> {
        unimplemented!()
    }

    fn key_unwrap<'a>(
        _kek_bytes: &'a [u8; 32],
        _cipher: &'a [u8; 40],
    ) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        unimplemented!()
    }

    fn derive_x25519<'a>(
        _pri_key: &'a [u8; 48],
        _peer_pub: &'a [u8; 44],
    ) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }
}
