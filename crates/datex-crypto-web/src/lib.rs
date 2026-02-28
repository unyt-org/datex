use datex_crypto_facade::{
    crypto::{Crypto, AsyncCryptoResult},
    error::CryptoError,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AesCtrParams, CryptoKey, CryptoKeyPair, js_sys,
    js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array},
};
mod utils;
use utils::{AsByteSlice, TryAsByteSlice, js_array, js_object};

mod sealed {
    use super::*;
    pub trait CryptoKeyType: JsCast {}
    impl CryptoKeyType for CryptoKey {}
    impl CryptoKeyType for CryptoKeyPair {}
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

    async fn export_crypto_key(
        key: &CryptoKey,
        format: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        let export_key_promise = Self::crypto_subtle()
            .export_key(format, key)
            .map_err(|_| CryptoError::KeyExport)?;
        let key: JsValue = JsFuture::from(export_key_promise)
            .await
            .map_err(|_| CryptoError::KeyExport)?;
        let bytes =
            key.try_as_u8_slice().map_err(|_| CryptoError::KeyExport)?;
        Ok(bytes)
    }

    async fn import_crypto_key(
        key: &[u8],
        format: &str,
        algorithm: &Object,
        key_usages: &[&str],
    ) -> Result<CryptoKey, CryptoError> {
        let key = Uint8Array::from(key);
        let import_key_promise = Self::crypto_subtle()
            .import_key_with_object(
                format,
                &Object::from(key),
                algorithm,
                true,
                &js_array(key_usages),
            )
            .map_err(|_| CryptoError::KeyImport)?;
        let key: JsValue = JsFuture::from(import_key_promise)
            .await
            .map_err(|_| CryptoError::KeyImport)?;
        let key: CryptoKey =
            key.dyn_into().map_err(|_| CryptoError::KeyImport)?;
        Ok(key)
    }

    // This method can either create a crypto key pair or a symmetric key
    async fn generate_crypto_key<T>(
        algorithm: &Object,
        extractable: bool,
        key_usages: &[&str],
    ) -> Result<T, CryptoError>
    where
        T: sealed::CryptoKeyType + From<JsValue>,
    {
        let key_generator_promise = Self::crypto_subtle()
            .generate_key_with_object(
                algorithm,
                extractable,
                &js_array(key_usages),
            )
            .map_err(|e| CryptoError::Other(format!("{e:?}")))?;
        let result: JsValue = JsFuture::from(key_generator_promise)
            .await
            .map_err(|_| CryptoError::KeyGeneration)?;

        let key_or_pair: T =
            result.try_into().map_err(|_: std::convert::Infallible| {
                CryptoError::KeyGeneration
            })?;
        Ok(key_or_pair)
    }
}

impl Crypto for CryptoWeb {
    fn create_uuid() -> String {
        Self::crypto().random_uuid()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let buffer = &mut vec![0u8; length];
        Self::crypto()
            .get_random_values_with_u8_array(buffer)
            .unwrap();
        buffer.to_vec()
    }

