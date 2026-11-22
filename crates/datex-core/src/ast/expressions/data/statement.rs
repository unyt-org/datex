use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct UnboundedStatement {
    pub is_first: bool,
    pub is_last: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Statements {
    pub statements: Vec<DatexExpression>,
    pub is_terminated: bool,
    pub unbounded: Option<UnboundedStatement>,
}

impl Statements {
    pub fn empty() -> Self {
        Statements {
            statements: Vec::new(),
            is_terminated: true,
            unbounded: None,
        }
    }
    pub fn new_terminated(statements: Vec<DatexExpression>) -> Self {
        Statements {
            statements,
            is_terminated: true,
            unbounded: None,
        }
    }
    pub fn new_unterminated(statements: Vec<DatexExpression>) -> Self {
        Statements {
            statements,
            is_terminated: false,
            unbounded: None,
        }
    }
}
