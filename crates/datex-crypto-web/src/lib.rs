use datex_crypto_facade::crypto::{AsyncCryptoResult, Crypto};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AesCtrParams, CryptoKey, CryptoKeyPair, js_sys,
    js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array},
};
mod utils;
use utils::{TryAsByteSlice, js_array, js_object};

mod sealed {
    use super::*;
    pub trait CryptoKeyType: JsCast {}
    impl CryptoKeyType for CryptoKey {}
    impl CryptoKeyType for CryptoKeyPair {}
}

use x25519_dalek::{PublicKey, StaticSecret};
use ed25519_dalek::{
    SigningKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
    },
};
use pkcs8::{AlgorithmIdentifierRef, ObjectIdentifier, PrivateKeyInfo};
use der::{Decode, Encode, asn1::BitStringRef};
use spki::SubjectPublicKeyInfoRef;

fn jsvalue_to_jserror(e: JsValue) -> JsError {
    if let Ok(err) = e.clone().dyn_into::<js_sys::Error>() {
        let msg = err
            .message()
            .as_string()
            .unwrap_or_else(|| "JavaScript error".to_string());
        JsError::new(&msg.to_string())
    } else if let Some(s) = e.as_string() {
        JsError::new(&s.to_string())
    } else {
        JsError::new(&format!("{:?}", e))
    }
}

pub struct CryptoWeb;
impl CryptoWeb {
    fn window() -> web_sys::Window {
        js_sys::global().unchecked_into::<web_sys::Window>()
    }
    fn crypto() -> web_sys::Crypto {
        Self::window().crypto().expect("no global crypto exists.")
    }
    fn crypto_subtle() -> web_sys::SubtleCrypto {
        Self::crypto().subtle()
    }

    /// Exports a `CryptoKey` to a byte vector in the specified format.
    async fn export_crypto_key(
        key: &CryptoKey,
        format: &str,
    ) -> Result<Vec<u8>, JsError> {
        let export_key_promise = Self::crypto_subtle()
            .export_key(format, key)
            .map_err(jsvalue_to_jserror)?;
        let key: JsValue = JsFuture::from(export_key_promise)
            .await
            .map_err(jsvalue_to_jserror)?;
        let bytes = key.try_as_u8_slice()?;
        Ok(bytes)
    }

    /// Imports a `CryptoKey` from a byte slice in the specified format, algorithm, and key usages.
    async fn import_crypto_key(
        key: &[u8],
        format: &str,
        algorithm: &Object,
        key_usages: &[&str],
    ) -> Result<CryptoKey, JsError> {
        let key = Uint8Array::from(key);
        let import_key_promise = Self::crypto_subtle()
            .import_key_with_object(
                format,
                &Object::from(key),
                algorithm,
                true,
                &js_array(key_usages),
            )
            .map_err(jsvalue_to_jserror)?;
        let key: JsValue = JsFuture::from(import_key_promise)
            .await
            .map_err(jsvalue_to_jserror)?;
        let key: CryptoKey = key.dyn_into().map_err(jsvalue_to_jserror)?;
        Ok(key)
    }

    // This method can either create a crypto key pair or a symmetric key
    async fn generate_crypto_key<T>(
        algorithm: &Object,
        extractable: bool,
        key_usages: &[&str],
    ) -> Result<T, JsError>
    where
        T: sealed::CryptoKeyType + From<JsValue>,
    {
        let key_generator_promise = Self::crypto_subtle()
            .generate_key_with_object(
                algorithm,
                extractable,
                &js_array(key_usages),
            )
            .map_err(jsvalue_to_jserror)?;
        let result: JsValue = JsFuture::from(key_generator_promise)
            .await
            .map_err(jsvalue_to_jserror)?;
        Ok(result.into())
    }
}

impl Crypto for CryptoWeb {
    type Sha256Error = JsError;
    type HkdfError = JsError;
    type Ed25519GenError = JsError;
    type Ed25519SignError = JsError;
    type Ed25519VerifyError = JsError;
    type AesCtrError = JsError;
    type KeyWrapError = JsError;
    type KeyUnwrapError = JsError;
    type X25519GenError = JsError;
    type X25519DeriveError = JsError;

