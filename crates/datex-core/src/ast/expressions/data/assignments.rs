use crate::{
    ast::{expressions::DatexExpression, resolved_variable::VariableId},
    global::{operators::ModificationOperator, stack_index::StackIndex},
    prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UnboxAssignment {
    pub operator: Option<ModificationOperator>,
    pub unbox_expression: DatexExpression,
    pub assigned_expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnboxSlotAssignment {
    pub operator: Option<ModificationOperator>,
    pub stack_index: StackIndex,
    pub assigned_expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAssignment {
    pub operator: Option<ModificationOperator>,
    pub base: DatexExpression,
    pub property: DatexExpression,
    pub assigned_expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableAssignment {
    pub id: Option<VariableId>,
    pub name: String,
    pub operator: Option<ModificationOperator>,
    pub expression: DatexExpression,
}
