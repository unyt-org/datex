use crate::{
    core_compiler::value_compiler::append_instruction,
    global::{
        protocol_structures::{
            injected_values::InjectedValueDeclaration,
            instructions::Instruction,
        },
    },
    prelude::*,
};
use binrw::{
    BinRead, BinResult, BinWrite, Endian,
    io::{Cursor, Seek, Write},
};
use core::{fmt::Display};
use itertools::Itertools;
use crate::disassembler::InstructionTree;
use crate::dxb_parser::body::InstructionWithSpan;
use crate::global::protocol_structures::instruction_data::{CallableDataBody, CallableSignatureData, InstructionBlockData};

#[derive(Clone, Debug, PartialEq)]
pub struct InstructionBlockDataDebugTree {
    pub length: u32,
    pub injected_variable_count: u32,
    pub injected_values: Vec<InjectedValueDeclaration>,
    pub body: InstructionTree<InstructionWithSpan>,
}

impl Display for InstructionBlockDataDebugTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[length: {}, injected_variable_count: {}, injected_values: [{}]]",
               self.length,
               self.injected_variable_count,
               self.injected_values.iter().map(|v| format!("{}", v)).join(", "),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InstructionBlockDataDebugFlat {
    pub length: u32,
    pub injected_variable_count: u32,
    pub injected_values: Vec<InjectedValueDeclaration>,
    pub body: Vec<InstructionWithSpan>,
}

impl Display for InstructionBlockDataDebugFlat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[length: {}, injected_variable_count: {}, injected_values: [{}]]",
               self.length,
               self.injected_variable_count,
               self.injected_values.iter().map(|v| format!("{}", v)).join(", "),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallableDataBodyDebugTree {
    pub injected_value_count: u32,
    pub length: u32, // if length is 0, the body has a native implementation
    pub body: InstructionTree<InstructionWithSpan>,
}

impl Display for CallableDataBodyDebugTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[length: {}, injected_value_count: {}]",
            self.length,
            self.injected_value_count,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CallableDataBodyDebugFlat {
    pub injected_value_count: u32,
    pub length: u32, // if length is 0, the body has a native implementation
    pub body: Vec<InstructionWithSpan>,
}

impl Display for CallableDataBodyDebugFlat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[length: {}, injected_value_count: {}]",
            self.length,
            self.injected_value_count,
        )
    }
}

#[derive(BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDeclarationDataDebugTree {
    pub signature: CallableSignatureData,
    pub body: InstructionBlockDataDebugTree,
}

#[derive(BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDeclarationDataDebugFlat {
    pub signature: CallableSignatureData,
    pub body: InstructionBlockDataDebugFlat,
}

#[derive(BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDataDebugTree {
    pub signature: CallableSignatureData,
    pub body: CallableDataBodyDebugTree,
}

#[derive(BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDataDebugFlat {
    pub signature: CallableSignatureData,
    pub body: CallableDataBodyDebugFlat,
}

impl From<&InstructionBlockDataDebugTree> for InstructionBlockDataDebugFlat {
    fn from(instruction_block_data: &InstructionBlockDataDebugTree) -> Self {
        InstructionBlockDataDebugFlat {
            length: instruction_block_data.length,
            injected_variable_count: instruction_block_data.injected_variable_count,
            injected_values: instruction_block_data.injected_values.clone(),
            body: instruction_block_data.body.flatten(),
        }
    }
}

impl From<&InstructionBlockDataDebugFlat> for InstructionBlockData {
    fn from(value: &InstructionBlockDataDebugFlat) -> Self {
        let mut cursor = Cursor::new(Vec::new());
        for instruction in &value.body {
            append_instruction(&mut cursor, instruction.instruction.clone());
        }
        Self {
            length: value.length,
            injected_value_count: value.injected_variable_count,
            injected_values: value.injected_values.clone(),
            body: cursor.into_inner(),
        }
    }
}

impl From<&CallableDataBodyDebugTree> for CallableDataBodyDebugFlat {
    fn from(value: &CallableDataBodyDebugTree) -> Self {
        CallableDataBodyDebugFlat {
            injected_value_count: value.injected_value_count,
            length: value.length,
            body: value.body.flatten(),
        }
    }
}

impl From<&CallableDataBodyDebugFlat> for CallableDataBody{
    fn from(value: &CallableDataBodyDebugFlat) -> Self {
        let mut cursor = Cursor::new(Vec::new());
        for instruction in &value.body {
            append_instruction(&mut cursor, instruction.instruction.clone());
        }
        CallableDataBody {
            injected_value_count: value.injected_value_count,
            length: value.length,
            body: cursor.into_inner(),
        }
    }
}

impl BinWrite for InstructionBlockDataDebugFlat {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = InstructionBlockData::from(self);
        raw.write_options(writer, endian, ())
    }
}

impl BinWrite for InstructionBlockDataDebugTree {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = InstructionBlockData::from(&InstructionBlockDataDebugFlat::from(self));
        raw.write_options(writer, endian, ())
    }
}

impl BinWrite for CallableDataBodyDebugFlat {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = CallableDataBody::from(self);
        raw.write_options(writer, endian, ())
    }
}

impl BinWrite for CallableDataBodyDebugTree {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = CallableDataBody::from(&CallableDataBodyDebugFlat::from(self));
        raw.write_options(writer, endian, ())
    }
}