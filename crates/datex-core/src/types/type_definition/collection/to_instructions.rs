use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::collection::CollectionTypeDefinition,
};

impl ToInstructions for CollectionTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        match self {
            CollectionTypeDefinition::List(list) => {
                list.to_instructions(visitor, cache)
            }
            CollectionTypeDefinition::Map(map) => {
                map.to_instructions(visitor, cache)
            }
            CollectionTypeDefinition::ListSlice(list) => {
                list.to_instructions(visitor, cache)
            }
            CollectionTypeDefinition::Range(range) => {
                range.to_instructions(visitor, cache)
            }
            CollectionTypeDefinition::Sender(sender) => {
                sender.to_instructions(visitor, cache)
            }
            CollectionTypeDefinition::Receiver(receiver) => {
                receiver.to_instructions(visitor, cache)
            }
        }
    }
}
