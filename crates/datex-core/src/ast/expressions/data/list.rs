//! This module defines the representation for list expressions in the AST.
use crate::{ast::expressions::DatexExpression, prelude::*};

/// Represents a list expression.
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    /// The items in the list expression.
    pub items: Vec<DatexExpression>,
}

impl List {
    pub fn new(items: Vec<DatexExpression>) -> Self {
        List { items }
    }
}

impl FromIterator<DatexExpression> for List {
    fn from_iter<T: IntoIterator<Item = DatexExpression>>(iter: T) -> Self {
        let items = iter.into_iter().collect();
        List { items }
    }
}
