//! This module defines the representation for tagged expressions in the AST.
use core::fmt::Display;

use crate::{ast::expressions::DatexExpression, prelude::*};

/// Represents a tagged expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct TagExpression {
    /// The tag associated with this expression.
    pub tag: String,
    /// The expression associated with this tag, if any.
    /// For tagged empty expressions, this will be `None`.
    pub expression: Option<DatexExpression>,
}

impl Display for TagExpression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "#{}", self.tag)
    }
}
