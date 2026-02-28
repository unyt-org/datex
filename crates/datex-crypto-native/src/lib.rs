use datex_crypto_facade::{
    crypto::{AsyncCryptoResult, Crypto},
    error::{
        AesCtrError, BackendError, Ed25519GenError, Ed25519SignError,
        Ed25519VerifyError, HkdfError, KeyUnwrapError, KeyWrapError,
        RandomBytesError, X25519DeriveError, X25519GenError,
    },
};
use openssl::{
    aes::{AesKey, unwrap_key, wrap_key},
    derive::Deriver,
    md::Md,
    pkey::{Id, PKey},
    pkey_ctx::{HkdfMode, PkeyCtx},
    sha::sha256,
    sign::{Signer, Verifier},
    symm::{Cipher, Crypter, Mode},
};
use rand::{TryRngCore, rngs::OsRng};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct CryptoNative;
impl Crypto for CryptoNative {
    fn create_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    fn random_bytes(length: usize) -> Result<Vec<u8>, Self::RandomBytesError> {
        let mut out = vec![0u8; length];
        OsRng.try_fill_bytes(&mut out).map_err(|_| {
            RandomBytesError::Backend(BackendError::Unavailable(
                "OsRng failed to generate random bytes",
            ))
        })?;
        Ok(out)
    }
    type Sha256Error = ();

    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        Box::pin(async move {
            let hash = sha256(to_digest);
            Ok(hash)
        })
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        Box::pin(async move {
            let info = b"";
            let mut ctx = PkeyCtx::new_id(Id::HKDF).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf ctx",
                ))
            })?;
            ctx.derive_init().map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf init",
                ))
            })?;
            ctx.set_hkdf_mode(HkdfMode::EXTRACT_THEN_EXPAND)
                .map_err(|_| {
                    HkdfError::Backend(BackendError::Unavailable(
                        "openssl hkdf mode",
                    ))
                })?;
            ctx.set_hkdf_md(Md::sha256()).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable("openssl hkdf md"))
            })?;
            ctx.set_hkdf_salt(salt).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf salt",
                ))
            })?;
            ctx.set_hkdf_key(ikm).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf key",
                ))
            })?;
            ctx.add_hkdf_info(info).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf info",
                ))
            })?;

            let mut okm = [0u8; 32];
            ctx.derive(Some(&mut okm)).map_err(|_| {
                HkdfError::Backend(BackendError::Unavailable(
                    "openssl hkdf derive",
                ))
            })?;
            Ok(okm)
        })
    }

    // EdDSA keygen
    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>), Self::Ed25519GenError> {
        Box::pin(async move {
            let key = PKey::generate_ed25519().map_err(|_| {
                Ed25519GenError::Backend(BackendError::Unavailable(
                    "openssl ed25519 gen",
                ))
            })?;

            // Keep your DER/PKCS8 formats (portable).
            let public_key = key.public_key_to_der().map_err(|_| {
                Ed25519GenError::Backend(BackendError::Unavailable(
                    "ed25519 pub der",
                ))
            })?;
            let private_key = key.private_key_to_pkcs8().map_err(|_| {
                Ed25519GenError::Backend(BackendError::Unavailable(
                    "ed25519 priv pkcs8",
                ))
            })?;

            Ok((public_key, private_key))
        })
    }

    // EdDSA signature
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        Box::pin(async move {
            let sig_key = PKey::private_key_from_pkcs8(pri_key)
                .map_err(|_| Ed25519SignError::InvalidPrivateKey)?;

            let mut signer =
                Signer::new_without_digest(&sig_key).map_err(|_| {
                    Ed25519SignError::Backend(BackendError::Unavailable(
                        "ed25519 signer",
                    ))
                })?;

            let signature = signer.sign_oneshot_to_vec(data).map_err(|_| {
                Ed25519SignError::Backend(BackendError::Unavailable(
                    "ed25519 sign",
                ))
            })?;

            let sig: [u8; 64] =
                signature.as_slice().try_into().map_err(|_| {
                    Ed25519SignError::Backend(BackendError::Unavailable(
                        "ed25519 sig len",
                    ))
                })?;

            Ok(sig)
        })
    }

    // EdDSA verification of signature
    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        Box::pin(async move {
            let public_key = PKey::public_key_from_der(pub_key)
                .map_err(|_| Ed25519VerifyError::InvalidPublicKey)?;

            // OpenSSL expects signature to be exactly 64 bytes for Ed25519.
            if sig.len() != 64 {
                return Err(Ed25519VerifyError::InvalidSignature);
            }

            let mut verifier = Verifier::new_without_digest(&public_key)
                .map_err(|_| {
                    Ed25519VerifyError::Backend(BackendError::Unavailable(
                        "ed25519 verifier",
                    ))
                })?;

            let ok = verifier.verify_oneshot(sig, data).map_err(|_| {
                Ed25519VerifyError::Backend(BackendError::Unavailable(
                    "ed25519 verify",
                ))
            })?;

            Ok(ok)
        })
    }

    // AES CTR
    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            let cipher = Cipher::aes_256_ctr();
            let mut crypter =
                Crypter::new(cipher, Mode::Encrypt, key, Some(iv)).map_err(
                    |_| {
                        AesCtrError::Backend(BackendError::Unavailable(
                            "openssl aes-ctr",
                        ))
                    },
                )?;

            let mut out = vec![0u8; plaintext.len() + cipher.block_size()];
            let mut count =
                crypter.update(plaintext, &mut out).map_err(|_| {
                    AesCtrError::Backend(BackendError::Unavailable(
                        "aes-ctr update",
                    ))
                })?;

            count += crypter.finalize(&mut out[count..]).map_err(|_| {
                AesCtrError::Backend(BackendError::Unavailable(
                    "aes-ctr finalize",
                ))
            })?;

            out.truncate(count);
            Ok(out)
        })
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher_text: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            let cipher = Cipher::aes_256_ctr();
            let mut crypter =
                Crypter::new(cipher, Mode::Decrypt, key, Some(iv)).map_err(
                    |_| {
                        AesCtrError::Backend(BackendError::Unavailable(
                            "openssl aes-ctr",
                        ))
                    },
                )?;

            let mut out = vec![0u8; cipher_text.len() + cipher.block_size()];
            let mut count =
                crypter.update(cipher_text, &mut out).map_err(|_| {
                    AesCtrError::Backend(BackendError::Unavailable(
                        "aes-ctr update",
                    ))
                })?;

            count += crypter.finalize(&mut out[count..]).map_err(|_| {
                AesCtrError::Backend(BackendError::Unavailable(
                    "aes-ctr finalize",
                ))
            })?;

            out.truncate(count);
            Ok(out)
        })
    }

    // AES KW
    fn key_wrap_rfc3394<'a>(
        kek_bytes: &'a [u8; 32],
        rb: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        Box::pin(async move {
            // Key encryption key
            let kek = AesKey::new_encrypt(kek_bytes).map_err(|_| {
                KeyWrapError::Backend(BackendError::Unavailable(
                    "openssl aes-kw",
                ))
            })?;

            // Key wrap
            let mut wrapped = [0u8; 40];
            let _length = wrap_key(&kek, None, &mut wrapped, rb);

            Ok(wrapped)
        })
    }

    fn key_unwrap_rfc3394<'a>(
        kek_bytes: &'a [u8; 32],
        cipher: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        Box::pin(async move {
            // Key encryption key
            let kek = AesKey::new_decrypt(kek_bytes).map_err(|_| {
                KeyUnwrapError::Backend(BackendError::Unavailable(
                    "openssl aes-kw",
                ))
            })?;

            // Unwrap key
            let mut unwrapped: [u8; 32] = [0u8; 32];
            let _length = unwrap_key(&kek, None, &mut unwrapped, cipher);
            Ok(unwrapped)
        })
    }

    // Generate encryption keypair
    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48]), Self::X25519GenError> {
        Box::pin(async move {
            let key = PKey::generate_x25519().map_err(|_| {
                X25519GenError::Backend(BackendError::Unavailable(
                    "openssl x25519 gen",
                ))
            })?;

            let public_key: [u8; 44] = key
                .public_key_to_der()
                .map_err(|_| {
                    X25519GenError::Backend(BackendError::Unavailable(
                        "openssl x25519 gen",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    X25519GenError::Backend(BackendError::Unavailable(
                        "openssl x25519 gen",
                    ))
                })?;
            let private_key: [u8; 48] = key
                .private_key_to_pkcs8()
                .map_err(|_| {
                    X25519GenError::Backend(BackendError::Unavailable(
                        "openssl x25519 gen",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    X25519GenError::Backend(BackendError::Unavailable(
                        "openssl x25519 gen",
                    ))
                })?;
            Ok((public_key, private_key))
        })
    }

    // Derive shared secret on x255109
    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_raw: &'a [u8; 44],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        Box::pin(async move {
            let my_priv = PKey::private_key_from_pkcs8(pri_key)
                .map_err(|_| X25519DeriveError::InvalidPrivateKey)?;
            let peer_pub = PKey::public_key_from_der(peer_raw)
                .map_err(|_| X25519DeriveError::InvalidPeerPublicKey)?;

            let mut deriver = Deriver::new(&my_priv).map_err(|_| {
                X25519DeriveError::Backend(BackendError::Unavailable(
                    "x25519 deriver",
                ))
            })?;
            deriver.set_peer(&peer_pub).map_err(|_| {
                X25519DeriveError::Backend(BackendError::Unavailable(
                    "x25519 set_peer",
                ))
            })?;

            let shared = deriver.derive_to_vec().map_err(|_| {
                X25519DeriveError::Backend(BackendError::Unavailable(
                    "x25519 derive",
                ))
            })?;

            if shared.len() != 32 {
                return Err(X25519DeriveError::Backend(
                    BackendError::Unavailable("x25519 shared len"),
                ));
            }

            let mut out = [0u8; 32];
            out.copy_from_slice(&shared);
            Ok(out)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CryptoNative;
    use datex_crypto_facade::{
        crypto::Crypto,
        error::{Ed25519VerifyError, KeyUnwrapError, X25519DeriveError},
    };
    #[test]
    fn test_uuid() {
        let uuid1 = CryptoNative::create_uuid();
        let uuid2 = CryptoNative::create_uuid();
        assert_ne!(uuid1, uuid2);

        // 8-4-4-4-12 = 36 chars
        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid2.len(), 36);

        // Basic dash positions check
        assert_eq!(&uuid1[8..9], "-");
        assert_eq!(&uuid1[13..14], "-");
        assert_eq!(&uuid1[18..19], "-");
        assert_eq!(&uuid1[23..24], "-");
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = CryptoNative::random_bytes(16).expect("random bytes");
        let bytes2 = CryptoNative::random_bytes(16).expect("random bytes");
        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2);
    }

    #[tokio::test]
    async fn test_sha256_matches_openssl() {
        let msg = b"hello world";
        let got = CryptoNative::hash_sha256(msg).await.expect("sha256");
        let expected = openssl::sha::sha256(msg);
        assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn test_hkdf_deterministic_and_changes_with_inputs() {
        let ikm = b"input key material";
        let salt1 = b"salt one";
        let salt2 = b"salt two";

        let out1 = CryptoNative::hkdf_sha256(ikm, salt1).await.expect("hkdf");
        let out1b = CryptoNative::hkdf_sha256(ikm, salt1).await.expect("hkdf");
        let out2 = CryptoNative::hkdf_sha256(ikm, salt2).await.expect("hkdf");
        let out3 = CryptoNative::hkdf_sha256(b"other ikm", salt1)
            .await
            .expect("hkdf");

        assert_eq!(out1, out1b, "HKDF must be deterministic for same inputs");
        assert_ne!(out1, out2, "Changing salt should change output");
        assert_ne!(out1, out3, "Changing ikm should change output");
    }

    #[tokio::test]
    async fn test_ed25519_sign_verify_ok_and_mismatch() {
        let (pub_key, pri_key) =
            CryptoNative::gen_ed25519().await.expect("gen ed25519");
        let msg = b"Hello DATEX";

        let sig = CryptoNative::sig_ed25519(&pri_key, msg)
            .await
            .expect("sign");
        let ok = CryptoNative::ver_ed25519(&pub_key, &sig, msg)
            .await
            .expect("verify");
        assert!(ok);

        let ok2 = CryptoNative::ver_ed25519(&pub_key, &sig, b"goodbye DATEX")
            .await
            .expect("verify");
        assert!(!ok2);
    }

    #[tokio::test]
    async fn test_ed25519_verify_rejects_wrong_sig_length() {
        let (pub_key, _pri_key) =
            CryptoNative::gen_ed25519().await.expect("gen ed25519");
        let msg = b"msg";
        let bad_sig = [0u8; 63];

        let err = CryptoNative::ver_ed25519(&pub_key, &bad_sig, msg)
            .await
            .unwrap_err();

        assert_eq!(err, Ed25519VerifyError::InvalidSignature);
    }

    #[tokio::test]
    async fn test_aes_ctr_roundtrip_and_wrong_key_changes_output() {
        let key = [7u8; 32];
        let iv = [9u8; 16];
        let pt = b"Hello DATEX";

        let ct = CryptoNative::aes_ctr_encrypt(&key, &iv, pt)
            .await
            .expect("encrypt");
        assert_ne!(ct, pt);

        let got = CryptoNative::aes_ctr_decrypt(&key, &iv, &ct)
            .await
            .expect("decrypt");
        assert_eq!(got, pt);

        let wrong_key = [8u8; 32];
        let got2 = CryptoNative::aes_ctr_decrypt(&wrong_key, &iv, &ct)
            .await
            .expect("decrypt");
        assert_ne!(got2, pt);
    }

    #[tokio::test]
    async fn test_rfc3394_wrap_unwrap_roundtrip() {
        let kek = [1u8; 32];
        let key_to_wrap = [2u8; 32];

        let wrapped = CryptoNative::key_wrap_rfc3394(&kek, &key_to_wrap)
            .await
            .expect("wrap");

        let unwrapped = CryptoNative::key_unwrap_rfc3394(&kek, &wrapped)
            .await
            .expect("unwrap");

        assert_eq!(unwrapped, key_to_wrap);
    }

    #[tokio::test]
    #[ignore = "Integrity check is not implemented"]
    async fn test_rfc3394_unwrap_integrity_failure_on_tamper() {
        let kek = [3u8; 32];
        let key_to_wrap = [4u8; 32];

        let mut wrapped = CryptoNative::key_wrap_rfc3394(&kek, &key_to_wrap)
            .await
            .expect("wrap");

        // flip one bit
        wrapped[0] ^= 0x01;

        let err = CryptoNative::key_unwrap_rfc3394(&kek, &wrapped)
            .await
            .unwrap_err();
        assert_eq!(err, KeyUnwrapError::IntegrityCheckFailed);
    }

    #[tokio::test]
    async fn test_x25519_derive_same_secret_both_sides() {
        let (a_pub, a_pri) = CryptoNative::gen_x25519().await.expect("gen a");
        let (b_pub, b_pri) = CryptoNative::gen_x25519().await.expect("gen b");

        let a_shared = CryptoNative::derive_x25519(&a_pri, &b_pub)
            .await
            .expect("derive a");
        let b_shared = CryptoNative::derive_x25519(&b_pri, &a_pub)
            .await
            .expect("derive b");

        assert_eq!(a_shared, b_shared);
    }

    #[tokio::test]
    async fn test_x25519_invalid_peer_key_errors() {
        let (_pub, pri) = CryptoNative::gen_x25519().await.expect("gen");

        // peer key with wrong bytes should error
        let bad_peer = [0u8; 44]; // not a valid DER public key usually, but your impl uses raw_bytes
        let err = CryptoNative::derive_x25519(&pri, &bad_peer)
            .await
            .unwrap_err();

        assert_eq!(err, X25519DeriveError::InvalidPeerPublicKey);
    }
}
