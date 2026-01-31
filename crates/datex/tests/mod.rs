#![feature(assert_matches)]
#![feature(iter_from_coroutine)]
#![feature(coroutines)]
#![feature(thread_local)]
#![feature(box_patterns)]
#![feature(gen_blocks)]
#![allow(static_mut_refs)]
extern crate core;

pub mod network;
pub mod values;

pub mod dif;
pub mod json;
pub mod parser;
mod mock_globals;