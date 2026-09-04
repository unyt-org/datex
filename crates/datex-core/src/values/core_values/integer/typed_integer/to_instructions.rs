use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::integer::typed_integer::TypedInteger,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for TypedInteger
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
        Box::new(gen move {
            yield match self {
                TypedInteger::I8(val) => RegularInstruction::int8(*val).into(),
                TypedInteger::I16(val) => RegularInstruction::int16(*val).into(),
                TypedInteger::I32(val) => RegularInstruction::int32(*val).into(),
                TypedInteger::I64(val) => RegularInstruction::int64(*val).into(),
                TypedInteger::I128(val) => RegularInstruction::int128(*val).into(),
                TypedInteger::U8(val) => RegularInstruction::uint8(*val).into(),
                TypedInteger::U16(val) => RegularInstruction::uint16(*val).into(),
                TypedInteger::U32(val) => RegularInstruction::uint32(*val).into(),
                TypedInteger::U64(val) => RegularInstruction::uint64(*val).into(),
                TypedInteger::U128(val) => RegularInstruction::uint128(*val).into(),
                TypedInteger::IBig(val) => {
                    RegularInstruction::big_integer(val.clone()).into()
                }
            };
        })
    }
}
