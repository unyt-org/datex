#![cfg_attr(not(feature = "std"), no_std)]
#![feature(coroutines)]
#![feature(iter_from_coroutine)]
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

extern crate alloc;
extern crate num_integer;

#[cfg(feature = "std")]
extern crate std;

pub mod channel;
pub mod dif;
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
#[cfg(feature = "compiler")]
pub mod parser;
pub mod references;
pub mod runtime;
#[cfg(feature = "compiler")]
pub mod type_inference;
#[cfg(feature = "compiler")]
pub mod visitor;

pub mod core_compiler;
pub mod dxb_parser;
pub mod serde;
mod stub;
pub mod task;
pub mod traits;
pub mod types;
pub mod utils;
pub mod values;

// reexport macros
pub use datex_macros as macros;
extern crate core;

// HashMap and HashSet that work in both std and no_std environments.
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
        if #[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))] {
            pub use datex_crypto_embedded::CryptoEmbedded as CryptoImpl;
        } else if #[cfg(target_arch = "wasm32")] {
            pub use datex_crypto_web::CryptoWeb as CryptoImpl;
        } else if #[cfg(any(feature = "std", test))] {
            pub use datex_crypto_native::CryptoNative as CryptoImpl;
        } else {
            pub use crate::stub::crypto::CryptoStub as CryptoImpl;
        }
    }
}

pub mod time {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            pub use web_time::{Duration, Instant};
        } else if #[cfg(feature = "std")] {
            pub use std::time::{Duration, Instant};
        } else if #[cfg(feature = "embedded")] {
            pub use embedded_time::{duration::*, Instant};
        } else {
            pub use crate::stub::time::{Duration, Instant};
        }
    }

    /// Monotonic nanoseconds since a crate-defined start point.
    #[inline]
    pub fn now_ns() -> u64 {
        start_instant().elapsed().as_nanos() as u64
    }
    #[inline]
    pub fn start_instant() -> Instant {
        #[cfg(target_has_atomic = "ptr")]
        {
            use core::sync::atomic::{AtomicU8, Ordering};

            static INIT: AtomicU8 = AtomicU8::new(0);

            // Safety: we write START once, then only read it.
            static mut START: core::mem::MaybeUninit<Instant> =
                core::mem::MaybeUninit::uninit();

            if INIT.load(Ordering::Acquire) == 0 {
                // Try to become the initializer.
                if INIT
                    .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    unsafe {
                        START.write(Instant::now());
                    }
                    INIT.store(2, Ordering::Release);
                } else {
                    // Wait until initialized.
                    while INIT.load(Ordering::Acquire) != 2 {
                        core::hint::spin_loop();
                    }
                }
            } else {
                // Wait until initialized.
                while INIT.load(Ordering::Acquire) != 2 {
                    core::hint::spin_loop();
                }
            }

            unsafe { *START.assume_init_ref() }
        }

        // No atomics: deterministic fallback (time starts "now" every call).
        #[cfg(not(target_has_atomic = "ptr"))]
        {
            Instant::now()
        }
    }
}

pub mod random {
    #[cfg(not(feature = "std"))]
    pub use foldhash::fast::RandomState;
    #[cfg(feature = "std")]
    pub use std::hash::RandomState;
}
