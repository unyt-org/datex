#![no_std]
#![feature(thread_local)]

#[cfg(test)]
extern crate std;

extern crate alloc;

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use datex_crypto_facade::crypto::{AsyncCryptoResult, Crypto};

use aes::cipher::{KeyIvInit, StreamCipher};
use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
        spki::der::pem::LineEnding,
    },
};
use hkdf::Hkdf;
use rand::{rand_core::TryRng, rngs::SysRng};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

use der::{Decode, Encode, asn1::BitStringRef};
use pkcs8::{AlgorithmIdentifierRef, ObjectIdentifier, PrivateKeyInfo};
use spki::SubjectPublicKeyInfoRef;

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
mod hal {
    use esp_hal::rng::Rng;
    use spin::{Mutex, MutexGuard, Once};
    use static_cell::StaticCell;

    static RNG: StaticCell<Mutex<Rng>> = StaticCell::new();
    static INIT: Once<&'static Mutex<Rng>> = Once::new();

    pub fn rng() -> MutexGuard<'static, Rng> {
        let m = INIT.call_once(|| RNG.init(Mutex::new(Rng::new())));

        m.lock()
    }
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
pub use hal::rng;

struct InfallibleRng;
impl InfallibleRng {
    fn read(&mut self, _: &mut [u8]) {
        panic!("RNG not supported on this platform");
    }
}

#[cfg(not(any(target_arch = "xtensa", target_arch = "riscv32")))]
fn rng() -> InfallibleRng {
    InfallibleRng
}

#[derive(Debug, Clone)]
pub struct CryptoEsp32;

impl Crypto for CryptoEsp32 {
    fn create_uuid() -> String {
        // TODO #705: use uuid crate?
        let mut bytes = [0u8; 16];
        rng().read(&mut bytes);

        // set version to 4 -- random
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        // set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]),
            u16::from_be_bytes([bytes[8], bytes[9]]),
            u64::from_be_bytes([
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15], 0, 0
            ]) >> 16
        )
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        rng().read(&mut bytes);
        bytes
    }

    fn hash_sha256<'a>(
        _to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        Box::pin(async move {
            // placeholder comment
            todo!("#706 Undescribed by author.")
        })
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        _salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        Box::pin(async move {
            let mut okm = [0u8; 32];
            let ctx = Hkdf::<Sha256>::new(None, ikm);
            ctx.expand(b"", &mut okm).unwrap();
            Ok(okm)
        })
    }

    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes256>;
            let mut msg = plaintext.to_vec();
            let mut cipher = Aes128Ctr64LE::new(key.into(), iv.into());
            cipher.apply_keystream(msg.as_mut_slice());
            Ok(msg)
        })
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Self::aes_ctr_encrypt(key, iv, cipher)
    }

    fn key_wrap_rfc3394<'a>(
        _kek_bytes: &'a [u8; 32],
        _rb: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        Box::pin(async move {
            // placeholder comment
            todo!("#712 Undescribed by author.")
        })
    }

    fn key_unwrap_rfc3394<'a>(
        _kek_bytes: &'a [u8; 32],
        _cipher: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        Box::pin(async move {
            // placeholder comment
            todo!("#713 Undescribed by author.")
        })
    }

    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, (Vec<u8>, Vec<u8>), Self::Ed25519GenError> {
        Box::pin(async move {
            let mut key = [0u8; 32];
            SysRng.try_fill_bytes(&mut key).unwrap();
            let x = SigningKey::from_bytes(&key);

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
            // note: raw pub key
            // let temp = x.verifying_key().to_bytes();
            let pub_key = x
                .verifying_key()
                .to_public_key_der()
                .unwrap()
                .as_bytes()
                .to_vec();
            Ok((pub_key, pri_x))
        })
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        Box::pin(async move {
            let prepped_key: [u8; 48] = pri_key.to_vec().try_into().unwrap();
            Ok(SigningKey::from_pkcs8_der(&prepped_key)
                .unwrap()
                .sign(data)
                .to_bytes())
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        Box::pin(async move {
            let sign: [u8; 64] = sig.try_into().unwrap();
            let ver = VerifyingKey::from_public_key_der(pub_key).unwrap();
            Ok(ver.verify(data, &Signature::from_bytes(&sign)).is_ok())
        })
    }

    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 44], [u8; 48]), Self::X25519GenError> {
        Box::pin(async move {
            /*
            let pri_key = StaticSecret::random().to_bytes();
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            Ok((pub_key, pri_key.to_bytes()));
            */
            let pri_key = StaticSecret::random();
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            let oid = ObjectIdentifier::new("1.3.101.110").unwrap();

            // For a historical reason the private key in pkcs8 is prefixed twice
            // with the octet string instruction code followed by the length of the octet string
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

            // PEM encoding
            /*
            let pri_pem = pkcs8::SecretDocument::from_pkcs8_der(&pri_x)
                .unwrap()
                .to_pem("PRIVATE KEY", LineEnding::default())
                .unwrap()
                .to_ascii_uppercase();
            */

            let pub_spki = SubjectPublicKeyInfoRef {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                subject_public_key: BitStringRef::new(0, &pub_key).unwrap(),
            }
            .to_der()
            .unwrap();

            // sanity check
            let z = PrivateKeyInfo::from_der(pri_x.as_slice()).unwrap();
            assert_eq!(prepped_key.to_vec(), z.private_key.to_vec());

            let public_key: [u8; 44] = pub_spki.try_into().unwrap();
            let private_key: [u8; 48] = pri_x.try_into().unwrap();
            Ok((public_key, private_key))
        })
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        Box::pin(async move {
            /*
            let x = StaticSecret::from(pri_key);
            let shared_sec = x.diffie_hellman(&peer_pub.into()).to_bytes();
            Ok(shared_sec.to_vec());
            */
            let x: [u8; 32] = pri_key[16..].try_into().unwrap();
            let xx = StaticSecret::from(x);
            let y: [u8; 32] = peer_pub[12..].try_into().unwrap();
            let yy = PublicKey::from(y);
            Ok(xx.diffie_hellman(&yy).to_bytes())
        })
    }
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
pub fn now_ms() -> u64 {
    let rtc = esp_hal::rtc_cntl::Rtc::new(unsafe {
        esp_hal::peripherals::Peripherals::steal()
            .LPWR
            .clone_unchecked()
    });
    rtc.current_time_us() / 1000
}

