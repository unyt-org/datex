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
// FIXME #228: Clippy bug / Rust Rover bug?!
// #![allow(unused_parens)]
#![feature(associated_type_defaults)]
#![feature(core_float_math)]
#![feature(thread_local)]
#![feature(future_join)]
#![allow(static_mut_refs)]

extern crate num_integer;

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod channel;
pub mod crypto;
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
pub mod generator;
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
extern crate self as datex_core;

// Note: always use collections mod for HashMap and HashSet
pub mod collections {
    #[cfg(feature = "std")]
    pub use std::collections::{HashMap, HashSet};

    #[cfg(all(not(feature = "std")))]
    pub use hashbrown::{HashMap, HashSet, hash_map, hash_set};
}

pub mod std_sync {
    #[cfg(not(feature = "std"))]
    pub use spin::Mutex;
    #[cfg(feature = "std")]
    pub use std::sync::Mutex;
}

pub mod time {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            pub use web_time::*;
        } else if #[cfg(feature = "std")] {
            pub use std::time::*;
        } else if #[cfg(feature = "embedded")] {
            pub use embedded_time::*;
        } else {
            pub use crate::stub::time::*;
        }
    }
}

pub mod random {
    #[cfg(not(feature = "std"))]
    pub use foldhash::fast::RandomState;
    #[cfg(feature = "std")]
    pub use std::hash::RandomState;
}
