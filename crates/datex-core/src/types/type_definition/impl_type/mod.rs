use core::fmt::Display;

use crate::{shared_values::PointerAddress, types::r#type::Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplTypeDefinition {
    pub inner_type: Box<Type>,
    pub impl_markers: Vec<PointerAddress>,
}

impl ImplTypeDefinition {
    pub fn new(inner_type: Type, impl_markers: Vec<PointerAddress>) -> Self {
        Self {
            inner_type: Box::new(inner_type),
            impl_markers,
        }
    }
}

impl Display for ImplTypeDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner_type)?;
        for marker in &self.impl_markers {
            write!(f, " + {}", marker)?;
        }
        Ok(())
    }
}
