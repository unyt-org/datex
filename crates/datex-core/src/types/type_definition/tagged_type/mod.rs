use crate::{prelude::*, types::type_definition::TypeDefinition};
use core::fmt::Display;
use crate::types::r#type::Type;

pub mod serde_dif;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaggedTypeDefinition {
    pub tag: String,
    pub ty: Option<Box<Type>>,
}
impl Display for TaggedTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "#{} {}", self.tag, ty)
        } else {
            write!(f, "#{}", self.tag)
        }
    }
}
