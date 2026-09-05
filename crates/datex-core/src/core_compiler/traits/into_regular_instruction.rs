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
impl<T: ?Sized> !IntoRegularInstruction for Box<T> {}
impl<'ctx, C, V> ToInstructions<'ctx, C> for V
where
    C: ValueVisitor<'ctx> + ?Sized,
    V: IntoRegularInstruction,
{
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut C,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        core::iter::once(self.into_regular_instruction().into())
    }
}
