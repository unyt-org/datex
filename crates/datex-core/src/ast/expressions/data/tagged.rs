use core::fmt::Display;

use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct TagExpression {
    pub tag: String,
    pub expression: Option<DatexExpression>,
}

impl Display for TagExpression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "#{}", self.tag)
    }
}
