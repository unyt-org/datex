//! This module defines the representation for data expressions in the AST.
use crate::{
    ast::expressions::{
        NormalExpressionData, Statements, traits::expression::Expression,
    },
    prelude::*,
    types::r#type::Type,
};
use core::ops::{self};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

#[derive(Clone, Debug)]
/// Represents a normal (non-type) expression in the AST
pub struct NormalExpression {
    /// The data contained in the expression.
    pub data: Box<NormalExpressionData>,
    /// The span of the expression in the source code.
    pub span: ops::Range<usize>,
    /// The type of the expression (assigned by type inference).
    pub ty: Option<Type>,
}

impl Default for NormalExpression {
    fn default() -> Self {
        NormalExpression {
            data: Box::new(NormalExpressionData::Statements(Statements {
                statements: Vec::new(),
                is_terminated: false,
                unbounded: None,
            })),
            span: 0..0,
            ty: None,
        }
    }
}

impl Expression<NormalExpressionData> for NormalExpression {
    fn new(data: NormalExpressionData, span: ops::Range<usize>) -> Self {
        NormalExpression {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    fn data(&self) -> &NormalExpressionData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut NormalExpressionData {
        &mut self.data
    }

    fn span(&self) -> &ops::Range<usize> {
        &self.span
    }

    fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

impl PartialEq for NormalExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for NormalExpression {}
