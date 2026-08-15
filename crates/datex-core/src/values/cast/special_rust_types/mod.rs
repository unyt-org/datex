mod r#box;
pub use r#box::*;

mod option;
pub use option::*;

mod r#vec;
pub use r#vec::*;

mod hash_map;
pub use hash_map::*;

use crate::shared_values::PointerAddress;

pub fn rust_none_marker() -> PointerAddress {
    // TODO better addr
    PointerAddress::self_owned([1u8, 2u8, 3u8, 4u8, 5u8])
}

pub fn rust_some_marker() -> PointerAddress {
    // TODO better addr
    PointerAddress::self_owned([1u8, 2u8, 3u8, 4u8, 6u8])
}
