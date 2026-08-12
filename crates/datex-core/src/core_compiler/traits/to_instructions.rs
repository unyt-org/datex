use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking, prelude::*,
};

pub trait ToInstructions {
    type InstructionType: Sized;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a>;
}
