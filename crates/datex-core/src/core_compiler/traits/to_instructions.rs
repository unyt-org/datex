use crate::core_compiler::shared_value_tracking::SharedValueTracking;
use crate::prelude::*;

pub trait ToInstructions<'a> {
    type InstructionType: Sized;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a>;
}
