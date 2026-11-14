use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking, value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
        traits::SharedContainerCommon,
    },
};

impl<'ctx> ValueVisitor for SharedValueTracking<'ctx> {
    /// Registers a visited shared value in the shared value tracking
    fn visit_shared_container<'a>(
        &'a mut self,
        shared_container: &'a SharedContainer,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        let ownership = shared_container.ownership();
        let index = self.register_shared_value(shared_container);

        Box::new(gen move {
            yield match ownership {
                SharedContainerOwnership::Owned => {
                    RegularInstruction::take_stack_value(index)
                }
                SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                ) => RegularInstruction::get_stack_value_shared_ref(index),
                SharedContainerOwnership::Referenced(
                    ReferenceMutability::Mutable,
                ) => RegularInstruction::get_stack_value_shared_ref_mut(index),
            }
            .into()
        })
    }
}
