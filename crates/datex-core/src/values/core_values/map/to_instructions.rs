use crate::{
    alloc::string::ToString,
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::map::{BorrowedMapKey, Map},
};
impl ToInstructions for Map {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            yield RegularInstruction::map(self.size() as u32).into();

            for (key, value) in self.iter() {
                let key_instructions: Vec<Instruction> =
                    key.to_instructions(ctx).collect();
                for instr in key_instructions {
                    yield instr;
                }
                for instr in value.to_instructions(ctx) {
                    yield instr;
                }
            }
        })
    }
}

impl<'b> ToInstructions for BorrowedMapKey<'b> {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            match *self {
                BorrowedMapKey::Text(text) => {
                    if text.len() < 256 {
                        yield RegularInstruction::key_value_short_text(
                            text.to_string(),
                        )
                        .into();
                    } else {
                        yield RegularInstruction::key_value_dynamic().into();
                        yield RegularInstruction::text(text.to_string()).into();
                    }
                }
                BorrowedMapKey::Value(val) => {
                    for instr in val.to_instructions(ctx) {
                        yield instr;
                    }
                }
            }
        })
    }
}
