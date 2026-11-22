//! This module defines the representation for [TypeExpressionData] that can hold the various kinds of type expressions used in the AST.
use alloc::{boxed::Box, string::String};

use crate::{
    ast::{
        expressions::{TypeExpression, VariableAccess},
        spanned::Spanned,
    },
    values::core_values::{
        boolean::Boolean,
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        integer::{Integer, typed_integer::TypedInteger},
        text::Text,
    },
};
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    shared_values::PointerAddress,
};
use core::ops;

mod structural_list;
pub use structural_list::*;

mod identifier_with_pointer_address;
pub use identifier_with_pointer_address::*;

mod fixed_sized_list;
pub use fixed_sized_list::*;

mod slice_list;
pub use slice_list::*;

mod intersection;
pub use intersection::*;

mod union;
pub use union::*;

mod generic_access;
pub use generic_access::*;

mod structural_map;
pub use structural_map::*;

mod callable;
pub use callable::*;

mod type_variant_access;
pub use type_variant_access::*;

mod range;
pub use range::*;

/// The different kinds of type expressions in the AST
#[derive(Clone, Debug, PartialEq)]
pub enum TypeExpressionData {
    // used for error recovery
    Recover,

    // a variable name or generic type identifier, e.g. integer, string, User, MyType, T
    Identifier(String),

    // a type name uniquely identified by a pointer address, e.g. User$1235
    IdentifierWithPointerAddress(IdentifierWithPointerAddress),

    VariableAccess(VariableAccess),
    GetReference(PointerAddress),

    GetCoreLibType(CoreLibTypeId),

    // literals
    Integer(Integer),
    TypedInteger(TypedInteger),
    Decimal(Decimal),
    TypedDecimal(TypedDecimal),
    Boolean(Boolean),
    Text(Text),
    Endpoint(Endpoint),

    // [integer, text, endpoint]
    // size known to compile time, arbitrary types
    StructuralList(StructuralList),

    Range(RangeTypeExpr),

    // [text; 3], integer[10]
    // fixed size and known to compile time, only one type
    FixedSizeList(FixedSizeList),

    // text[], integer[]
    // size not known to compile time, only one type
    SliceList(SliceList),

    // text & "test"
    Intersection(Intersection),

    // text | integer
    Union(Union),

    // User<text, integer>
    GenericAccess(GenericAccess),

    // e.g. function (x: text) -> text yeets error
    Callable(CallableTypeExpression),

    // structurally typed map, e.g. { x: integer, y: text }
    StructuralMap(StructuralMap),

    // modifiers
    Ref(Box<TypeExpression>),
    RefMut(Box<TypeExpression>),
    Shared(Box<TypeExpression>),
    Mut(Box<TypeExpression>),

    VariantAccess(TypeVariantAccess),
}

impl TypeExpressionData {
    pub fn null() -> Self {
        TypeExpressionData::GetCoreLibType(CoreLibBaseTypeId::Null.into())
    }

    pub fn unit() -> Self {
        TypeExpressionData::GetCoreLibType(CoreLibBaseTypeId::Unit.into())
    }
}

impl Spanned for TypeExpressionData {
    type Output = TypeExpression;

    fn with_span<T: Into<ops::Range<usize>>>(self, span: T) -> Self::Output {
        TypeExpression {
            data: Box::new(self),
            span: span.into(),
            ty: None,
        }
    }

    fn with_default_span(self) -> Self::Output {
        TypeExpression {
            data: Box::new(self),
            span: 0..0,
            ty: None,
        }
    }
}

impl Spanned for Box<TypeExpressionData> {
    type Output = TypeExpression;

    fn with_span<T: Into<ops::Range<usize>>>(self, span: T) -> Self::Output {
        TypeExpression {
            data: self,
            span: span.into(),
            ty: None,
        }
    }

    fn with_default_span(self) -> Self::Output {
        TypeExpression {
            data: self,
            span: 0..0,
            ty: None,
        }
    }
}
