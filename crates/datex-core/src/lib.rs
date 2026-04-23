#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]
#![feature(gen_blocks)]
#![feature(async_iterator)]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(box_patterns)]
#![feature(if_let_guard)]
#![feature(try_trait_v2)]
// #![allow(unused_parens)]
#![feature(associated_type_defaults)]
#![feature(core_float_math)]
#![feature(thread_local)]
#![feature(future_join)]
#![allow(static_mut_refs)]
#![feature(variant_count)]
extern crate alloc;
extern crate num_integer;

#[cfg(feature = "std")]
extern crate std;

pub mod channel;
pub mod prelude;

#[cfg(feature = "ast")]
pub mod ast;
#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "decompiler")]
pub mod decompiler;
#[cfg(feature = "compiler")]
pub mod fmt;
pub mod global;
pub mod libs;
#[cfg(all(feature = "lsp", feature = "std"))]
pub mod lsp;
pub mod network;
#[cfg(feature = "parser")]
pub mod parser;
pub mod runtime;
pub mod shared_values;
#[cfg(feature = "compiler")]
pub mod type_inference;
#[cfg(feature = "compiler")]
pub mod visitor;

pub mod core_compiler;
mod dif;
pub mod disassembler;
pub mod dxb_parser;
#[cfg(all(feature = "macro_utils", feature = "std", feature = "compiler"))]
pub mod macro_utils;
pub mod serde;
mod stub;
pub mod task;
pub mod traits;
pub mod types;
pub mod utils;
mod value_updates;
pub mod values;

// reexport macros
pub use datex_macros_internal as macros;
extern crate core;

/// HashMap and HashSet that work in both std and no_std environments.
pub mod collections {
    #[cfg(feature = "std")]
    pub use std::collections::{HashMap, HashSet, hash_map, hash_set};

    #[cfg(not(feature = "std"))]
    pub use hashbrown::{HashMap, HashSet, hash_map, hash_set};
}

/// Reexport of Mutex that works in both std and no_std environments.
pub mod std_sync {
    #[cfg(not(feature = "std"))]
    pub use spin::Mutex;
    #[cfg(feature = "std")]
    pub use std::sync::Mutex;
}

/// Crypto implementations selection based on target architecture and features.
pub mod crypto {
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "target_native", test))] {
            pub use datex_crypto_native::CryptoNative as CryptoImpl;
        } else if #[cfg(feature = "target_esp32")] {
            pub use datex_crypto_esp32::CryptoEsp32 as CryptoImpl;
        } else if #[cfg(feature = "target_wasm")] {
            pub use datex_crypto_web::CryptoWeb as CryptoImpl;
        } else {
            pub use crate::stub::crypto::CryptoStub as CryptoImpl;
        }
    }
}

pub mod time {

    mod system_time {
        cfg_if::cfg_if! {
            if #[cfg(feature = "target_wasm")] {
                pub use web_time::{SystemTime, UNIX_EPOCH};
            } else if #[cfg(feature = "target_native")] {
                pub use std::time::{SystemTime, UNIX_EPOCH};
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "target_wasm")] {
            pub use web_time::{Instant};
        } else if #[cfg(feature = "std")] {
            pub use std::time::{Instant};
        } else if #[cfg(feature = "embassy_runtime")] {
            pub use embassy_time::{Instant};
        } else {
            pub use crate::stub::time::{Instant};
        }
    }

    // current unix timestamp in milliseconds
    pub fn now_ms() -> u64 {
        cfg_if::cfg_if! {
            if #[cfg(any(feature = "target_wasm", feature = "target_native"))] {
                use system_time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("System time is before UNIX_EPOCH")
                    .as_millis() as u64
            } else if #[cfg(feature = "target_esp32")] {
                datex_crypto_esp32::now_ms()
            } else {
                Instant::now().elapsed().as_millis() as u64
            }
        }
    }
}

pub mod random {
    #[cfg(not(feature = "std"))]
    pub use foldhash::fast::RandomState;
    #[cfg(feature = "std")]
    pub use std::hash::RandomState;
}
