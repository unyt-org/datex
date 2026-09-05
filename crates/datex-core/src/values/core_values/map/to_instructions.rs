use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    values::core_values::map::{BorrowedMapKey, Map},
};
impl<'ctx, T> ToInstructions<'ctx, T> for Map
where
    T: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        gen move {
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
        }
    }
}

impl<'ctx, 'b, T> ToInstructions<'ctx, T> for BorrowedMapKey<'b>
where
    T: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
        'b: 'a,
    {
        gen move {
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
        }
    }
}