#[cfg(test)]
mod tests {
    use datex_crypto_facade::crypto::Crypto;

    use super::CryptoEsp32;

    #[tokio::test]
    async fn test_x25519() {
        let (a_pub_key, a_pri_key) = CryptoEsp32::gen_x25519().await.unwrap();
        let (b_pub_key, b_pri_key) = CryptoEsp32::gen_x25519().await.unwrap();

        let a_sec = CryptoEsp32::derive_x25519(&a_pri_key, &b_pub_key)
            .await
            .unwrap();
        let b_sec = CryptoEsp32::derive_x25519(&b_pri_key, &a_pub_key)
            .await
            .unwrap();
        assert_eq!(a_sec, b_sec);
    }

    #[tokio::test]
    async fn test_ed25519() {
        let msg = b"SomeMsg".to_vec();
        let (pub_key, pri_key) = CryptoEsp32::gen_ed25519().await.unwrap();
        let sign = CryptoEsp32::sig_ed25519(pri_key.as_slice(), msg.as_slice())
            .await
            .unwrap();
        let ver = CryptoEsp32::ver_ed25519(pub_key.as_slice(), &sign, &msg)
            .await
            .unwrap();

        // std::println!("{:?} - {}", pri_key, pri_key.len());
        assert_eq!(pub_key.len(), 44);
        assert_eq!(pri_key.len(), 48);
        assert!(ver);
    }

