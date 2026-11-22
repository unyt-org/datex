//! This module defines the representation for data expressions in the AST.
use crate::{
    ast::expressions::{
        DataExpressionData, Statements, traits::expression::Expression,
    },
    prelude::*,
    types::r#type::Type,
};
use core::ops::{self};

#[derive(Clone, Debug)]
/// Represents a data expression in the AST
pub struct DataExpression {
    /// The data contained in the expression.
    pub data: Box<DataExpressionData>,
    /// The span of the expression in the source code.
    pub span: ops::Range<usize>,
    /// The type of the expression (assigned by type inference).
    pub ty: Option<Type>,
}

impl Default for DataExpression {
    fn default() -> Self {
        DataExpression {
            data: Box::new(DataExpressionData::Statements(Statements {
                statements: Vec::new(),
                is_terminated: false,
                unbounded: None,
            })),
            span: 0..0,
            ty: None,
        }
    }
}

impl Expression<DataExpressionData> for DataExpression {
    fn new(data: DataExpressionData, span: ops::Range<usize>) -> Self {
        DataExpression {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    fn data(&self) -> &DataExpressionData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut DataExpressionData {
        &mut self.data
    }

    fn span(&self) -> &ops::Range<usize> {
        &self.span
    }

    fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

impl PartialEq for DataExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for DataExpression {}
