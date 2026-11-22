use crate::{
    ast::{expressions::ValueAccessType, resolved_variable::VariableId},
    prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct VariableAccess {
    pub id: VariableId,
    pub name: String,
    pub access_type: ValueAccessType,
}
