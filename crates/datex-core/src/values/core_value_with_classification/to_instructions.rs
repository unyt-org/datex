use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
};
use crate::instruction::regular_instruction::RegularInstruction;
use crate::preludes::derive::ValueTag;
use crate::values::core_value_with_classification::CoreValueWithClassification;

impl ToInstructions for CoreValueWithClassification {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            let classification = &self.classification;
            
            if let Some(entity_type) = &classification.entity_type {
                yield RegularInstruction::EntityValue(entity_type.pointer_address()).into()
            }

            for impl_address in &classification.impls {
                todo!(
                    "Compiling values with Impls classification is not yet implemented"
                )
            }

            if let Some(ValueTag {tag, is_empty}) = &classification.tag {
                yield RegularInstruction::tagged_value(
                    tag.clone(),
                    *is_empty,
                )
                    .into();
                if *is_empty {
                    // early return, don't append null value; TODO: assert that value is actually null?
                    return;
                };
            }
            
            for instruction in self.inner.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}
