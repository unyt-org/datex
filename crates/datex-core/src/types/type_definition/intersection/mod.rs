mod type_match;

use crate::{prelude::*, types::r#type::Type};
use core::ops::Deref;
pub mod equality;
pub mod serde_dif;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntersectionTypeDefinition(pub Vec<Type>);

impl Deref for IntersectionTypeDefinition {
    type Target = [Type];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Type> for IntersectionTypeDefinition {
    fn from_iter<I: IntoIterator<Item = Type>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
