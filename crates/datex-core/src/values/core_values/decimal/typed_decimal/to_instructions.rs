use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::regular_instruction::RegularInstruction,
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl<'ctx, T> ToInstructions<'ctx, T> for TypedDecimal
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(gen move {
            todo!(
                "TODO: append type cast with only id (no need to access shared container)"
            );
            // let id = CoreLibTypeId::from(self);
            yield match &self {
                TypedDecimal::F32(val) => {
                    RegularInstruction::decimal_f32(val.into_inner())
                }
                TypedDecimal::F64(val) => {
                    RegularInstruction::decimal_f64(val.into_inner())
                }
                TypedDecimal::Decimal(val) => {
                    RegularInstruction::decimal_big(val.clone())
                }
            }
        })
    }
}
