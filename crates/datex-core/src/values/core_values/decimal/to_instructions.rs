use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::decimal::Decimal,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for Decimal
where
    T: ValueVisitor<'ctx> + ?Sized,
{

    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        gen move {
            match &self {
                Decimal::Finite(big_decimal) if big_decimal.is_integer() => {
                    if let Some(int) = big_decimal.to_i16() {
                        yield RegularInstruction::decimal_as_int16(int).into();
                    } else if let Some(int) = big_decimal.to_i32() {
                        yield RegularInstruction::decimal_as_int32(int).into();
                    } else {
                        yield RegularInstruction::decimal(self.clone()).into();
                    }
                }
                _ => {
                    yield RegularInstruction::decimal(self.clone()).into();
                }
            }
        }
    }
}
