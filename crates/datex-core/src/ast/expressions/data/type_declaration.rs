use crate::{
    ast::{resolved_variable::VariableId, type_expressions::TypeExpression},
    prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TypeDeclarationExpression {
    pub id: Option<VariableId>,
    pub name: String, // TODO #614: separate variant from name
    pub definition: TypeExpression,
    pub hoisted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityDeclarationExpression {
    pub id: Option<VariableId>,
    pub name: String,
    pub definition: TypeExpression,
    pub hoisted: bool,
}
