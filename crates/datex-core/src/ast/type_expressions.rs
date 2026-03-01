use alloc::{boxed::Box, string::String};

use crate::{
    ast::{
        expressions::VariableAccess, resolved_variable::ResolvedVariable,
        spanned::Spanned,
    },
    prelude::*,
    values::core_values::{
        callable::CallableKind,
        decimal::{typed_decimal::TypedDecimal, Decimal},
        endpoint::Endpoint,
        integer::{typed_integer::TypedInteger, Integer},
        r#type::Type,
    },
};

use core::ops;
use crate::shared_values::pointer_address::PointerAddress;

#[derive(Clone, Debug, PartialEq)]
/// The different kinds of type expressions in the AST
pub enum TypeExpressionData {
    // used for error recovery
    Recover,

    Null,

    Unit,

    // a variable name or generic type identifier, e.g. integer, string, User, MyType, T
    Identifier(String),

    VariableAccess(VariableAccess),
    GetReference(PointerAddress),

    // literals
    Integer(Integer),
    TypedInteger(TypedInteger),
    Decimal(Decimal),
    TypedDecimal(TypedDecimal),
    Boolean(bool),
    Text(String),
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
    Shared(Box<TypeExpression>),

    VariantAccess(TypeVariantAccess),
}

impl Spanned for TypeExpressionData {
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
    pub data: TypeExpressionData,
    pub span: ops::Range<usize>,
    pub ty: Option<Type>,
}
impl TypeExpression {
    pub fn new(data: TypeExpressionData, span: ops::Range<usize>) -> Self {
        Self {
            data,
            span,
            ty: None,
        }
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
