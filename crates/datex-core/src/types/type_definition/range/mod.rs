use core::fmt::Display;

use crate::types::r#type::Type;
pub mod equality;
mod serde_dif;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeTypeDefinition {
    pub start: Box<Type>,
    pub end: Box<Type>,
    // TODO inclusive / exclusive
}

impl Display for RangeTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
