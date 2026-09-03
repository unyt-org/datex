pub mod serde_dif;

use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
use crate::shared_values::PointerAddress;
use crate::types::entity_type::EntityType;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ValueTag {
    pub tag: String,
    /// if set to true, the inner value is expected to be null but treated as non-existing (#Example instead of #Example(null))
    pub is_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, AsRefStr)]
pub enum ValueClassification {
    #[default]
    None,
    /// a nominal (entity) type associated with the value (e.g. Example {...})
    Entity(EntityType),
    /// list of impls that are associated with the value (e.g. null + $1234 + $5678)
    Impls(Vec<PointerAddress>),
    /// a tagged value, e.g. #Tagged(42)
    Tag(ValueTag)
}

impl ValueClassification {
    pub fn is_none(&self) -> bool {
        matches!(self, ValueClassification::None)
    }
}


impl From<EntityType> for ValueClassification {
    fn from(entity_type: EntityType) -> Self {
        ValueClassification::Entity(entity_type)
    }
}

impl From<Option<EntityType>> for ValueClassification {
    fn from(entity_type: Option<EntityType>) -> Self {
        match entity_type {
            Some(entity_type) => ValueClassification::Entity(entity_type),
            None => ValueClassification::None,
        }
    }
}

impl From<Vec<PointerAddress>> for ValueClassification {
    fn from(impls: Vec<PointerAddress>) -> Self {
        ValueClassification::Impls(impls)
    }
}

impl From<String> for ValueClassification {
    fn from(tag: String) -> Self {
        ValueClassification::Tag(ValueTag { tag, is_empty: false })
    }
}