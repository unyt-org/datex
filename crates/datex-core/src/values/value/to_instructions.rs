use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    preludes::derive::RegularInstruction,
    values::value::{
        Value,
        value_classification::{ValueClassification, ValueTag},
    },
};

impl ToInstructions for Value {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        let classification = self.unresolved_classification();
        Box::new(gen move {
            if let Some(entity_type_address) = &classification.entity_type_address {
                yield RegularInstruction::EntityValue(entity_type_address.clone()).into()
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

            // append inner instructions
            match self {
                Value::Native(native) => {
                    for instruction in native.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                Value::Core(core) => {
                    for instruction in core.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }
        })
    }
}
