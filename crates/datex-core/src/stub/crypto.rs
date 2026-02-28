use crate::prelude::*;
use core::{
    cell::OnceCell,
    sync::atomic::{AtomicU32, Ordering},
};
use datex_crypto_facade::crypto::{AsyncCryptoResult, Crypto};

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

    fn random_bytes(length: usize) -> Result<Vec<u8>, Self::RandomBytesError> {
        Ok(vec![0u8; length])
    }

    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        unimplemented!()
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        unimplemented!()
    }

    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>), Self::Ed25519GenError> {
        unimplemented!()
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        unimplemented!()
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        unimplemented!()
    }

    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        unimplemented!()
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        unimplemented!()
    }

    fn key_wrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        key_to_wrap: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        unimplemented!()
    }

    fn key_unwrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        wrapped: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        unimplemented!()
    }

    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48]), Self::X25519GenError> {
        unimplemented!()
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        unimplemented!()
    }
}
