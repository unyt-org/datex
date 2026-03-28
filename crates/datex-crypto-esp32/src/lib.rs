#![no_std]
#![feature(thread_local)]

#[cfg(test)]
extern crate std;

extern crate alloc;

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use datex_crypto_facade::crypto::{AsyncCryptoResult, Crypto};

use aes::cipher::{KeyIvInit, StreamCipher};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

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
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        Box::pin(async move {
            let x = Sha256::digest(to_digest);
            let y: [u8; 32] = x.try_into().unwrap();
            Ok(y)
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
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::Ed25519GenError> {
        Box::pin(async move {
            let key: [u8; 32] = Self::random_bytes(32).try_into().unwrap();
            let x = SigningKey::from_bytes(&key);
            let pub_key = x.verifying_key().to_bytes();
            Ok((pub_key, x.to_bytes()))
        })
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        Box::pin(async move {
            let prepped_key: [u8; 32] = pri_key.to_vec().try_into().unwrap();
            Ok(SigningKey::from_bytes(&prepped_key).sign(data).to_bytes())
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        Box::pin(async move {
            let sign: [u8; 64] = sig.try_into().unwrap();
            let prepped_key: [u8; 32] = pub_key.to_vec().try_into().unwrap();
            let ver = VerifyingKey::from_bytes(&prepped_key).unwrap();
            Ok(ver.verify(data, &Signature::from_bytes(&sign)).is_ok())
        })
    }

    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::X25519GenError> {
        Box::pin(async move {
            let key: [u8; 32] = Self::random_bytes(32).try_into().unwrap();
            let pri_key = StaticSecret::from(key);
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            Ok((pub_key, pri_key.to_bytes()))
        })
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 32],
        peer_pub: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        Box::pin(async move {
            let x: [u8; 32] = pri_key.to_vec().try_into().unwrap();
            let y: [u8; 32] = peer_pub.to_vec().try_into().unwrap();
            let private_key = StaticSecret::from(x);
            let public_key = PublicKey::from(y);
            Ok(private_key.diffie_hellman(&public_key).to_bytes())
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
        assert_eq!(pub_key.len(), 32);
        assert_eq!(pri_key.len(), 32);
        assert!(ver);
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
