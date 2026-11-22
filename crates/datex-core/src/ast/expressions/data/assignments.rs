//! This module defines the representation for assignment expressions in the AST.
use crate::{
    ast::{expressions::DatexExpression, resolved_variable::VariableId},
    global::{operators::ModificationOperator, stack_index::StackIndex},
    prelude::*,
};

/// Represents an assignment to an unbox expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnboxAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The unbox expression being assigned to.
    pub unbox_expression: DatexExpression,
    /// The expression being assigned.
    pub assigned_expression: DatexExpression,
}

/// Represents an assignment to an unbox slot.
#[derive(Clone, Debug, PartialEq)]
pub struct UnboxSlotAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The stack index of the unbox slot being assigned to.
    pub stack_index: StackIndex,
    /// The expression being assigned.
    pub assigned_expression: DatexExpression,
}

/// Represents an assignment to a property of an object.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The base object whose property is being assigned.
    pub base: DatexExpression,
    /// The property being assigned to.
    pub property: DatexExpression,
    /// The expression being assigned.
    pub assigned_expression: DatexExpression,
}

/// Represents an assignment to a variable.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableAssignment {
    /// The ID of the variable being assigned, if known.
    pub id: Option<VariableId>,
    /// The name of the variable being assigned.
    pub name: String,
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The expression being assigned to the variable.
    pub expression: DatexExpression,
}
