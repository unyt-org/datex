use crate::{
    ast::{
        expressions::{DatexExpression, VariableKind},
        resolved_variable::VariableId,
        type_expressions::TypeExpression,
    },
    prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    pub id: Option<VariableId>,
    pub kind: VariableKind,
    pub name: String,
    pub type_annotation: Option<TypeExpression>,
    pub init_expression: DatexExpression,
}
