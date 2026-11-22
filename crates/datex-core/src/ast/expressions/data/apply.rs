//! This module defines the representation for [Apply] that represents the application of a function or callable expression to a set of arguments.
use crate::{ast::expressions::DatexExpression, prelude::*};

/// Represents the application of a function or callable expression to a set of arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Apply {
    /// The function or callable expression being applied.
    pub base: DatexExpression,
    /// The arguments being passed to the function or callable expression.
    pub arguments: Vec<DatexExpression>,
}
