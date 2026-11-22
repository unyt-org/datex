use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAccess {
    pub base: DatexExpression,
    pub property: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RootPropertyAccess {
    pub property_name: String,
}
