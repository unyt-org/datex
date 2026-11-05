use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all, yield_all_collected},
        value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::map::Map,
};

impl ToInstructions for Map {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::map(
                self.size() as u32,
                self.allow_only_text_keys,
            )
            .into();

            for (key, value) in self.iter() {
                match key.try_as_str() {
                    // text -> insert key string
                    Some(string) => {
                        if string.len() < 256 {
                            yield RegularInstruction::key_value_short_text(
                                string.to_string(),
                            )
                            .into();
                        } else {
                            yield RegularInstruction::key_value_dynamic()
                                .into();
                            yield RegularInstruction::text(string.to_string())
                                .into();
                        }
                    }
                    // other -> insert key as dynamic
                    None => {
                        yield RegularInstruction::key_value_dynamic().into();
                        yield_all_collected!(
                            key.to_instructions(visitor, cache)
                        );
                    }
                }

                yield_all!(value.to_instructions(visitor, cache));
            }
        })
    }
}
