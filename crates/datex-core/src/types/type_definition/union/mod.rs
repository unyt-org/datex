mod type_match;

use crate::{prelude::*, types::r#type::Type};
use core::ops::Deref;

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
