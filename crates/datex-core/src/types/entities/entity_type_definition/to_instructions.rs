use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    preludes::derive::{EntityTypeDefinition, RegularInstruction},
    values::{
        core_value::CoreValue,
        value::{Value, value_classification::ValueClassification},
    },
};

impl ToInstructions for EntityTypeDefinition {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            todo!("Implement ToInstructions for EntityTypeDefinition")
        })
    }
}
