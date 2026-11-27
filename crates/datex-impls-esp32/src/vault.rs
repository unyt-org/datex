//! `Vault` implementation for ESP32 using the existing `nvs` partition.

use alloc::{format, string::String};
use core::cell::RefCell;

use critical_section::Mutex;
use datex_facades::vault::Vault;
use esp_hal::peripherals::FLASH;
use esp_nvs::{Key, Nvs};
use esp_storage::FlashStorage;

/// Matches `nvs, data, nvs, 0x9000, 0x6000` in the partition table
/// FIXME: make this configurable dynamically
const NVS_OFFSET: usize = 0x9000;
const NVS_SIZE: usize = 0x6000;

/// NVS namespace for DATEX vault
const NAMESPACE: Key = Key::from_str("datex-vault");

static NVS: Mutex<RefCell<Option<Nvs<FlashStorage<'static>>>>> =
    Mutex::new(RefCell::new(None));

/// Must be called once before starting the main DATEX runtime loop
pub fn init_flash(flash: FLASH<'static>) {
    let nvs = Nvs::new(NVS_OFFSET, NVS_SIZE, FlashStorage::new(flash))
        .expect("failed to open nvs partition");
    critical_section::with(|cs| *NVS.borrow_ref_mut(cs) = Some(nvs));
}

fn with_nvs<R>(f: impl FnOnce(&mut Nvs<FlashStorage<'static>>) -> R) -> R {
    critical_section::with(|cs| {
        let mut nvs = NVS.borrow_ref_mut(cs);
        f(nvs.as_mut().expect("vault not initialized"))
    })
}

/// hash endpoint string + key (NVS key limit is 15bytes)
fn nvs_key(endpoint: &str, key: &str) -> Key {
    // FNV-1a, 64-bit
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in endpoint.bytes().chain(core::iter::once(b'/')).chain(key.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let hex = format!("{:014x}", hash & 0x00ff_ffff_ffff_ffff);
    Key::from_str(&hex)
}

pub struct VaultEsp32;

impl Vault for VaultEsp32 {
    fn new(location: &str) -> Self {
        // FIXME: location must be set to "nvs" for now
        if location != "nvs" {
            panic!("VaultEsp32 only supports 'nvs' location, got: {}", location);
        }
        VaultEsp32
    }

    fn store(&self, endpoint: &str, key: &str, data: &str) {
        let k = nvs_key(endpoint, key);
        let result = with_nvs(|nvs| nvs.set(&NAMESPACE, &k, data));
        result.expect("vault: failed to store value");
    }
    
    fn get(&self, endpoint: &str, key: &str) -> Option<String> {
        let k = nvs_key(endpoint, key);
        let result = with_nvs(|nvs| nvs.get::<String>(&NAMESPACE, &k));
        result.ok()
    }
    

    fn delete(&self, endpoint: &str, key: &str) {
        let k = nvs_key(endpoint, key);
        let _ = with_nvs(|nvs| nvs.delete(&NAMESPACE, &k));
    }
}
