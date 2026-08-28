use crate::{prelude::*, types::r#type::Type};
use core::fmt::Display;
pub mod equality;
mod serde_dif;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeTypeDefinition {
    pub start: Box<Type>,
    pub end: Box<Type>,
    // TODO inclusive / exclusive
}
impl RangeTypeDefinition {
    pub fn new(start: Type, end: Type) -> Self {
        Self {
            start: Box::new(start),
            end: Box::new(end),
        }
    }
}

impl Display for RangeTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
