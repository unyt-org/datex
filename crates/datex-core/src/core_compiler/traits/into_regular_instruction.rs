//! Helper trait to implement [into_regular_instruction] if a type can (without context) be converted
//! to a [RegularInstruction] directly.

use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

pub trait ToRegularInstruction {
    fn to_regular_instruction(&self) -> RegularInstruction;
}
impl<V> ToInstructions for V
where
    V: ToRegularInstruction,
{
    fn to_instructions(
        &self,
        _visitor: &mut dyn ValueVisitor,
        _cache: &mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction>> {
        Box::new(core::iter::once(self.to_regular_instruction().into()))
    }
}

// Tells the compiler that Box<T> does not implement ToRegularInstruction, even if T does.
// This allows implementing impl<V> ToInstructions for Box<V> without conflicting.
impl<T: ?Sized> !ToRegularInstruction for Box<T> {}
