use datex_crypto_facade::{
    crypto::{Crypto, CryptoResult},
    error::CryptoError,
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
use rand::Rng;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct CryptoNative;
impl Crypto for CryptoNative {
    fn create_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        (0..length).map(|_| rng.random()).collect()
    }

    fn hash_sha256(to_digest: &'_ [u8]) -> CryptoResult<'_, [u8; 32]> {
        Box::pin(async move {
            let hash = sha256(to_digest);
            Ok(hash)
        })
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let info = b"";
            let mut ctx = PkeyCtx::new_id(Id::HKDF)
                .map_err(|_| CryptoError::KeyGeneration)?;
            ctx.derive_init().map_err(|_| CryptoError::KeyGeneration)?;
            ctx.set_hkdf_mode(HkdfMode::EXTRACT_THEN_EXPAND)
                .map_err(|_| CryptoError::KeyGeneration)?;
            ctx.set_hkdf_md(Md::sha256())
                .map_err(|_| CryptoError::KeyGeneration)?;
            ctx.set_hkdf_salt(salt)
                .map_err(|_| CryptoError::KeyGeneration)?;
            ctx.set_hkdf_key(ikm)
                .map_err(|_| CryptoError::KeyGeneration)?;
            ctx.add_hkdf_info(info)
                .map_err(|_| CryptoError::KeyGeneration)?;
            let mut okm = [0u8; 32_usize];
            ctx.derive(Some(&mut okm))
                .map_err(|_| CryptoError::KeyGeneration)?;
            Ok(okm)
        })
    }
    // EdDSA keygen
    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        Box::pin(async move {
            let key = PKey::generate_ed25519()
                .map_err(|_| CryptoError::KeyGeneration)?;

            let public_key: Vec<u8> = key
                .public_key_to_der()
                .map_err(|_| CryptoError::KeyGeneration)?;
            let private_key: Vec<u8> = key
                .private_key_to_pkcs8()
                .map_err(|_| CryptoError::KeyGeneration)?;
            Ok((public_key, private_key))
        })
    }

    // EdDSA signature
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, [u8; 64]> {
        Box::pin(async move {
            let sig_key = PKey::private_key_from_pkcs8(pri_key)
                .map_err(|_| CryptoError::KeyImport)?;
            let mut signer = Signer::new_without_digest(&sig_key)
                .map_err(|_| CryptoError::Signing)?;
            let signature = signer
                .sign_oneshot_to_vec(data)
                .map_err(|_| CryptoError::Signing)?;
            let signature: [u8; 64] =
                signature.try_into().expect("Invalid signature length");
            Ok(signature)
        })
    }

    // EdDSA verification of signature
    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> CryptoResult<'a, bool> {
        Box::pin(async move {
            let public_key = PKey::public_key_from_der(pub_key)
                .map_err(|_| CryptoError::KeyImport)?;
            let mut verifier = Verifier::new_without_digest(&public_key)
                .map_err(|_| CryptoError::KeyImport)?;
            let verification = verifier
                .verify_oneshot(sig, data)
                .map_err(|_| CryptoError::Verification)?;
            Ok(verification)
        })
    }

    // AES CTR
    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        Box::pin(async move {
            let cipher = Cipher::aes_256_ctr();
            let mut enc = Crypter::new(cipher, Mode::Encrypt, key, Some(iv))
                .map_err(|_| CryptoError::Encryption)?;

            let mut out = vec![0u8; plaintext.len()];
            let count = enc
                .update(plaintext, &mut out)
                .map_err(|_| CryptoError::Encryption)?;
            out.truncate(count);
            Ok(out)
        })
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        ciphertext: &'a [u8],
    ) -> CryptoResult<'a, Vec<u8>> {
        Self::aes_ctr_encrypt(key, iv, ciphertext)
    }

    // AES KW
    fn key_upwrap<'a>(
        kek_bytes: &'a [u8; 32],
        rb: &'a [u8; 32],
    ) -> CryptoResult<'a, [u8; 40]> {
        Box::pin(async move {
            // Key encryption key
            let kek = AesKey::new_encrypt(kek_bytes)
                .map_err(|_| CryptoError::Encryption)?;

            // Key wrap
            let mut wrapped = [0u8; 40];
            let _length = wrap_key(&kek, None, &mut wrapped, rb);

            Ok(wrapped)
        })
    }

    fn key_unwrap<'a>(
        kek_bytes: &'a [u8; 32],
        cipher: &'a [u8; 40],
    ) -> CryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            // Key encryption key
            let kek = AesKey::new_decrypt(kek_bytes)
                .map_err(|_| CryptoError::Decryption)?;

            // Unwrap key
            let mut unwrapped: [u8; 32] = [0u8; 32];
            let _length = unwrap_key(&kek, None, &mut unwrapped, cipher);
            Ok(unwrapped)
        })
    }

    // Generate encryption keypair
    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        Box::pin(async move {
            let key = PKey::generate_x25519()
                .map_err(|_| CryptoError::KeyGeneration)?;
            let public_key: [u8; 44] = key
                .public_key_to_der()
                .map_err(|_| CryptoError::KeyGeneration)?
                .try_into()
                .map_err(|_| CryptoError::KeyGeneration)?;
            let private_key: [u8; 48] = key
                .private_key_to_pkcs8()
                .map_err(|_| CryptoError::KeyGeneration)?
                .try_into()
                .map_err(|_| CryptoError::KeyGeneration)?;
            Ok((public_key, private_key))
        })
    }

    // Derive shared secret on x255109
    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> CryptoResult<'a, Vec<u8>> {
        Box::pin(async move {
            let peer_pub = PKey::public_key_from_der(peer_pub)
                .map_err(|_| CryptoError::KeyImport)?;
            let my_priv = PKey::private_key_from_pkcs8(pri_key)
                .map_err(|_| CryptoError::KeyImport)?;

            let mut deriver = Deriver::new(&my_priv)
                .map_err(|_| CryptoError::KeyGeneration)?;
            deriver
                .set_peer(&peer_pub)
                .map_err(|_| CryptoError::KeyGeneration)?;
            let derived = deriver
                .derive_to_vec()
                .map_err(|_| CryptoError::KeyGeneration)?;
            Ok(derived)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CryptoNative;
    use datex_crypto_facade::crypto::Crypto;
    #[test]
    fn test_uuid() {
        let uuid1 = CryptoNative::create_uuid();
        let uuid2 = CryptoNative::create_uuid();
        assert_ne!(uuid1, uuid2);

        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid2.len(), 36);
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = CryptoNative::random_bytes(16);
        let bytes2 = CryptoNative::random_bytes(16);
        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2);
    }
}
