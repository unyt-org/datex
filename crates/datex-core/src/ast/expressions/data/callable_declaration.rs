use crate::{
    ast::{expressions::DatexExpression, type_expressions::TypeExpression},
    prelude::*,
    types::type_definition::callable::CallableKind,
};

#[derive(Clone, Debug, PartialEq)]
pub struct CallableDeclaration {
    pub signature: CallableSignature,
    pub body: DatexExpression,
    pub injected_variable_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallableSignature {
    pub name: Option<String>,
    pub kind: CallableKind,
    pub requires_async: bool,
    pub parameters: Vec<(String, TypeExpression)>,
    pub rest_parameter: Option<(String, TypeExpression)>,
    pub return_type: Option<TypeExpression>,
    pub yeet_type: Option<TypeExpression>,
}
