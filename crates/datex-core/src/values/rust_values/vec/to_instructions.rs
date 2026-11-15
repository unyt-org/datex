use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::CompactListData,
        regular_instruction::RegularInstruction,
    },
    libs::core::{core_lib_id::CoreLibIdIndex, type_id::CoreLibVariantTypeId},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::{
        integer::typed_integer::IntegerTypeVariant, native::DatexNativeBase,
    },
};

impl<K> ToInstructions for Vec<K>
where
    K: DatexNativeBase,
{
    default fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::list(self.len() as u32).into();
            for item in self.iter() {
                for instruction in item.to_instructions(visitor, cache) {
                    yield instruction;
                }
            }
        })
    }
}

/// Optimized implementation of `ToInstructions` for `Vec<u8>`, which uses a compact list representation.
impl ToInstructions for Vec<u8> {
    fn to_instructions<'a>(
        &'a self,
        _visitor: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::CompactList(CompactListData {
                element_count: self.len() as u32,
                core_lib_type: CoreLibIdIndex::from(
                    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8),
                ),
                elements: self.clone(),
            })
            .into();
        })
    }
}
