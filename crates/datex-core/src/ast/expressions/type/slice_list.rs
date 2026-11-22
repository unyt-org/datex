//! This module defines the representation for slice list type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};
/// Represents a slice list type expression in the AST, including the element type.
#[derive(Clone, Debug, PartialEq)]
pub struct SliceList(pub Box<TypeExpression>);
