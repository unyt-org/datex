mod type_match;

use crate::{prelude::*, types::r#type::Type};
use core::ops::Deref;

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
