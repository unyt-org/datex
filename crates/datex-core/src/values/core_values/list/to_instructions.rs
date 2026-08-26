use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::list::List,
};

impl<'ctx, T> ToInstructions<'ctx, T> for List
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(gen move {
            yield RegularInstruction::list(self.items.len() as u32);
            for item in &self.items {
                todo!("Implement instruction generation for value container");
                // for instruction in item.to_instructions(ctx) {
                //     yield instruction;
                // }
            }
        })
    }
}
