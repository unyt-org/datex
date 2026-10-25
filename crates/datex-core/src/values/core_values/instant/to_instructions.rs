use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::instant::Instant,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Instant {
    fn to_bytecode_items<'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(core::iter::once(RegularInstruction::instant(self.0).into()))
    }
}
