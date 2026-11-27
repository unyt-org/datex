//! This module defines the representation for map expressions in the AST.
use crate::{ast::expressions::NormalExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a map expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    /// The entries in the map expression (key-value pairs).
    pub entries: Vec<(NormalExpression, NormalExpression)>,
}

impl FromIterator<(NormalExpression, NormalExpression)> for Map {
    fn from_iter<T: IntoIterator<Item = (NormalExpression, NormalExpression)>>(
        iter: T,
    ) -> Self {
        let entries = iter.into_iter().collect();
        Map { entries }
    }
}

impl Map {
    pub fn new(entries: Vec<(NormalExpression, NormalExpression)>) -> Self {
        Map { entries }
    }
}
