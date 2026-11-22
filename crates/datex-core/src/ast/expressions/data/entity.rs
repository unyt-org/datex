use crate::{
    ast::expressions::DatexExpression, prelude::*,
    shared_values::PointerAddress,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EntityValueExpression {
    pub entity_name: String,
    pub entity_address: Option<PointerAddress>,
    pub value: DatexExpression,
}
