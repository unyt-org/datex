use crate::prelude::*;
use bs58;
use core::{fmt::Display, pin::Pin, result::Result};
pub type CryptoResult<'a, T> =
Pin<Box<dyn Future<Output = Result<T, CryptoError>> + 'a>>;

pub trait Crypto: Send + Sync {
    /// Creates a new UUID.
    fn create_uuid() -> String;

    /// Generates cryptographically secure random bytes of the specified length.
    fn random_bytes(length: usize) -> Vec<u8>;

    /// Sha256 hash
    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]>;

    /// Encodes 32 bytes to base58
    fn enc_b58<'a>(
        to_encode: &'a [u8; 32],
    ) -> Result<[u8; 44], CryptoError> {
        let mut out_buf = [0u8; 44];
        bs58::encode(to_encode)
            .onto(&mut out_buf[..])
            .map_err(|_| CryptoError::Decryption)?;
        Ok(out_buf)
    }

    /// Decodes 32 bytes from base58
    fn dec_b58<'a>(
        to_decode: &'a [u8; 44],
    ) -> Result<[u8; 32], CryptoError> {
        let mut out_buf = [0u8; 32];
        bs58::decode(to_decode)
            .onto(&mut out_buf[..])
            .map_err(|_| CryptoError::Decryption)?;
        Ok(out_buf)
    }

    /// Hash key derivation function.
    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]>;

    /// Generates an Ed25519 key pair.
    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)>;

    /// Signs data with the given Ed25519 private key.
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, [u8; 64]>;

    /// Verifies an Ed25519 signature with the given public key and data.
    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, bool>;

    /// AES-256 in CTR mode encryption, returns the ciphertext.
    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>>;

    /// AES-256 in CTR mode decryption, returns the plaintext.
    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>>;

    /// AES Key Wrap (RFC 3394), returns the wrapped key (ciphertext).
    fn key_upwrap<'a>(
        kek_bytes: &'a [u8; 32],
        rb: &'a [u8; 32],
    ) -> CryptoResult<'a, [u8; 40]>;

    /// AES Key Unwrap (RFC 3394), returns the unwrapped key (plaintext).
    fn key_unwrap<'a>(
        kek_bytes: &'a [u8; 32],
        cipher: &'a [u8; 40],
    ) -> CryptoResult<'a, [u8; 32]>;

    /// Generates an X25519 key pair, returns (public_key, private_key).
    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])>;

    /// Derives a shared secret using X25519 given my private key and the peer's public key.
    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> CryptoResult<'a, Vec<u8>>;
}

// pub struct Crypto;

#[derive(Debug, Clone)]
pub enum CryptoError {
    Other(String),
    KeyGeneration,
    KeyExport,
    KeyImport,
    Encryption,
    Decryption,
    Signing,
    Verification,
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::Other(msg) => core::write!(f, "Crypto: {}", msg),
            CryptoError::KeyGeneration => {
                core::write!(f, "CryptoError: Key generation failed")
            }
            CryptoError::KeyExport => {
                core::write!(f, "CryptoError: Key export failed")
            }
            CryptoError::KeyImport => {
                core::write!(f, "CryptoError: Key import failed")
            }
            CryptoError::Encryption => {
                core::write!(f, "CryptoError: Encryption failed")
            }
            CryptoError::Decryption => {
                core::write!(f, "CryptoError: Decryption failed")
            }
            CryptoError::Signing => {
                core::write!(f, "CryptoError: Signing failed")
            }
            CryptoError::Verification => {
                core::write!(f, "CryptoError: Verification failed")
            }
        }
    }
}
