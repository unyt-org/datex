use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::endpoint::Endpoint,
};

impl ToInstructions for Endpoint {
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(core::iter::once(
            RegularInstruction::endpoint(self.clone()).into(),
        ))
    }
}
