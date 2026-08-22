use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::list::List,
};

impl ToInstructions for List {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
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
