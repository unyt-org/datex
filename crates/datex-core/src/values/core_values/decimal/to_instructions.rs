use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::decimal::Decimal,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Decimal
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(gen move {
            match &self {
                Decimal::Finite(big_decimal) if big_decimal.is_integer() => {
                    if let Some(int) = big_decimal.to_i16() {
                        yield RegularInstruction::decimal_as_int16(int);
                    } else if let Some(int) = big_decimal.to_i32() {
                        yield RegularInstruction::decimal_as_int32(int);
                    } else {
                        yield RegularInstruction::decimal(self.clone());
                    }
                }
                _ => {
                    yield RegularInstruction::decimal(self.clone());
                }
            }
        })
    }
}
