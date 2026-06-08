use crate::{prelude::*, types::type_definition::TypeDefinition};
use core::fmt::Display;
use core::hash::Hash;

mod metadata;
pub mod serde_dif;
pub use metadata::*;
pub mod type_match;
#[derive(Debug, Eq, Clone)]
pub struct TypeDefinitionWithMetadata {
    pub definition: TypeDefinition,
    pub metadata: TypeMetadata,

    pub reference_name: Option<String>,
}

impl PartialEq for TypeDefinitionWithMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.definition == other.definition && self.metadata == other.metadata
    }
}

impl Hash for TypeDefinitionWithMetadata {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.definition.hash(state);
        self.metadata.hash(state);
    }
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
