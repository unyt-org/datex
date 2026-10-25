use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    shared_values::{ReferencedSharedContainer, SharedContainer},
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for ReferencedSharedContainer {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        let reference = SharedContainer::Referenced(self.clone());
        let instructions: Vec<_> = reference.to_bytecode_items(visitor, cache).collect(); // FIXME: no collect
        Box::new(instructions.into_iter())
    }
}
