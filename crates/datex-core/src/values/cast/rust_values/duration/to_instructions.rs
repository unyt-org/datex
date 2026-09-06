use core::time::Duration;

use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, ToInstructionsDyn},
        value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::Instant,
};

// impl IntoRegularInstruction for Instant {
//     fn into_regular_instruction(&self) -> RegularInstruction {
//         // TODO: Implement the conversion from Instant to RegularInstruction
//         unimplemented!()
//     }
// }

impl ToInstructions for Duration {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move { todo!() })
    }
}
// impl ToInstructionsDyn for Duration {
//     fn to_instructions_dyn<'a, 'ctx>(
//         &'a self,
//         ctx: &'a mut (dyn ValueVisitor<'ctx> + 'ctx),
//     ) -> Box<dyn Iterator<Item = Instruction> + 'a>
//     where
//         'ctx: 'a,
//     {
//         Box::new(self.to_instructions(ctx))
//     }
// }
