use strum::{AsRefStr, EnumString, VariantNames};
pub mod type_definition;
use crate::types::type_definition::{
    collection::type_definition::{
        ListCollectionTypeDefinition, ListSliceCollectionTypeDefinition,
        MapCollectionTypeDefinition, ReceiverTypeDefinition,
        SenderTypeDefinition,
    },
    range::RangeTypeDefinition,
};
use core::fmt::Display;
pub mod serde_dif;
mod to_instructions;
// TODO #377: Rename to Generic type definition?
#[derive(Debug, Clone, PartialEq, Hash, Eq, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum CollectionTypeDefinition {
    // e.g. [integer]
    List(ListCollectionTypeDefinition),

    // e.g. [integer; 5]
    ListSlice(ListSliceCollectionTypeDefinition),

    // e.g. {string: integer}
    Map(MapCollectionTypeDefinition),
    Range(RangeTypeDefinition),

    // Sender<T>
    Sender(SenderTypeDefinition),
    // Receiver<T>
    Receiver(ReceiverTypeDefinition),
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, AsRefStr, EnumString, VariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum CollectionTypeDefinitionKind {
    List,
    ListSlice,
    Map,
    Range,
    Sender,
    Receiver,
}

impl Display for CollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CollectionTypeDefinition::List(ty) => core::write!(f, "{}", ty),
            CollectionTypeDefinition::ListSlice(ty) => {
                core::write!(f, "{}", ty)
            }
            CollectionTypeDefinition::Map(map) => core::write!(f, "{}", map),
            CollectionTypeDefinition::Range(range) => {
                core::write!(f, "{}", range)
            }
            CollectionTypeDefinition::Sender(ty) => core::write!(f, "{}", ty),
            CollectionTypeDefinition::Receiver(ty) => core::write!(f, "{}", ty),
        }
    }
}
