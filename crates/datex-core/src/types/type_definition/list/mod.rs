use core::ops::Deref;
pub mod serde_dif;
use crate::{prelude::*, types::r#type::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTypeDefinition(pub Vec<Type>);

impl ListTypeDefinition {
    pub fn new(items: Vec<Type>) -> Self {
        Self(items)
    }
}

impl Deref for ListTypeDefinition {
    type Target = Vec<Type>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Type> for ListTypeDefinition {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        ListTypeDefinition(iter.into_iter().collect())
    }
}
