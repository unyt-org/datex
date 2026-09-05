use crate::{
    core_compiler::into_regular_instruction::IntoRegularInstruction,
    instruction::regular_instruction::RegularInstruction,
};

impl IntoRegularInstruction for u8 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint8(*self)
    }
}

impl IntoRegularInstruction for u16 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint16(*self)
    }
}
impl IntoRegularInstruction for u32 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint32(*self)
    }
}
impl IntoRegularInstruction for u64 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint64(*self)
    }
}
impl IntoRegularInstruction for u128 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint128(*self)
    }
}

impl IntoRegularInstruction for i8 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int8(*self)
    }
}

impl IntoRegularInstruction for i16 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int16(*self)
    }
}

impl IntoRegularInstruction for i32 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int32(*self)
    }
}

impl IntoRegularInstruction for i64 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int64(*self)
    }
}
impl IntoRegularInstruction for i128 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int128(*self)
    }
}

#[cfg(target_pointer_width = "32")]
impl IntoRegularInstruction for isize {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int32(*self as i32)
    }
}

#[cfg(target_pointer_width = "32")]
impl IntoRegularInstruction for usize {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint32(*self as u32)
    }
}

#[cfg(target_pointer_width = "64")]
impl IntoRegularInstruction for isize {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::int64(*self as i64)
    }
}

#[cfg(target_pointer_width = "64")]
impl IntoRegularInstruction for usize {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::uint64(*self as u64)
    }
}
