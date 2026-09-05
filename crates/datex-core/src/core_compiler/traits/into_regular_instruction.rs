//! Helper trait to implement [into_regular_instruction] if a type can (without context) be converted
//! to a [RegularInstruction] directly.
use crate::{
    core_compiler::{
        traits::to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
};

pub trait IntoRegularInstruction {
    fn into_regular_instruction(&self) -> RegularInstruction;
}

impl<'ctx, T, V> ToInstructions<'ctx, T> for V
where
    T: ValueVisitor<'ctx> + ?Sized,
    V: IntoRegularInstruction,
{
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        core::iter::once(self.into_regular_instruction().into())
    }
}
