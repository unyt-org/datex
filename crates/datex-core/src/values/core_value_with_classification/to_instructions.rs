use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{
        core_value_with_classification::CoreValueWithClassification,
        value::value_classification::ValueTag,
    },
};

impl ToInstructions for CoreValueWithClassification {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            let classification = &self.classification;

            yield RegularInstruction::classified_value(&classification).into();
            if let Some(entity_type) = &classification.entity_type {
                for item in visitor.visit_shared_container(
                    entity_type.shared_container(),
                    cache,
                ) {
                    yield item;
                }
            }

            // in case of tag
            if let Some(tag) = &classification.tag
                && tag.is_empty
            {
                // skip processing if empty tag, as no value expepcted
                return;
            }
            yield_all!(self.inner.to_bytecode_items(visitor, cache));
        })
    }
}
