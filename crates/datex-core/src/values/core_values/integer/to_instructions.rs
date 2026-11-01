use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::integer::Integer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Integer {
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(core::iter::once(
            RegularInstruction::integer(self.clone()).into(),
        ))
    }
}
