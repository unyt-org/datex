use crate::prelude::*;
use core::sync::atomic::{AtomicU32, Ordering};
use datex_facades::crypto::{AsyncCryptoResult, Crypto, CryptoSync};

pub struct CryptoStub;

static UUID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn generate_pseudo_uuid() -> String {
    let count = UUID_COUNTER.fetch_add(1, Ordering::Relaxed);

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

    fn hash_sha256<'a>(
        _to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        todo!()
    }

    fn hkdf_sha256<'a>(
        _ikm: &'a [u8],
        _salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        todo!()
    }

    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::Ed25519GenError> {
        todo!()
    }

    fn sig_ed25519<'a>(
        _pri_key: &'a [u8],
        _data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        todo!()
    }

    fn ver_ed25519<'a>(
        _pub_key: &'a [u8],
        _sig: &'a [u8],
        _data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        todo!()
    }

    fn aes_ctr_encrypt<'a>(
        _key: &'a [u8; 32],
        _iv: &'a [u8; 16],
        _plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        todo!()
    }

    fn aes_ctr_decrypt<'a>(
        _key: &'a [u8; 32],
        _iv: &'a [u8; 16],
        _cipher: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        todo!()
    }

    fn key_wrap_rfc3394<'a>(
        _kek: &'a [u8; 32],
        _key_to_wrap: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        todo!()
    }

    fn key_unwrap_rfc3394<'a>(
        _kek: &'a [u8; 32],
        _wrapped: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        todo!()
    }

    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::X25519GenError> {
        todo!()
    }

    fn derive_x25519<'a>(
        _pri_key: &'a [u8; 32],
        _peer_pub: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        todo!()
    }
}

impl CryptoSync for CryptoStub {
    fn gen_ed25519_cheat()
    -> Result<([u8; 32], [u8; 32]), datex_facades::error::Ed25519GenError> {
        todo!()
    }

    fn gen_x25519_cheat()
    -> Result<([u8; 32], [u8; 32]), datex_facades::error::X25519GenError> {
        todo!()
    }

    fn derive_x25519_cheat(
        _pri_key: &[u8; 32],
        _peer_raw: &[u8; 32],
    ) -> Result<[u8; 32], datex_facades::error::X25519DeriveError> {
        todo!()
    }

    fn hkdf_cheat(
        _ikm: &[u8],
        _salt: &[u8],
    ) -> Result<[u8; 32], datex_facades::error::HkdfError> {
        todo!()
    }

    fn aes_cheat(
        _key: &[u8; 32],
        _iv: &[u8; 16],
        _data: &[u8],
    ) -> Result<Vec<u8>, datex_facades::error::AesCtrError> {
        todo!()
    }

    fn aes_kw_wrap_cheat(
        _kek_bytes: &[u8; 32],
        _rb: &[u8; 32],
    ) -> Result<[u8; 40], datex_facades::error::KeyWrapError> {
        todo!()
    }
}
