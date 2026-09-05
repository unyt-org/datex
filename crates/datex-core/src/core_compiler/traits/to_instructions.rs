use core::cell::RefCell;

use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
};

pub struct InstructionContext<'tracking, 'ctx> {
    pub shared_value_tracking:
        Option<&'ctx RefCell<SharedValueTracking<'tracking>>>,
}

impl<'tracking, 'ctx> InstructionContext<'tracking, 'ctx> {
    pub fn empty() -> Self {
        Self {
            shared_value_tracking: None,
        }
    }
}

pub trait ToInstructions<'ctx, V>
where
    V: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut V,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a;
}

pub trait ToInstructionsDyn {
    fn to_instructions_dyn<'a, 'ctx>(
        &'a self,
        ctx: &'a mut (dyn ValueVisitor<'ctx> + 'ctx),
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a;
}
// collides with box
// impl<T> ToInstructionsDyn for T
// where
//     T: for<'ctx> ToInstructions<'ctx, dyn ValueVisitor<'ctx> + 'ctx> + ?Sized,
// {
//     default fn to_instructions_dyn<'a, 'ctx>(
//         &'a self,
//         ctx: &'a mut (dyn ValueVisitor<'ctx> + 'ctx),
//     ) -> Box<dyn Iterator<Item = Instruction> + 'a>
//     where
//         'ctx: 'a,
//     {
//         Box::new(self.to_instructions(ctx))
//     }
// }
