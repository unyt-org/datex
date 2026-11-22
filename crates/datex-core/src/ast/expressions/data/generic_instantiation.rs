use crate::{
    ast::{expressions::DatexExpression, type_expressions::TypeExpression},
    prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct GenericInstantiation {
    pub base: DatexExpression,
    pub generic_arguments: Vec<TypeExpression>,
}
