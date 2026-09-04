use core::cell::RefCell;

use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking, prelude::*,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

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

pub trait ToInstructions<'ctx, T>
where
    T: ValueVisitor<'ctx>,
{
    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>>;
}


pub trait ToInstructionsDyn {
    fn to_instructions_dyn(
        &self,
        ctx: &mut dyn ValueVisitor,
    ) -> Box<dyn Iterator<Item = Instruction>>;
}
