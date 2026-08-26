use core::cell::RefCell;

use crate::{
    compiler::context::CompilationContext,
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    types::r#type::Type,
};
impl<'ctx, T> ToInstructions<'ctx, T> for Type
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
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
