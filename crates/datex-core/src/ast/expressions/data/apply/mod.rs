//! This module defines the representation for [Apply] that represents the application of a function or callable expression to a set of arguments.
use crate::{ast::expressions::NormalExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents the application of a function or callable expression to a set of arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Apply {
    /// The function or callable expression being applied.
    pub base: NormalExpression,
    /// The arguments being passed to the function or callable expression.
    pub arguments: Vec<NormalExpression>,
}
