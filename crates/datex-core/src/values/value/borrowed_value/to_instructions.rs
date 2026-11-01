use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::value::borrowed_value::BorrowedValue,
};
use itertools::Itertools;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for BorrowedValue<'_> {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        match self {
            BorrowedValue::Native(native) => native.to_instructions(visitor, cache),
            BorrowedValue::Core(core) => {
                let core = core.clone_to_core_value_with_classification();
                // FIXME: get this to work without collect
                Box::new(
                    core.to_instructions(visitor, cache).collect_vec().into_iter(),
                )
            }
        }
    }
}
