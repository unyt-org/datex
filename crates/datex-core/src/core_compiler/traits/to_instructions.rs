use core::cell::RefCell;

use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
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

pub trait ToInstructions {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a;
}
