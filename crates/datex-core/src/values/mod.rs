//! This module contains the implementation of all levels of value representations, including
//! [core_value::CoreValue] as the most basic value types, [value::Value] as the main value representation in the type system, and [value_container] as the container for values that can be shared across different parts of the system.
pub mod borrowed_value_container;
mod cast;
pub mod core_value;
pub mod core_values;
pub mod value;
pub mod value_container;
