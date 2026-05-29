use crate::{prelude::*, types::type_definition::TypeDefinition};
use core::fmt::Display;

mod metadata;
pub mod serde_dif;
pub use metadata::*;
pub mod type_match;
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeDefinitionWithMetadata {
    pub definition: TypeDefinition,
    pub metadata: TypeMetadata,
}

impl Display for TypeDefinitionWithMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let metadata_str = self.metadata.to_string();
        if !metadata_str.is_empty() {
            write!(f, "{} ", metadata_str)?;
        }
        write!(f, "{}", self.definition)
    }
}
