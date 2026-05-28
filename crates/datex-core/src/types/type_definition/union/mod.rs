mod type_match;

use crate::{prelude::*, types::r#type::Type};
use core::ops::Deref;
pub mod serde_dif;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionTypeDefinition(pub Vec<Type>);

impl Deref for UnionTypeDefinition {
    type Target = [Type];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Type> for UnionTypeDefinition {
    fn from_iter<I: IntoIterator<Item = Type>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
