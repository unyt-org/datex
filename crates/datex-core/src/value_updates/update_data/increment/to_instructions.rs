use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::IncrementUpdateData,
};

use crate::prelude::*;
impl ToInstructions for IncrementUpdateData {
    fn to_instructions<'a>(
        &'a self,
        _visitor: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        todo!()
    }
}
