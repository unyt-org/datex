//! This module defines the abstract syntax tree (AST) for DATEX.
//! The AST represents a nested structure of [expressions::DatexExpression] in a way that can transformed by the compiler.

pub mod expressions;
pub mod resolved_variable;
pub mod spanned;
#[cfg(feature = "std")]
pub mod src_id;
pub mod type_expressions;