    fn create_uuid() -> String {
        Self::crypto().random_uuid()
    }
    fn random_bytes(length: usize) -> Vec<u8> {
        let buffer = &mut vec![0u8; length];
        Self::crypto()
            .get_random_values_with_u8_array(buffer)
            .expect("getRandomValues failed");
        buffer.to_vec()
    }

    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        Box::pin(async move {
            let subtle = CryptoWeb::crypto_subtle();

            let prom = subtle
                .digest_with_object_and_u8_array(
                    &js_object(vec![("name", "SHA-256")]),
                    to_digest,
                )
                .map_err(jsvalue_to_jserror)?;

            let bits =
                JsFuture::from(prom).await.map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&bits).to_vec();
            let out: [u8; 32] = v
                .try_into()
                .map_err(|_| JsError::new("SHA-256: output length != 32"))?;
            Ok(out)
        })
    }

    // hkdf
    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        Box::pin(async move {
            let subtle = CryptoWeb::crypto_subtle();

            let usages = Array::of1(&JsValue::from_str("deriveBits"));
            let ikm_buf = Uint8Array::from(ikm).buffer();

            let key_js = JsFuture::from(
                subtle
                    .import_key_with_object(
                        "raw",
                        &ikm_buf.into(),
                        &js_object(vec![("name", "HKDF")]),
                        false,
                        &usages,
                    )
                    .map_err(jsvalue_to_jserror)?,
            )
            .await
            .map_err(jsvalue_to_jserror)?;

            let base_key: CryptoKey =
                key_js.dyn_into().map_err(jsvalue_to_jserror)?;

            let params = Object::new();
            Reflect::set(&params, &"name".into(), &"HKDF".into())
                .map_err(jsvalue_to_jserror)?;
            Reflect::set(&params, &"hash".into(), &"SHA-256".into())
                .map_err(jsvalue_to_jserror)?;
            Reflect::set(&params, &"salt".into(), &Uint8Array::from(salt))
                .map_err(jsvalue_to_jserror)?;
            Reflect::set(&params, &"info".into(), &Uint8Array::from(&[][..]))
                .map_err(jsvalue_to_jserror)?;

            let bits = JsFuture::from(
                subtle
                    .derive_bits_with_object(&params, &base_key, 256u32)
                    .map_err(jsvalue_to_jserror)?,
            )
            .await
            .map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&bits).to_vec();
            let out: [u8; 32] = v
                .try_into()
                .map_err(|_| JsError::new("HKDF: output length != 32"))?;
            Ok(out)
        })
    }

    // Signature and Verification
    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::Ed25519GenError> {
        Box::pin(async move {
            let algorithm =
                js_object(vec![("name", JsValue::from_str("Ed25519"))]);

            let key_pair: CryptoKeyPair = Self::generate_crypto_key(
                &algorithm,
                true,
                &["sign", "verify"],
            )
            .await?;

            let pub_key =
                Self::export_crypto_key(&key_pair.get_public_key(), "spki")
                    .await?;
            let raw_pub_key: [u8; 32] = pub_key[12..].try_into().unwrap();

            let pri_key =
                Self::export_crypto_key(&key_pair.get_private_key(), "pkcs8")
                    .await?;
            let raw_pri_key: [u8; 32] = pri_key[16..].try_into().unwrap();

            Ok((raw_pub_key, raw_pri_key))
        })
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        Box::pin(async move {
            let x = SigningKey::from_bytes(
                pri_key
                            .try_into()
                            .unwrap()
                );

            let oid = ObjectIdentifier::new("1.3.101.112").unwrap();
            let prepped_key =
                [[4u8, 32u8].to_vec(), x.to_bytes().to_vec()].concat();
            let pri_x = PrivateKeyInfo {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                private_key: &prepped_key,
                public_key: None,
            }
            .to_der()
            .unwrap();

            let key = Self::import_crypto_key(
                &pri_x,
                "pkcs8",
                &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                &["sign"],
            )
            .await
            .map_err(|_| JsError::new("Ed25519 import pkcs8 (sign)"))?;

            let prom = Self::crypto_subtle()
                .sign_with_object_and_u8_array(
                    &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                    &key,
                    data,
                )
                .map_err(jsvalue_to_jserror)?;

            let ab: ArrayBuffer = JsFuture::from(prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&ab).to_vec();
            let sig: [u8; 64] = v.try_into().map_err(|_| {
                JsError::new("Ed25519 sign: signature length != 64")
            })?;
            Ok(sig)
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        Box::pin(async move {
            if sig.len() != 64 {
                return Err(JsError::new(
                    "Ed25519 verify: signature must be 64 bytes",
                ));
            }

            let key = Self::import_crypto_key(
                pub_key,
                "raw",
                &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                &["verify"],
            )
            .await?;

            let prom = Self::crypto_subtle()
                .verify_with_object_and_u8_array_and_u8_array(
                    &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                    &key,
                    sig,
                    data,
                )
                .map_err(jsvalue_to_jserror)?;

            let v = JsFuture::from(prom).await.map_err(jsvalue_to_jserror)?;

            Ok(v.as_bool().unwrap_or(false))
        })
    }

    // aes ctr
    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            let usages = Array::of1(&JsValue::from_str("encrypt"));
            let key_buf = Uint8Array::from(key.as_slice()).buffer();

            let key_js = JsFuture::from(
                subtle
                    .import_key_with_object(
                        "raw",
                        &key_buf.into(),
                        &js_object(vec![("name", "AES-CTR")]),
                        false,
                        &usages,
                    )
                    .map_err(jsvalue_to_jserror)?,
            )
            .await
            .map_err(jsvalue_to_jserror)?;

            let base_key: CryptoKey =
                key_js.dyn_into().map_err(jsvalue_to_jserror)?;

            let params = AesCtrParams::new(
                "AES-CTR",
                &Uint8Array::from(iv.as_slice()),
                64u8,
            );

            let pt = Uint8Array::from(plaintext);
            let prom = subtle
                .encrypt_with_object_and_buffer_source(
                    &params.into(),
                    &base_key,
                    &pt,
                )
                .map_err(jsvalue_to_jserror)?;

            let ct = JsFuture::from(prom).await.map_err(jsvalue_to_jserror)?;

            let ct_buf: ArrayBuffer =
                ct.dyn_into().map_err(jsvalue_to_jserror)?;

            Ok(Uint8Array::new(&ct_buf).to_vec())
        })
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        ciphertext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            let usages = Array::of1(&JsValue::from_str("decrypt"));
            let key_buf = Uint8Array::from(key.as_slice()).buffer();

            let key_js = JsFuture::from(
                subtle
                    .import_key_with_object(
                        "raw",
                        &key_buf.into(),
                        &js_object(vec![("name", "AES-CTR")]),
                        false,
                        &usages,
                    )
                    .map_err(jsvalue_to_jserror)?,
            )
            .await
            .map_err(jsvalue_to_jserror)?;

            let base_key: CryptoKey =
                key_js.dyn_into().map_err(jsvalue_to_jserror)?;

            let params = AesCtrParams::new(
                "AES-CTR",
                &Uint8Array::from(iv.as_slice()),
                64u8,
            );

            let ct = Uint8Array::from(ciphertext);
            let prom = subtle
                .decrypt_with_object_and_buffer_source(
                    &params.into(),
                    &base_key,
                    &ct,
                )
                .map_err(jsvalue_to_jserror)?;

            let pt = JsFuture::from(prom).await.map_err(jsvalue_to_jserror)?;

            let pt_buf: ArrayBuffer =
                pt.dyn_into().map_err(jsvalue_to_jserror)?;

            Ok(Uint8Array::new(&pt_buf).to_vec())
        })
    }

    fn key_wrap_rfc3394<'a>(
        kek_bytes: &'a [u8; 32],
        key_to_wrap_bytes: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            let kek_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-KW"))]);
            let kek_prom = subtle
                .import_key_with_object(
                    "raw",
                    &Uint8Array::from(kek_bytes.as_slice()).buffer(),
                    &kek_algorithm,
                    false,
                    &Array::of2(&"wrapKey".into(), &"unwrapKey".into()),
                )
                .map_err(jsvalue_to_jserror)?;

            let kek: CryptoKey = JsFuture::from(kek_prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let key_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-CTR"))]);
            let key_prom = subtle
                .import_key_with_object(
                    "raw",
                    &Uint8Array::from(key_to_wrap_bytes.as_slice()).buffer(),
                    &key_algorithm,
                    true,
                    &Array::of2(&"encrypt".into(), &"decrypt".into()),
                )
                .map_err(jsvalue_to_jserror)?;

            let key_to_wrap: CryptoKey = JsFuture::from(key_prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let wrap_prom = subtle
                .wrap_key_with_str("raw", &key_to_wrap, &kek, "AES-KW")
                .map_err(jsvalue_to_jserror)?;

            let wrapped = JsFuture::from(wrap_prom)
                .await
                .map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&wrapped).to_vec();
            let out: [u8; 40] = v.try_into().map_err(|_| {
                JsError::new("AES-KW wrap: output length != 40")
            })?;
            Ok(out)
        })
    }

    fn key_unwrap_rfc3394<'a>(
        kek_bytes: &'a [u8; 32],
        wrapped_key: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            let kek_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-KW"))]);
            let kek_prom = subtle
                .import_key_with_object(
                    "raw",
                    &Uint8Array::from(kek_bytes.as_slice()).buffer(),
                    &kek_algorithm,
                    false,
                    &Array::of2(&"wrapKey".into(), &"unwrapKey".into()),
                )
                .map_err(jsvalue_to_jserror)?;

            let kek: CryptoKey = JsFuture::from(kek_prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let unwrapped_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-CTR"))]);
            let wrapped_key_array = Uint8Array::from(wrapped_key.as_slice());

            let unwrap_prom = subtle
                .unwrap_key_with_js_u8_array_and_str_and_object(
                    "raw",
                    &wrapped_key_array,
                    &kek,
                    "AES-KW",
                    &unwrapped_algorithm,
                    true,
                    &Array::of2(&"encrypt".into(), &"decrypt".into()),
                )
                .map_err(jsvalue_to_jserror)?;

            let unwrapped_key: CryptoKey = JsFuture::from(unwrap_prom)
                .await
                // IMPORTANT: if integrity fails, promise rejects -> we return the actual JS error here
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let export_prom = subtle
                .export_key("raw", &unwrapped_key)
                .map_err(jsvalue_to_jserror)?;

            let exported = JsFuture::from(export_prom)
                .await
                .map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&exported).to_vec();
            let out: [u8; 32] = v.try_into().map_err(|_| {
                JsError::new("AES-KW unwrap: output length != 32")
            })?;
            Ok(out)
        })
    }

    // x25519 key gen
    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::X25519GenError> {
        Box::pin(async move {
            let algorithm =
                js_object(vec![("name", JsValue::from_str("X25519"))]);

            let key_pair: CryptoKeyPair = Self::generate_crypto_key(
                &algorithm,
                true,
                &["deriveKey", "deriveBits"],
            )
            .await
            .map_err(|_| JsError::new("X25519 generateKey"))?;

            let pub_vec =
                Self::export_crypto_key(&key_pair.get_public_key(), "spki")
                    .await?;
            let pri_vec =
                Self::export_crypto_key(&key_pair.get_private_key(), "pkcs8")
                    .await?;

            let pub_key: [u8; 44] = pub_vec
                .try_into()
                .map_err(|_| JsError::new("X25519 spki length != 44"))?;

            let raw_pub_key: [u8; 32] = pub_key[12..].try_into().unwrap();
            let pri_key: [u8; 48] = pri_vec
                .try_into()
                .map_err(|_| JsError::new("X25519 pkcs8 length != 48"))?;
            let raw_pri_key: [u8; 32] = pri_key[16..].try_into().unwrap();

            Ok((raw_pub_key, raw_pri_key))
        })
    }

    fn derive_x25519<'a>(
        my_raw: &'a [u8; 32],
        peer_pub: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        Box::pin(async move {
            // Format private key
            let pri_key = StaticSecret::from(*my_raw);
            // WIP fill key with random bytes
            let pub_key = PublicKey::from(*peer_pub).to_bytes();
            let oid = ObjectIdentifier::new("1.3.101.110").unwrap();

            let prepped_key =
                [[4u8, 34u8].to_vec(), pri_key.to_bytes().to_vec()].concat();
            let pri_x = PrivateKeyInfo {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                private_key: &prepped_key,
                public_key: None,
            }
            .to_der()
            .unwrap();

            let pub_spki = SubjectPublicKeyInfoRef {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                subject_public_key: BitStringRef::new(0, &pub_key).unwrap(),
            }
            .to_der()
            .unwrap();


            let subtle = Self::crypto_subtle();
            let alg = js_object(vec![("name", JsValue::from_str("X25519"))]);

            let pri_prom = subtle
                .import_key_with_object(
                    "pkcs8",
                    &Uint8Array::from(pri_x.as_slice()).buffer(),
                    &alg,
                    false,
                    &Array::of2(&"deriveKey".into(), &"deriveBits".into()),
                )
                .map_err(jsvalue_to_jserror)?;

            let pri_key: CryptoKey = JsFuture::from(pri_prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let pub_prom = subtle
                .import_key_with_object(
                    "spki",
                    &Uint8Array::from(pub_spki.as_slice()).buffer(),
                    &alg,
                    false,
                    &Array::new(),
                )
                .map_err(jsvalue_to_jserror)?;

            let pub_key: CryptoKey = JsFuture::from(pub_prom)
                .await
                .map_err(jsvalue_to_jserror)?
                .dyn_into()
                .map_err(jsvalue_to_jserror)?;

            let derive_algorithm = js_object(vec![
                ("name", JsValue::from_str("X25519")),
                ("public", pub_key.into()),
            ]);

            let bits_prom = subtle
                .derive_bits_with_object(&derive_algorithm, &pri_key, 256u32)
                .map_err(jsvalue_to_jserror)?;

            let derived = JsFuture::from(bits_prom)
                .await
                .map_err(jsvalue_to_jserror)?;

            let v = Uint8Array::new(&derived).to_vec();
            let out: [u8; 32] = v
                .try_into()
                .map_err(|_| JsError::new("X25519 derived length != 32"))?;
            Ok(out)
        })
    }
}
