//! The [ToBytes] implementations for converting instructions to bytes
use crate::{
    global::protocol_structures::serializable::Serializable,
    instruction::Instruction, prelude::*,
};
use binrw::io::Write;

impl Serializable for Instruction {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Instruction::Regular(instr) => instr.to_bytes(),
            Instruction::Type(instr) => instr.to_bytes(),
        }
    }
}
impl Serializable for &Instruction {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Instruction::Regular(instr) => instr.to_bytes(),
            Instruction::Type(instr) => instr.to_bytes(),
        }
    }
}

impl Serializable for Vec<Instruction> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut cursor = binrw::io::Cursor::new(Vec::new());
        for instruction in self {
            cursor.write_all(&instruction.to_bytes()).unwrap();
        }
        cursor.into_inner()
    }
}
impl Serializable for &[Instruction] {
    fn to_bytes(&self) -> Vec<u8> {
        let mut cursor = binrw::io::Cursor::new(Vec::new());
        for instruction in *self {
            cursor.write_all(&instruction.to_bytes()).unwrap();
        }
        cursor.into_inner()
    }
}
