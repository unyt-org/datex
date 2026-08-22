use core::cell::RefCell;

use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking, prelude::*,
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
    type InstructionType: Sized;

    fn to_instructions<'tracking, 'ctx>(
        &'ctx self,
        ctx: &'ctx InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'ctx>;
}
