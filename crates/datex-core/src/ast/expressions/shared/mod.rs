//! Shared definitions for expressions in the Datex AST used in type and data space.
mod variable_kind;
pub use variable_kind::*;

mod resolved_variable;
pub use resolved_variable::*;

mod value_access_type;
pub use value_access_type::*;
