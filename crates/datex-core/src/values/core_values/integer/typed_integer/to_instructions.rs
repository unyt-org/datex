use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl ToInstructions for TypedInteger {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield match self {
                TypedInteger::I8(val) => RegularInstruction::int8(*val),
                TypedInteger::I16(val) => RegularInstruction::int16(*val),
                TypedInteger::I32(val) => RegularInstruction::int32(*val),
                TypedInteger::I64(val) => RegularInstruction::int64(*val),
                TypedInteger::I128(val) => RegularInstruction::int128(*val),
                TypedInteger::U8(val) => RegularInstruction::uint8(*val),
                TypedInteger::U16(val) => RegularInstruction::uint16(*val),
                TypedInteger::U32(val) => RegularInstruction::uint32(*val),
                TypedInteger::U64(val) => RegularInstruction::uint64(*val),
                TypedInteger::U128(val) => RegularInstruction::uint128(*val),
                TypedInteger::IBig(val) => {
                    RegularInstruction::big_integer(val.clone())
                }
            };
        })
    }
}
