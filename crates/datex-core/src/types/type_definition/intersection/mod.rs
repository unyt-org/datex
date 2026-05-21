mod type_match;

use core::ops::Deref;
use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIntersection(pub Vec<Type>);

impl Deref for TypeIntersection {
    type Target = [Type];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Type> for TypeIntersection {
    fn from_iter<I: IntoIterator<Item = Type>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}