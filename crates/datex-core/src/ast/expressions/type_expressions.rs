//! This module defines the representation for type expressions in the AST.
use alloc::{boxed::Box, string::String};

use crate::{
    ast::{
        expressions::{
            TypeExpressionData, VariableAccess, traits::expression::Expression,
        },
        resolved_variable::ResolvedVariable,
        spanned::Spanned,
    },
    prelude::*,
    values::core_values::{
        boolean::Boolean,
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        integer::{Integer, typed_integer::TypedInteger},
        text::Text,
    },
};

use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    shared_values::PointerAddress,
    types::{r#type::Type, type_definition::callable::CallableKind},
};
use core::ops;

#[derive(Clone, Debug)]
/// Represents a type expression in the AST, including its data, span, and optional type.
pub struct TypeExpression {
    /// The data contained in the type expression.
    pub data: Box<TypeExpressionData>,
    /// The span of the type expression in the source code.
    pub span: ops::Range<usize>,
    /// The type of the expression (assigned by type inference).
    pub ty: Option<Type>,
}

impl Expression<TypeExpressionData> for TypeExpression {
    fn new(data: TypeExpressionData, span: ops::Range<usize>) -> Self {
        TypeExpression {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    fn data(&self) -> &TypeExpressionData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut TypeExpressionData {
        &mut self.data
    }

    fn span(&self) -> &ops::Range<usize> {
        &self.span
    }

    fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

impl PartialEq for TypeExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
