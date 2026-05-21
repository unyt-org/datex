mod type_match;

use core::ops::Deref;
use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeUnion(pub Vec<Type>);

impl Deref for TypeUnion {
    type Target = [Type];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Type> for TypeUnion {
    fn from_iter<I: IntoIterator<Item = Type>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}