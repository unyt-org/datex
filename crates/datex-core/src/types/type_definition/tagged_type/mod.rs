use core::fmt::Display;

use crate::types::type_definition::TypeDefinition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaggedTypeDefinition {
    pub tag: String,
    pub ty: Option<Box<TypeDefinition>>,
}
impl Display for TaggedTypeDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "#{} {}", self.tag, ty)
        } else {
            write!(f, "#{}", self.tag)
        }
    }
}
