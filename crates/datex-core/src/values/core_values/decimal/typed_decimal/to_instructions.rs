use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl ToInstructions for TypedDecimal {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
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
