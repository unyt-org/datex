use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::collection::type_definition::SenderTypeDefinition,
};
use core::ops::Deref;

impl ToInstructions for SenderTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Sender.into();
            yield_all!(self.0.deref().to_instructions(visitor, cache));
        })
    }
}
