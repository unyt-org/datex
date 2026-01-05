use datex_crypto_facade::crypto::{Crypto, CryptoResult};
use crate::prelude::*;

pub struct CryptoStub;

impl Crypto for CryptoStub {
    fn create_uuid() -> String {
        "00000000-0000-0000-0000-000000000000".to_string()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        vec![0u8; length]
    }

    fn hash_sha256<'a>(to_digest: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn hkdf_sha256<'a>(ikm: &'a [u8], salt: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        unimplemented!()
    }

    fn sig_ed25519<'a>(pri_key: &'a [u8], data: &'a [u8]) -> CryptoResult<'a, [u8; 64]> {
        unimplemented!()
    }

    fn ver_ed25519<'a>(pub_key: &'a [u8], sig: &'a [u8], data: &'a [u8]) -> CryptoResult<'a, bool> {
        unimplemented!()
    }

    fn aes_ctr_encrypt<'a>(key: &'a [u8; 32], iv: &'a [u8; 16], plaintext: &'a [u8]) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    fn aes_ctr_decrypt<'a>(key: &'a [u8; 32], iv: &'a [u8; 16], cipher: &'a [u8]) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }

    fn key_upwrap<'a>(kek_bytes: &'a [u8; 32], rb: &'a [u8; 32]) -> CryptoResult<'a, [u8; 40]> {
        unimplemented!()
    }

    fn key_unwrap<'a>(kek_bytes: &'a [u8; 32], cipher: &'a [u8; 40]) -> CryptoResult<'a, [u8; 32]> {
        unimplemented!()
    }

    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        unimplemented!()
    }

    fn derive_x25519<'a>(pri_key: &'a [u8; 48], peer_pub: &'a [u8; 44]) -> CryptoResult<'a, Vec<u8>> {
        unimplemented!()
    }
}