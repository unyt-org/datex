//! The `prelude` module contains re-exports of commonly used types, traits, and functions from the alloc crate and other modules.
pub use crate::collections::*;

pub use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    format,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
