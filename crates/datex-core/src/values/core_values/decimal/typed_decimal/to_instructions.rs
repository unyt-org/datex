use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for TypedDecimal
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        gen move {
            todo!(
                "TODO: append type cast with only id (no need to access shared container)"
            );
            // let id = CoreLibTypeId::from(self);
            yield match &self {
                TypedDecimal::F32(val) => {
                    RegularInstruction::decimal_f32(val.into_inner()).into()
                }
                TypedDecimal::F64(val) => {
                    RegularInstruction::decimal_f64(val.into_inner()).into()
                }
                TypedDecimal::Decimal(val) => {
                    RegularInstruction::decimal_big(val.clone()).into()
                }
            }
        }
    }
}
