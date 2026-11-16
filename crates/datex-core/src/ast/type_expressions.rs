use alloc::{boxed::Box, string::String};

use crate::{
    ast::{
        expressions::VariableAccess, resolved_variable::ResolvedVariable,
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

#[derive(Clone, Debug, PartialEq)]
/// The different kinds of type expressions in the AST
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

#[derive(Clone, Debug)]
/// A type expression in the AST
pub struct TypeExpression {
    pub data: Box<TypeExpressionData>,
    pub span: ops::Range<usize>,
    pub ty: Option<Type>,
}
impl TypeExpression {
    pub fn new(data: TypeExpressionData, span: ops::Range<usize>) -> Self {
        Self {
            data: Box::new(data),
            span,
            ty: None,
        }
    }

    pub fn data(&self) -> &TypeExpressionData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut TypeExpressionData {
        &mut self.data
    }
}

impl PartialEq for TypeExpression {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructuralList(pub Vec<TypeExpression>);

#[derive(Clone, Debug, PartialEq)]
pub struct IdentifierWithPointerAddress {
    pub name: String,
    pub pointer_address: PointerAddress,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FixedSizeList {
    pub ty: Box<TypeExpression>,
    pub size: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliceList(pub Box<TypeExpression>);

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection(pub Vec<TypeExpression>);

#[derive(Clone, Debug, PartialEq)]
pub struct Union(pub Vec<TypeExpression>);

#[derive(Clone, Debug, PartialEq)]
pub struct GenericAccess {
    pub base: String,
    pub access: Vec<TypeExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructuralMap(pub Vec<(TypeExpression, TypeExpression)>);

#[derive(Clone, Debug, PartialEq)]
pub struct CallableTypeExpression {
    pub kind: CallableKind,
    pub parameter_types: Vec<(Option<String>, TypeExpression)>,
    pub rest_parameter_type: Option<(Option<String>, Box<TypeExpression>)>,
    pub return_type: Option<Box<TypeExpression>>,
    pub yeet_type: Option<Box<TypeExpression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeVariantAccess {
    pub name: String,
    pub variant: String,
    pub base: Option<ResolvedVariable>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RangeTypeExpr {
    pub start: Box<TypeExpression>,
    pub end: Box<TypeExpression>,
}
