use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::text::Text,
};

impl ToInstructions for Text {
    fn to_instructions(
        &self,
        visitor: &mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction>> {
        // FIXME: more efficient way to do this without collecting?
        let instructions = self
            .as_string()
            .to_instructions(visitor, cache)
            .collect::<Vec<_>>();
        Box::new(instructions.into_iter())
    }
}
