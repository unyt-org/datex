use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::ToInstructions,
    },
    global::protocol_structures::type_instructions::TypeInstruction,
    prelude::*,
    types::r#type::Type,
};
impl<'a> ToInstructions<'a> for Type {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            match self {
                Type::Entity(_) => unreachable!(),
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
