use core::cell::RefCell;

use crate::{
    compiler::context::CompilationContext,
    core_compiler::shared_value_tracking::SharedValueTracking,
    global::stack_index::StackIndex, prelude::*,
    shared_values::SharedContainer,
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

pub trait SharedValueTrackingProvider<'ctx> {
    fn shared_value_tracking<'a>(
        &'a self,
    ) -> Option<&'a RefCell<SharedValueTracking<'ctx>>>;
}
pub trait ToInstructions<'ctx, T>
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType: Sized;

    fn to_instructions<'a>(
        &self,
        ctx: &'a T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>>;
}
