use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    types::r#type::Type,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for Type
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
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
