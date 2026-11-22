//! This module defines the AST representation for expressions including their underlying data, span, and type information.
pub use crate::prelude::*;
use crate::types::r#type::Type;
use core::{
    fmt::Display,
    ops::{self},
};

// Reexport all data modules for expressions
mod data;
pub use data::*;

#[derive(Clone, Debug)]
/// An expression in the AST
pub struct DatexExpression {
    pub data: Box<DatexExpressionData>,
    pub span: ops::Range<usize>,
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
impl DatexExpression {
    pub fn new(data: DatexExpressionData, span: ops::Range<usize>) -> Self {
        DatexExpression {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    pub fn data(&self) -> &DatexExpressionData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut DatexExpressionData {
        &mut self.data
    }

    pub fn span(&self) -> &ops::Range<usize> {
        &self.span
    }

    pub fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

// PartialEquality for DatexExpression ignores the span (allows for easier testing)
impl PartialEq for DatexExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for DatexExpression {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableKind {
    Const,
    Var,
}

impl Display for VariableKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VariableKind::Const => core::write!(f, "const"),
            VariableKind::Var => core::write!(f, "var"),
        }
    }
}
