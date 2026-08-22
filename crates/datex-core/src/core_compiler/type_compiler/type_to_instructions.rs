use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    types::r#type::Type,
};
impl ToInstructions for Type {
    type InstructionType = TypeInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            match self {
                Type::Entity(_) => unreachable!(),
                Type::Definition(def) => {
                    for instruction in def.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }
        })
    }
}
