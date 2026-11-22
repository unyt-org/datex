//! This module defines the representation for map expressions in the AST.
use crate::{ast::expressions::DatexExpression, prelude::*};

/// Represents a map expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    /// The entries in the map expression (key-value pairs).
    pub entries: Vec<(DatexExpression, DatexExpression)>,
}

impl FromIterator<(DatexExpression, DatexExpression)> for Map {
    fn from_iter<T: IntoIterator<Item = (DatexExpression, DatexExpression)>>(
        iter: T,
    ) -> Self {
        let entries = iter.into_iter().collect();
        Map { entries }
    }
}

impl Map {
    pub fn new(entries: Vec<(DatexExpression, DatexExpression)>) -> Self {
        Map { entries }
    }
}
