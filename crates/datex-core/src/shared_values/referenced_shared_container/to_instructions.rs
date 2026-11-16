use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{ReferencedSharedContainer, SharedContainer},
};

impl ToInstructions for ReferencedSharedContainer {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        let reference = SharedContainer::Referenced(self.clone());
        let instructions: Vec<_> =
            reference.to_instructions(visitor, cache).collect(); // FIXME: no collect
        Box::new(instructions.into_iter())
    }
}
