use crate::{
    core_compiler::{
        shared_value_tracking::{self, SharedValueTracking},
        to_instructions::ToInstructions,
    },
    global::protocol_structures::type_instructions::TypeInstruction,
    types::r#type::Type,
};
use crate::prelude::*;
impl ToInstructions for Type {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<dyn Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            match self {
                Type::Nominal(_) => unreachable!(),
                Type::Alias(def) => {
                    for instruction in
                        def.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
            }
        })
    }
}
