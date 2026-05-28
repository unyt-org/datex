use core::ops::Deref;
pub mod equality;
mod serde_dif;

use crate::{prelude::*, types::r#type::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTypeDefinition(pub Vec<(Type, Type)>);

impl MapTypeDefinition {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Deref for MapTypeDefinition {
    type Target = Vec<(Type, Type)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<(Type, Type)> for MapTypeDefinition {
    fn from_iter<T: IntoIterator<Item = (Type, Type)>>(iter: T) -> Self {
        MapTypeDefinition(iter.into_iter().collect())
    }
}
