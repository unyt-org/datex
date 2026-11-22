use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteExecution {
    pub left: DatexExpression,
    pub right: DatexExpression,
    /// internal metadata set by precompiler, indicates how many variables from parent scope are accessed inside the remote execution
    pub injected_variable_count: Option<u32>,
}
