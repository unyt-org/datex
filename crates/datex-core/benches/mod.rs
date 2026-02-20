#![feature(custom_test_frameworks)]
#![feature(thread_local)]
#![test_runner(criterion::runner)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::alloc_instead_of_core)]
#![allow(clippy::std_instead_of_core)]
#![feature(assert_matches)]

cfg_if::cfg_if! {
    if #[cfg(all(feature = "compiler", feature = "decompiler"))] {
        mod json;
        use crate::json::json_benches;
    }
}

use crate::runtime::runtime_benches;
use criterion::criterion_main;
mod runtime;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "compiler", feature = "decompiler"))] {
        criterion_main!(runtime_benches, json_benches);
    } else {
        criterion_main!(runtime_benches);
    }
}
