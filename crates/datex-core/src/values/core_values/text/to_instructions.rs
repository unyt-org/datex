use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::core_values::text::Text,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Text {
    fn to_bytecode_items(
        &self,
        visitor: &mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem>> {
        // FIXME: more efficient way to do this without collecting?
        let instructions =
            self.as_string().to_bytecode_items(visitor, cache).collect::<Vec<_>>();
        Box::new(instructions.into_iter())
    }
}
