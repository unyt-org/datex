use crate::core_compiler::shared_value_tracking::SharedValueTracking;
use crate::prelude::*;

pub trait ToInstructions {
    type InstructionType: Sized;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<dyn Iterator<Item = Self::InstructionType> + 'a>;
}