    #[tokio::test]
    async fn check_ed_and_x() {
        let ser_pub: [u8; 44] = [
            48, 42, 48, 5, 6, 3, 43, 101, 110, 3, 33, 0, 106, 251, 212, 218,
            131, 11, 184, 255, 109, 73, 74, 73, 124, 75, 108, 2, 190, 233, 34,
            228, 244, 30, 86, 193, 70, 36, 155, 81, 223, 181, 76, 83,
        ];
        let ser_pri: [u8; 48] = [
            48, 46, 2, 1, 0, 48, 5, 6, 3, 43, 101, 110, 4, 34, 4, 32, 181, 170,
            218, 225, 185, 123, 28, 10, 5, 76, 13, 28, 89, 124, 205, 151, 225,
            132, 183, 90, 104, 74, 139, 47, 152, 207, 100, 33, 2, 184, 166,
            217,
        ];
        let cli_pub: [u8; 44] = [
            48, 42, 48, 5, 6, 3, 43, 101, 110, 3, 33, 0, 244, 222, 220, 93,
            110, 52, 47, 78, 15, 33, 207, 47, 84, 139, 123, 228, 254, 72, 241,
            22, 17, 211, 37, 40, 191, 128, 232, 197, 104, 140, 167, 12,
        ];
        let cli_pri: [u8; 48] = [
            48, 46, 2, 1, 0, 48, 5, 6, 3, 43, 101, 110, 4, 34, 4, 32, 187, 195,
            133, 120, 172, 61, 170, 25, 75, 103, 226, 163, 137, 242, 206, 180,
            177, 128, 122, 13, 236, 34, 83, 207, 9, 137, 104, 108, 139, 23,
            107, 79,
        ];
        let ser_sec = CryptoEsp32::derive_x25519(&ser_pri, &cli_pub)
            .await
            .unwrap();
        let cli_sec = CryptoEsp32::derive_x25519(&cli_pri, &ser_pub)
            .await
            .unwrap();
        let shared_secret_check: [u8; 32] = [
            186, 148, 122, 28, 89, 38, 223, 152, 165, 218, 70, 66, 159, 86,
            169, 235, 167, 32, 203, 45, 153, 141, 39, 112, 39, 186, 77, 65,
            230, 38, 154, 34,
        ];
        assert_eq!(ser_sec, shared_secret_check);
        assert_eq!(cli_sec, shared_secret_check);

        // signatures
        let data = b"Some message to  sign".to_vec();
        let pub_key: [u8; 44] = [
            48, 42, 48, 5, 6, 3, 43, 101, 112, 3, 33, 0, 23, 90, 144, 62, 109,
            49, 38, 236, 202, 74, 60, 0, 251, 56, 16, 83, 167, 236, 51, 191,
            90, 202, 225, 244, 59, 24, 242, 79, 112, 133, 51, 184,
        ];
        let pri_key: [u8; 48] = [
            48, 46, 2, 1, 0, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 254, 225,
            119, 84, 255, 1, 51, 183, 133, 59, 19, 6, 176, 150, 37, 219, 178,
            48, 168, 22, 139, 189, 12, 209, 26, 237, 58, 130, 111, 169, 62,
            252,
        ];
        let sig = CryptoEsp32::sig_ed25519(&pri_key, &data).await.unwrap();
        let sig_check: [u8; 64] = [
            10, 93, 243, 184, 21, 238, 165, 132, 57, 149, 73, 176, 98, 96, 160,
            186, 31, 197, 47, 167, 154, 168, 185, 102, 243, 241, 76, 128, 220,
            34, 128, 218, 17, 90, 106, 167, 233, 16, 213, 179, 48, 2, 85, 64,
            249, 76, 214, 168, 132, 191, 198, 205, 72, 42, 35, 136, 228, 73,
            174, 116, 222, 76, 130, 3,
        ];

        assert_eq!(sig, sig_check);
        assert!(
            CryptoEsp32::ver_ed25519(&pub_key, &sig, &data)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_aes_ctr() {
        let key = [0u8; 32];
        let nonce = [0u8; 16];
        let msg = b"SomeMsg".to_vec();
        let encrypted =
            CryptoEsp32::aes_ctr_encrypt(&key, &nonce, &msg.clone())
                .await
                .unwrap();
        let decrypted =
            CryptoEsp32::aes_ctr_decrypt(&key, &nonce, &encrypted.clone())
                .await
                .unwrap();
        assert_ne!(msg, encrypted);
        assert_eq!(msg, decrypted);
    }

    #[tokio::test]
    async fn test_hkdf() {
        let key = [0u8; 32];
        let x = CryptoEsp32::hkdf_sha256(&key, &[0u8; 16]).await.unwrap();
        let y: [u8; 32] = [
            223, 114, 4, 84, 111, 27, 238, 120, 184, 83, 36, 167, 137, 140,
            161, 25, 179, 135, 224, 19, 134, 209, 174, 240, 55, 120, 29, 74,
            138, 3, 106, 238,
        ];
        assert_eq!(x, y);
    }
}
