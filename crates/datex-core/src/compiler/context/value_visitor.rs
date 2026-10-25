use core::cell::RefCell;
use crate::{
    compiler::context::CompilationContext,
    core_compiler::value_visitor::ValueVisitor,
    prelude::*, shared_values::SharedContainer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::utils::goat_mut::GoatMut;

impl<'ctx> ValueVisitor for CompilationContext<'ctx> {
    fn visit_shared_container<'a>(
        &'a mut self,
        shared_container: &'a SharedContainer,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        // Delegate the visit to the shared value tracking system
        self.core_context
            .shared_value_tracking
            .get_mut()
            .visit_shared_container(shared_container)
    }

    fn try_get_cache(&mut self) -> GoatMut<SharedReferencesCache> {
        GoatMut::map(GoatMut::Ref(self.core_context.shared_value_tracking.borrow_mut()), |tracking| {
            tracking.cache
        })
    }
}
