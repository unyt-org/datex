use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::value::{Value, value_classification::ValueClassification},
};

impl ToInstructions for Value {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            match &self.classification {
                ValueClassification::None => {
                    for instruction in self.inner.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                ValueClassification::Entity(entity_type) => todo!(),
                ValueClassification::Impls(items) => todo!(),
                ValueClassification::Tag(value_tag) => todo!(),
            }
        })
    }
}
