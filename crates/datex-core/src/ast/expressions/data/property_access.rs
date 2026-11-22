//! This module defines the representation for property access expressions in the AST.
use crate::{ast::expressions::DataExpression, prelude::*};

/// Represents a property access expression, e.g. `myStruct.property`.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAccess {
    /// The base expression on which the property is accessed.
    pub base: DataExpression,
    /// The property being accessed.
    pub property: DataExpression,
}

/// Represents a root property access expression, e.g. `$.property`.
#[derive(Clone, Debug, PartialEq)]
pub struct RootPropertyAccess {
    /// The name of the property being accessed on the root.
    pub property_name: String,
}
