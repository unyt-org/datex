use crate::{
    dxb_parser::body::DXBParserError,
    global::{
        protocol_structures::{
            instruction_data::{
                ImplTypeData, IntegerData, ListData, ShortTextData, TextData,
                TypeReferenceData,
            },
            instructions::NextExpectedInstructions,
        },
        type_instruction_codes::TypeInstructionCode,
    },
    prelude::*,
    shared_values::pointer_address::PointerAddress,
    types::type_definition_with_metadata::TypeMetadata,
};
use binrw::{
    BinRead, BinResult, BinWrite, Endian,
    io::{Read, Seek},
    meta::{EndianKind, ReadEndian},
};
use core::fmt::{Display, Write as FmtWrite};
use serde::{Serialize, Serializer, ser::SerializeTuple};

#[derive(Clone, Debug, PartialEq, BinWrite)]
#[brw(little)]
pub enum TypeInstruction {
    ImplType(ImplTypeData),
    SharedTypeReference(TypeReferenceData),
    LiteralText(TextData),
    LiteralInteger(IntegerData),
    List(ListData),
    Range, // TODO #670: add more type instructions
}

impl From<&TypeInstruction> for TypeInstructionCode {
    fn from(instruction: &TypeInstruction) -> Self {
        match instruction {
            TypeInstruction::ImplType(_) => {
                TypeInstructionCode::TYPE_WITH_IMPLS
            }
            TypeInstruction::SharedTypeReference(_) => {
                TypeInstructionCode::SHARED_TYPE_REFERENCE
            }
            TypeInstruction::LiteralText(_) => {
                TypeInstructionCode::TYPE_LITERAL_TEXT
            }
            TypeInstruction::LiteralInteger(_) => {
                TypeInstructionCode::TYPE_LITERAL_INTEGER
            }
            TypeInstruction::List(_) => TypeInstructionCode::TYPE_LIST,
            TypeInstruction::Range => TypeInstructionCode::TYPE_RANGE,
        }
    }
}

/// Serializes TypeInstruction to tuple (instruction code as string, optional metadata as string)
impl Serialize for TypeInstruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let instruction_code = TypeInstructionCode::from(self).to_string();
        let metadata_string = self.metadata_string();

        if let Some(metadata_string) = metadata_string {
            let mut state = serializer.serialize_tuple(2)?;
            state.serialize_element(&instruction_code)?;
            state.serialize_element(&metadata_string)?;
            state.end()
        } else {
            serializer.serialize_str(&instruction_code)
        }
    }
}

impl TypeInstruction {
    /// Returns how many (if any) regular or type instructions are expected as child instructions for a given instructions
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        match self {
            TypeInstruction::List(list) => {
                NextExpectedInstructions::Type(list.element_count)
            } // list elements

            TypeInstruction::ImplType(_) => NextExpectedInstructions::Type(1), // impl type

            TypeInstruction::Range => NextExpectedInstructions::Type(2), // range has 2 type instructions

            _ => NextExpectedInstructions::None,
        }
    }

    /// Based on the instruction code, read the corresponding instruction data and construct the TypeInstruction variant
    fn read_instruction<R: Read + Seek>(
        reader: &mut R,
        instruction_code: TypeInstructionCode,
    ) -> BinResult<Self> {
        match instruction_code {
            TypeInstructionCode::TYPE_LIST => {
                ListData::read(reader).map(TypeInstruction::List)
            }
            TypeInstructionCode::TYPE_LITERAL_INTEGER => {
                IntegerData::read(reader).map(TypeInstruction::LiteralInteger)
            }
            TypeInstructionCode::TYPE_LITERAL_TEXT => {
                TextData::read(reader).map(TypeInstruction::LiteralText)
            }
            TypeInstructionCode::TYPE_LITERAL_SHORT_TEXT => {
                ShortTextData::read(reader)
                    .map(|data| TextData(data.0))
                    .map(TypeInstruction::LiteralText)
            }
            TypeInstructionCode::TYPE_WITH_IMPLS => {
                ImplTypeData::read(reader).map(TypeInstruction::ImplType)
            }
            TypeInstructionCode::SHARED_TYPE_REFERENCE => {
                TypeReferenceData::read(reader)
                    .map(TypeInstruction::SharedTypeReference)
            }
            TypeInstructionCode::TYPE_RANGE => Ok(TypeInstruction::Range),
        }
    }

    fn read_type_instruction_code<R: Read + Seek>(
        mut reader: &mut R,
    ) -> Result<TypeInstructionCode, DXBParserError> {
        let instruction_code = u8::read(&mut reader)
            .map_err(|_| DXBParserError::FailedToReadInstructionCode)?;

        TypeInstructionCode::try_from(instruction_code).map_err(|_| {
            DXBParserError::InvalidInstructionCode(instruction_code)
        })
    }

    pub fn metadata_string(&self) -> Option<String> {
        let mut string = String::new();

        match self {
            TypeInstruction::LiteralText(data) => {
                write!(string, "{}", data.0)
            }
            TypeInstruction::LiteralInteger(data) => {
                write!(string, "{}", data.0)
            }
            TypeInstruction::List(data) => {
                write!(string, "{}", data.element_count)
            }
            TypeInstruction::SharedTypeReference(reference_data) => {
                write!(
                    string,
                    "(mutability: {:?}, address: {})",
                    TypeMetadata::from(&reference_data.metadata),
                    PointerAddress::from(&reference_data.address)
                )
            }
            TypeInstruction::ImplType(data) => {
                write!(string, "({} impls)", data.impl_count)
            }
            _ => {
                // no custom disassembly
                return None;
            }
        }
        .unwrap();

        Some(string)
    }
}

impl BinRead for TypeInstruction {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let instruction_code = TypeInstruction::read_type_instruction_code(
            reader,
        )
        .map_err(|e| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: format!("Failed to read type instruction code: {:?}", e),
        })?;
        TypeInstruction::read_instruction(reader, instruction_code)
    }
}

impl ReadEndian for TypeInstruction {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl Display for TypeInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = TypeInstructionCode::from(self);
        write!(f, "{} ", code)?;

        if let Some(metadata_string) = self.metadata_string() {
            write!(f, " {}", metadata_string)?;
        }

        Ok(())
    }
}
