//! The [ToBytes] implementations for converting instructions to bytes
use crate::{
    instruction::{
        Instruction, regular_instruction::RegularInstruction,
        traits::to_bytes::ToBytes, type_instruction::TypeInstruction,
    },
    prelude::*,
};
use binrw::{
    BinResult, BinWrite, Endian,
    io::{Seek, Write},
    meta::{EndianKind, WriteEndian},
};
use core::{fmt::Display, prelude::rust_2024::*};
use serde::Serialize;

use core::sync::atomic::{AtomicUsize, Ordering};

impl ToBytes for Instruction {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Instruction::Regular(instr) => instr.to_bytes(),
            Instruction::Type(instr) => instr.to_bytes(),
        }
    }
}
impl ToBytes for &Instruction {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Instruction::Regular(instr) => instr.to_bytes(),
            Instruction::Type(instr) => instr.to_bytes(),
        }
    }
}

impl ToBytes for Vec<Instruction> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut cursor = binrw::io::Cursor::new(Vec::new());
        for instruction in self {
            cursor.write_all(&instruction.to_bytes()).unwrap();
        }
        cursor.into_inner()
    }
}
impl ToBytes for &[Instruction] {
    fn to_bytes(&self) -> Vec<u8> {
        let mut cursor = binrw::io::Cursor::new(Vec::new());
        for instruction in *self {
            cursor.write_all(&instruction.to_bytes()).unwrap();
        }
        cursor.into_inner()
    }
}
