//! This module defines the representation for map expressions in the AST.
use crate::{ast::expressions::DataExpression, prelude::*};

/// Represents a map expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    /// The entries in the map expression (key-value pairs).
    pub entries: Vec<(DataExpression, DataExpression)>,
}

impl FromIterator<(DataExpression, DataExpression)> for Map {
    fn from_iter<T: IntoIterator<Item = (DataExpression, DataExpression)>>(
        iter: T,
    ) -> Self {
        let entries = iter.into_iter().collect();
        Map { entries }
    }
}

impl Map {
    pub fn new(entries: Vec<(DataExpression, DataExpression)>) -> Self {
        Map { entries }
    }
}
