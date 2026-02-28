use alloc::{boxed::Box, string::String, vec::Vec};
use bs58;
use core::{pin::Pin, result::Result};

use crate::error::*;

pub type AsyncCryptoResult<'a, T, E> =
    Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;

pub trait Crypto: Send + Sync {
    /// Generate a new UUID (version 4). Returns the UUID as a string.
    fn create_uuid() -> String;

    // Random bytes
    type RandomBytesError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::RandomBytesError;

    /// Generate `length` random bytes.
    fn random_bytes(length: usize) -> Result<Vec<u8>, Self::RandomBytesError>;

    // Hash
    type Sha256Error: core::fmt::Debug + Send + Sync + 'static =
        crate::error::Sha256Error;

    /// Compute the SHA-256 hash of the input data. Returns the 32-byte hash.
    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error>;

    /// Decode the given Base58 string into a 32-byte array.
    /// Returns an error if the input is not valid Base58 or does not decode to exactly 32 bytes.
    fn dec_b58_32(data: &str) -> Result<[u8; 32], B58DecodeError> {
        let bytes = bs58::decode(data)
            .into_vec()
            .map_err(|_| B58DecodeError::InvalidBase58)?;

        if bytes.len() != 32 {
            return Err(B58DecodeError::WrongLength {
                expected: 32,
                got: bytes.len(),
            });
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }

    // HKDF
    type HkdfError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::HkdfError;

    /// Derive a 32-byte key from the input keying material and salt using HKDF-SHA256.
    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError>;

    // Ed25519
    type Ed25519GenError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::Ed25519GenError;

    /// Generate a new Ed25519 key pair. Returns the public and private keys.
    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>), Self::Ed25519GenError>;

    type Ed25519SignError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::Ed25519SignError;

    /// Sign the given data using the provided Ed25519 private key. Returns the signature.
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError>;

    // Verify should not “error” for signature mismatch, only for malformed inputs
    type Ed25519VerifyError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::Ed25519VerifyError;

    /// Verify the given Ed25519 signature against the provided public key and data.
    /// Returns true if the signature is valid, false if it is invalid.
    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError>;

    // AES-CTR (generally only invalid input / backend unavailable)
    type AesCtrError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::AesCtrError;

    /// Encrypt the plaintext using AES-256 in CTR mode with the given key and IV. Returns the ciphertext.
    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError>;

    /// Decrypt the ciphertext using AES-256 in CTR mode with the given key and IV. Returns the plaintext.
    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError>;

    // RFC3394 wrap/unwrap: unwrap has a unique failure mode (integrity fail)
    type KeyWrapError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::KeyWrapError;

    /// Wrap the given 32-byte key using the provided 32-byte KEK with the RFC3394 algorithm. Returns the wrapped key.
    fn key_wrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        key_to_wrap: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError>;

    type KeyUnwrapError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::KeyUnwrapError;

    /// Unwrap the given 40-byte wrapped key using the provided 32-byte KEK with the RFC3394 algorithm.
    /// Returns the unwrapped key.
    fn key_unwrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        wrapped: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError>;

    // X25519
    type X25519GenError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::X25519GenError;

    /// Generate a new X25519 key pair. Returns the public key as a base58 string and the private key as bytes.
    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48]), Self::X25519GenError>;

    type X25519DeriveError: core::fmt::Debug + Send + Sync + 'static =
        crate::error::X25519DeriveError;

    /// Derive a shared secret using the X25519 key agreement protocol with the given private key and peer's public key.
    /// Returns the derived 32-byte shared secret.
    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError>;

    // Base58
    /// Encode the given bytes into a Base58 string.
    fn enc_b58(data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }

    /// Decode the given Base58 string into bytes. Returns an error if the input is not valid Base58.
    fn dec_b58(data: &str) -> Result<Vec<u8>, B58DecodeError> {
        bs58::decode(data)
            .into_vec()
            .map_err(|_| B58DecodeError::InvalidBase58)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dummy implementation of Crypto for testing the default impls.
    struct T;

    impl Crypto for T {
        fn create_uuid() -> String {
            // not used
            String::new()
        }

        fn random_bytes(
            length: usize,
        ) -> Result<Vec<u8>, Self::RandomBytesError> {
            unimplemented!()
        }

        fn hash_sha256<'a>(
            _to_digest: &'a [u8],
        ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
            unimplemented!()
        }

        fn hkdf_sha256<'a>(
            _ikm: &'a [u8],
            _salt: &'a [u8],
        ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
            unimplemented!()
        }

        fn gen_ed25519<'a>()
        -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>), Self::Ed25519GenError>
        {
            unimplemented!()
        }

        fn sig_ed25519<'a>(
            _pri_key: &'a [u8],
            _data: &'a [u8],
        ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
            unimplemented!()
        }

        fn ver_ed25519<'a>(
            _pub_key: &'a [u8],
            _sig: &'a [u8],
            _data: &'a [u8],
        ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
            unimplemented!()
        }

        fn aes_ctr_encrypt<'a>(
            _key: &'a [u8; 32],
            _iv: &'a [u8; 16],
            _plaintext: &'a [u8],
        ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
            unimplemented!()
        }

        fn aes_ctr_decrypt<'a>(
            _key: &'a [u8; 32],
            _iv: &'a [u8; 16],
            _cipher: &'a [u8],
        ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
            unimplemented!()
        }

        fn key_wrap_rfc3394<'a>(
            _kek: &'a [u8; 32],
            _key_to_wrap: &'a [u8; 32],
        ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
            unimplemented!()
        }

        fn key_unwrap_rfc3394<'a>(
            _kek: &'a [u8; 32],
            _wrapped: &'a [u8; 40],
        ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
            unimplemented!()
        }

        fn gen_x25519<'a>()
        -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48]), Self::X25519GenError>
        {
            unimplemented!()
        }

        fn derive_x25519<'a>(
            _pri_key: &'a [u8; 48],
            _peer_pub: &'a [u8; 44],
        ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
            unimplemented!()
        }
    }

    #[test]
    fn b58_roundtrip_arbitrary_bytes() {
        let input = b"hello DATEX";
        let encoded = T::enc_b58(input);
        let decoded = T::dec_b58(&encoded).unwrap();
        assert_eq!(input, &decoded[..]);
    }

    #[test]
    fn b58_roundtrip_empty() {
        let input: &[u8] = b"";
        let encoded = T::enc_b58(input);
        let decoded = T::dec_b58(&encoded).unwrap();
        assert_eq!(input, &decoded[..]);
    }

    #[test]
    fn b58_roundtrip_32_bytes() {
        let mut input = [0u8; 32];
        for (i, b) in input.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(3);
        }

        let encoded = T::enc_b58(&input);
        let decoded = T::dec_b58_32(&encoded).unwrap();
        assert_eq!(input, decoded);
    }

    #[test]
    fn b58_dec_invalid_base58_errors() {
        // Contains characters not in base58 alphabet
        let bad = "0OIl";
        let err = T::dec_b58(bad).unwrap_err();
        assert_eq!(err, B58DecodeError::InvalidBase58);
    }

    #[test]
    fn b58_dec32_wrong_length_errors() {
        // "2g" decodes to a single byte [0x61] ("a"), not 32 bytes.
        let s = "2g";
        let err = T::dec_b58_32(s).unwrap_err();
        assert_eq!(
            err,
            B58DecodeError::WrongLength {
                expected: 32,
                got: 1
            }
        );
    }
}
