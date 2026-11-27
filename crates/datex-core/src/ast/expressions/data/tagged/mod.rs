//! This module defines the representation for tagged expressions in the AST.
use core::fmt::Display;

use crate::{ast::expressions::NormalExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a tagged expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct Tagged {
    /// The tag associated with this expression.
    pub tag: String,
    /// The expression associated with this tag, if any.
    /// For tagged empty expressions, this will be `None`.
    pub expression: Option<NormalExpression>,
}

impl Display for Tagged {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "#{}", self.tag)
    }
}
