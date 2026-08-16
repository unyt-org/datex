//! This module contains implementations of the `DatexValueProxySerialize` and `DatexValueProxyDeserialize` traits for special Rust types, such as [Box], [Vec], [HashMap], and [Option].
//! These implementations allow for the conversion of these types to and from [Value](crate::values::value::Value) and [ValueContainer](crate::values::value_container::ValueContainer).
mod r#box;
mod hash_map;
mod option;
mod r#vec;

// use crate::shared_values::PointerAddress;
// pub fn rust_none_marker() -> PointerAddress {
//     // TODO better addr
//     PointerAddress::self_owned([1u8, 2u8, 3u8, 4u8, 5u8])
// }
// pub fn rust_some_marker() -> PointerAddress {
//     // TODO better addr
//     PointerAddress::self_owned([1u8, 2u8, 3u8, 4u8, 6u8])
// }