    fn hash_sha256<'a>(ikm: &'a [u8]) -> AsyncCryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let subtle = CryptoWeb::crypto_subtle();

            let bits = JsFuture::from(
                subtle
                    .digest_with_object_and_u8_array(
                        &js_object(vec![("name", "SHA-256")]),
                        ikm,
                    )
                    .map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?;

            let okm = Uint8Array::new(&bits).to_vec().try_into().unwrap();
            Ok(okm)
        })
    }
    // hkdf
    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let info = b"".to_vec();
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
                    .map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?;
            let base_key: CryptoKey =
                key_js.dyn_into().map_err(|_| CryptoError::KeyImport)?;

            let params = Object::new();
            Reflect::set(&params, &"name".into(), &"HKDF".into())
                .map_err(|_| CryptoError::KeyImport)?;
            Reflect::set(&params, &"hash".into(), &"SHA-256".into())
                .map_err(|_| CryptoError::KeyImport)?;
            Reflect::set(&params, &"salt".into(), &Uint8Array::from(salt))
                .map_err(|_| CryptoError::KeyImport)?;
            Reflect::set(
                &params,
                &"info".into(),
                &Uint8Array::from(info.as_slice()),
            )
            .map_err(|_| CryptoError::KeyImport)?;

            let bit_len: u32 = 32_u32 * 8;
            let bits = JsFuture::from(
                subtle
                    .derive_bits_with_object(&params, &base_key, bit_len)
                    .map_err(|_| CryptoError::KeyGeneration)?,
            )
            .await
            .map_err(|_| CryptoError::KeyGeneration)?;

            let okm: [u8; 32] =
                Uint8Array::new(&bits).to_vec().try_into().unwrap();
            if okm.len() != 32 {
                return Err(CryptoError::KeyExport);
            }
            Ok(okm)
        })
    }

    // Signature and Verification
    fn gen_ed25519<'a>() -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        Box::pin(async move {
            let algorithm =
                js_object(vec![("name", JsValue::from_str("Ed25519"))]);
            let key_pair: CryptoKeyPair = Self::generate_crypto_key(
                &algorithm,
                true,
                &["sign", "verify"],
            )
            .await
            .map_err(|_| CryptoError::KeyGeneration)?;

            let pub_key =
                Self::export_crypto_key(&key_pair.get_public_key(), "spki")
                    .await?;
            let pri_key =
                Self::export_crypto_key(&key_pair.get_private_key(), "pkcs8")
                    .await?;

            Ok((pub_key, pri_key))
        })
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64]> {
        Box::pin(async move {
            let key = Self::import_crypto_key(
                pri_key,
                "pkcs8",
                &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                &["sign"],
            )
            .await?;

            let sig_prom = Self::crypto_subtle()
                .sign_with_object_and_u8_array(
                    &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                    &key,
                    data,
                )
                .map_err(|_| CryptoError::Signing)?;

            let result: ArrayBuffer = JsFuture::from(sig_prom)
                .await
                .map_err(|_| CryptoError::Signing)?
                .try_into()
                .map_err(|_: std::convert::Infallible| CryptoError::Signing)?;

            let sig: [u8; 64] = result
                .as_u8_slice()
                .try_into()
                .expect("Signature length incorrect");

            Ok(sig)
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool> {
        Box::pin(async move {
            let key = Self::import_crypto_key(
                pub_key,
                "spki",
                &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                &["verify"],
            )
            .await?;

            let verified_promise = Self::crypto_subtle()
                .verify_with_object_and_u8_array_and_u8_array(
                    &js_object(vec![("name", JsValue::from_str("Ed25519"))]),
                    &key,
                    sig,
                    data,
                )
                .map_err(|_| CryptoError::Verification)?;

            let result: bool = JsFuture::from(verified_promise)
                .await
                .map_err(|_| CryptoError::Verification)?
                .as_bool()
                .ok_or(CryptoError::Verification)?;

            Ok(result)
        })
    }

    // aes ctr
    fn aes_ctr_encrypt<'a>(
        hash: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            let usages = Array::of1(
                &JsValue::from_str("encrypt"),
                // &JsValue::from_str("decrypt"),
            );

            let ikm_buf = Uint8Array::from(hash.as_slice()).buffer();

            let key_js = JsFuture::from(
                subtle
                    .import_key_with_object(
                        "raw",
                        &ikm_buf.into(),
                        &js_object(vec![("name", "AES-CTR")]),
                        false,
                        &usages,
                    )
                    .map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?;
            let base_key: CryptoKey =
                key_js.dyn_into().map_err(|_| CryptoError::KeyImport)?;

            let params = AesCtrParams::new(
                "AES-CTR",
                &Uint8Array::from(iv.as_slice()),
                64u8,
            );

            let pt = Uint8Array::from(plaintext);

            let ct = JsFuture::from(
                subtle
                    .encrypt_with_object_and_buffer_source(
                        &params.into(),
                        &base_key,
                        &pt,
                    )
                    .map_err(|_| CryptoError::Encryption)?,
            )
            .await
            .map_err(|_| CryptoError::Encryption)?;

            let ct_buf: ArrayBuffer =
                ct.dyn_into().map_err(|_| CryptoError::Encryption)?;
            let ct_bytes = Uint8Array::new(&ct_buf).to_vec();

            Ok(ct_bytes)
        })
    }

    fn aes_ctr_decrypt<'a>(
        hash: &'a [u8; 32],
        iv: &'a [u8; 16],
        ciphertext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>> {
        Box::pin(async move {
            let subtle = CryptoWeb::crypto_subtle();

            let usages = Array::of1(
                // &JsValue::from_str("encrypt"),
                &JsValue::from_str("decrypt"),
            );

            let ikm_buf = Uint8Array::from(hash.as_slice()).buffer();

            let key_js = JsFuture::from(
                subtle
                    .import_key_with_object(
                        "raw",
                        &ikm_buf.into(),
                        &js_object(vec![("name", "AES-CTR")]),
                        false,
                        &usages,
                    )
                    .map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?;
            let base_key: CryptoKey =
                key_js.dyn_into().map_err(|_| CryptoError::KeyImport)?;

            let params = AesCtrParams::new(
                "AES-CTR",
                &Uint8Array::from(iv.as_slice()),
                64u8,
            );

            let ct = Uint8Array::from(ciphertext);

            let pt = JsFuture::from(
                subtle
                    .decrypt_with_object_and_buffer_source(
                        &params.into(),
                        &base_key,
                        &ct,
                    )
                    .map_err(|_| CryptoError::Decryption)?,
            )
            .await
            .map_err(|_| CryptoError::Decryption)?;

            let pt_buf: ArrayBuffer =
                pt.dyn_into().map_err(|_| CryptoError::Decryption)?;
            let pt_bytes = Uint8Array::new(&pt_buf).to_vec();

            Ok(pt_bytes)
        })
    }

    fn key_upwrap<'a>(
        // Key Encryption Key (AES-256)
        kek_bytes: &'a [u8; 32],
        // The AES-CTR key to wrap
        key_to_wrap_bytes: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40]> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            // Import the Key Encryption Key (KEK)
            let kek_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-KW"))]);

            let kek_promise = subtle.import_key_with_object(
                "raw",
                &Uint8Array::from(kek_bytes.as_slice()).buffer(),
                &kek_algorithm,
                false, // not extractable
                &Array::of2(
                    &JsValue::from_str("wrapKey"),
                    &JsValue::from_str("unwrapKey"),
                ),
            );

            let kek: CryptoKey = JsFuture::from(
                kek_promise.map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?
            .dyn_into()
            .map_err(|_| CryptoError::KeyImport)?;

            // Import the key to be wrapped (AES-CTR key)
            let key_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-CTR"))]);

            let key_promise = subtle.import_key_with_object(
                "raw",
                &Uint8Array::from(key_to_wrap_bytes.as_slice()).buffer(),
                &key_algorithm,
                true, // must be extractable to wrap it
                &Array::of2(
                    &JsValue::from_str("encrypt"),
                    &JsValue::from_str("decrypt"),
                ),
            );

            let key_to_wrap: CryptoKey = JsFuture::from(
                key_promise.map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?
            .dyn_into()
            .map_err(|_| CryptoError::KeyImport)?;

            // Wrap the key
            let wrap_promise = subtle.wrap_key_with_str(
                "raw",        // format to wrap in
                &key_to_wrap, // key to wrap
                &kek,         // wrapping key
                "AES-KW",     // wrapping algorithm
            );

            let wrapped_buffer = JsFuture::from(
                wrap_promise.map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?;

            let uint8_array = Uint8Array::new(&wrapped_buffer);
            let mut result: [u8; 40] = vec![0u8; uint8_array.length() as usize]
                .try_into()
                .map_err(|_| CryptoError::KeyImport)?;
            uint8_array.copy_to(&mut result);

            Ok(result)
        })
    }

    fn key_unwrap<'a>(
        kek_bytes: &'a [u8; 32], // Key Encryption Key (same as used for wrapping)
        wrapped_key: &'a [u8; 40], // The wrapped key data
    ) -> AsyncCryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let subtle = CryptoWeb::crypto_subtle();

            // Import the Key Encryption Key (KEK)
            let kek_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-KW"))]);

            let kek_promise = subtle.import_key_with_object(
                "raw",
                &Uint8Array::from(kek_bytes.as_slice()).buffer(),
                &kek_algorithm,
                false, // not extractable
                &Array::of2(
                    &JsValue::from_str("wrapKey"),
                    &JsValue::from_str("unwrapKey"),
                ),
            );

            let kek: CryptoKey = JsFuture::from(
                kek_promise.map_err(|_| CryptoError::KeyImport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyImport)?
            .dyn_into()
            .map_err(|_| CryptoError::KeyImport)?;

            // Unwrap the key
            let unwrapped_algorithm =
                js_object(vec![("name", JsValue::from_str("AES-CTR"))]);

            // Convert wrapped_key to Uint8Array
            let wrapped_key_array = Uint8Array::from(wrapped_key.as_slice());

            let unwrap_promise = subtle
                .unwrap_key_with_js_u8_array_and_str_and_object(
                    "raw",                // format the wrapped key is in
                    &wrapped_key_array,   // wrapped key as Uint8Array
                    &kek,                 // unwrapping key
                    "AES-KW",             // unwrapping algorithm
                    &unwrapped_algorithm, // algorithm of the unwrapped key
                    true,                 // extractable
                    &Array::of2(
                        &JsValue::from_str("encrypt"),
                        &JsValue::from_str("decrypt"),
                    ),
                );

            let unwrapped_key: CryptoKey = JsFuture::from(
                unwrap_promise.map_err(|_| CryptoError::KeyExport)?,
            )
            .await
            .map_err(|_| CryptoError::KeyExport)?
            .dyn_into()
            .map_err(|_| CryptoError::KeyExport)?;

            // Export the unwrapped key as raw bytes
            let export_promise = subtle
                .export_key("raw", &unwrapped_key)
                .map_err(|_| CryptoError::KeyExport)?;

            let exported_buffer = JsFuture::from(export_promise)
                .await
                .map_err(|_| CryptoError::KeyExport)?;

            let uint8_array = Uint8Array::new(&exported_buffer);
            let mut result: [u8; 32] = vec![0u8; uint8_array.length() as usize]
                .try_into()
                .map_err(|_| CryptoError::KeyExport)?;
            uint8_array.copy_to(&mut result);

            Ok(result)
        })
    }

    // x25519 key gen
    fn gen_x25519<'a>() -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48])> {
        Box::pin(async move {
            let algorithm =
                js_object(vec![("name", JsValue::from_str("X25519"))]);

            let key_pair: CryptoKeyPair = Self::generate_crypto_key(
                &algorithm,
                true,
                &["deriveKey", "deriveBits"],
            )
            .await
            .map_err(|_| CryptoError::KeyGeneration)?;

            let pub_key: [u8; 44] =
                Self::export_crypto_key(&key_pair.get_public_key(), "spki")
                    .await
                    .map_err(|_| CryptoError::KeyGeneration)?
                    .try_into()
                    .map_err(|_| CryptoError::KeyGeneration)?;
            let pri_key: [u8; 48] =
                Self::export_crypto_key(&key_pair.get_private_key(), "pkcs8")
                    .await
                    .map_err(|_| CryptoError::KeyGeneration)?
                    .try_into()
                    .map_err(|_| CryptoError::KeyGeneration)?;

            Ok((pub_key, pri_key))
        })
    }

    fn derive_x25519<'a>(
        my_raw: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> AsyncCryptoResult<'a, Vec<u8>> {
        Box::pin(async move {
            let subtle = Self::crypto_subtle();

            // Private Key
            let pri_key_algorithm =
                js_object(vec![("name", JsValue::from_str("X25519"))]);

            let pri_key_promise = subtle
                .import_key_with_object(
                    "pkcs8",
                    &Uint8Array::from(my_raw.as_slice()).buffer(),
                    &pri_key_algorithm,
                    false, // not extractable
                    &Array::of2(
                        &JsValue::from_str("deriveKey"),
                        &JsValue::from_str("deriveBits"),
                    ),
                )
                .map_err(|_| CryptoError::KeyImport)?;

            let pri_key: CryptoKey = JsFuture::from(pri_key_promise)
                .await
                .map_err(|_| CryptoError::KeyImport)?
                .dyn_into()
                .map_err(|_| CryptoError::KeyImport)?;

            // Public Key
            let pub_key_promise = subtle
                .import_key_with_object(
                    "spki",
                    &Uint8Array::from(peer_pub.as_slice()).buffer(),
                    &pri_key_algorithm, // same algorithm object
                    false,              // not extractable
                    &Array::new(),      // no usage for public key
                )
                .map_err(|_| CryptoError::KeyImport)?;

            let pub_key: CryptoKey = JsFuture::from(pub_key_promise)
                .await
                .map_err(|_| CryptoError::KeyImport)?
                .dyn_into()
                .map_err(|_| CryptoError::KeyImport)?;

            let derive_algorithm = js_object(vec![
                ("name", JsValue::from_str("X25519")),
                ("public", pub_key.into()),
            ]);

            // Derive bits
            let derive_promise = subtle
                .derive_bits_with_object(&derive_algorithm, &pri_key, 256u32)
                .map_err(|_| CryptoError::KeyGeneration)?;

            let derived_buffer = JsFuture::from(derive_promise)
                .await
                .map_err(|_| CryptoError::KeyExport)?;

            let uint8_array = Uint8Array::new(&derived_buffer);
            let mut result = vec![0u8; uint8_array.length() as usize];
            uint8_array.copy_to(&mut result);

            Ok(result)
        })
    }
}
