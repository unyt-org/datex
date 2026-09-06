//! Helper trait to implement [into_regular_instruction] if a type can (without context) be converted
//! to a [RegularInstruction] directly.
use crate::{
    core_compiler::{
        to_instructions::ToInstructionsDyn,
        traits::to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
};

pub trait IntoRegularInstruction {
    fn into_regular_instruction(&self) -> RegularInstruction;
}
// impl<V> ToInstructions for V
// where
//     V: IntoRegularInstruction,
// {
//     fn to_instructions<'ctx, 'a>(
//         &'a self,
//         ctx: &'a mut dyn ValueVisitor<'ctx>,
//     ) -> Box<dyn Iterator<Item = Instruction> + 'a>
//     where
//         'ctx: 'a,
//     {
//         Box::new(core::iter::once(self.into_regular_instruction().into()))
//     }
// }

#[macro_export]
macro_rules! impl_regular_to_instructions {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::core_compiler::traits::to_instructions::ToInstructions for $ty {
                fn to_instructions<'ctx, 'a>(
                    &'a self,
                    _ctx: &'a mut dyn $crate::core_compiler::value_visitor::ValueVisitor<'ctx>,
                ) -> alloc::boxed::Box<dyn Iterator<Item = $crate::instruction::Instruction> + 'a>
                where
                    'ctx: 'a,
                {
                    alloc::boxed::Box::new(core::iter::once(
                        self.into_regular_instruction().into()
                    ))
                }
            }
        )*
    };
}
pub use impl_regular_to_instructions;
