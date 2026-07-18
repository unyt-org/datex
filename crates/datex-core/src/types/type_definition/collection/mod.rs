use strum::AsRefStr;
pub mod type_definition;
use crate::{
    global::operators::ModificationOperator,
    types::{
        traits::operator_handler::OperatorHandler,
        type_definition::{
            collection::type_definition::{
                list::ListCollectionTypeDefinition,
                list_slice::ListSliceCollectionTypeDefinition,
                map::MapCollectionTypeDefinition,
            },
            range::RangeTypeDefinition,
        },
    },
    value_updates::update_data::UpdateModificationOperator,
};
use core::fmt::Display;
pub mod serde_dif;
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
}

impl OperatorHandler for CollectionTypeDefinition {
    fn get_update_type_for_modification(
        &self,
        operator: ModificationOperator,
    ) -> Result<UpdateModificationOperator, ()> {
        match self {
            CollectionTypeDefinition::List(list_definition) => {
                list_definition.get_update_type_for_modification(operator)
            }
            _ => Err(()),
        }
    }
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
        }
    }
}
