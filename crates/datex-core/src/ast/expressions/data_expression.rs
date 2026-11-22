//! This module defines the representation for data expressions in the AST.
use crate::{
    ast::expressions::{
        DatexExpressionData, Statements, traits::expression::Expression,
    },
    prelude::*,
    types::r#type::Type,
};
use core::ops::{self};

#[derive(Clone, Debug)]
/// Represents a data expression in the AST
pub struct DatexExpression {
    /// The data contained in the expression.
    pub data: Box<DatexExpressionData>,
    /// The span of the expression in the source code.
    pub span: ops::Range<usize>,
    /// The type of the expression (assigned by type inference).
    pub ty: Option<Type>,
}

impl Default for DatexExpression {
    fn default() -> Self {
        DatexExpression {
            data: Box::new(DatexExpressionData::Statements(Statements {
                statements: Vec::new(),
                is_terminated: false,
                unbounded: None,
            })),
            span: 0..0,
            ty: None,
        }
    }
}

impl Expression<DatexExpressionData> for DatexExpression {
    fn new(data: DatexExpressionData, span: ops::Range<usize>) -> Self {
        DatexExpression {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    fn data(&self) -> &DatexExpressionData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut DatexExpressionData {
        &mut self.data
    }

    fn span(&self) -> &ops::Range<usize> {
        &self.span
    }

    fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

impl PartialEq for DatexExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for DatexExpression {}
