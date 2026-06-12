use crate::{
    dxb_parser::body::DXBParserError,
    global::protocol_structures::{
        instruction_data::{
            ImplTypeData, ListData, MapData, ShortTextData, TextData,
            TypeReferenceData,
        },
        instructions::NextExpectedInstructions,
    },
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    shared_values::PointerAddress,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        type_definition_with_metadata::TypeMetadata,
    },
    values::core_values::{boolean::Boolean, integer::Integer},
};
use binrw::{
    BinRead, BinResult, BinWrite,
    io::{Read, Seek},
};
use core::fmt::{Display, Write as FmtWrite};
use serde::{Serialize, Serializer, ser::SerializeTuple};
use strum::AsRefStr;

#[derive(Clone, Debug, PartialEq, BinWrite, BinRead, AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[brw(little)]
pub enum TypeInstruction {
    #[brw(magic = 0x0u8)]
    TypeDefinitionCoreType(CoreLibTypeId),
    #[brw(magic = 0x1u8)]
    TypeDefinitionImplType(ImplTypeData),
    #[brw(magic = 0x2u8)]
    TypeDefinitionSharedTypeReference(TypeReferenceData),
    #[brw(magic = 0x3u8)]
    TypeDefinitionList(ListData),
    #[brw(magic = 0x4u8)]
    TypeDefinitionLiteral(LiteralTypeDefinition),
    #[brw(magic = 0x5u8)]
    TypeDefinitionRange,
    #[brw(magic = 0x6u8)]
    TypeDefinitionWithMetadata(TypeMetadata),

    #[brw(magic = 0x8u8)]
    TypeDefinitionMap(MapData),
}

/// Serializes TypeInstruction to tuple (instruction code as string, optional metadata as string)
impl Serialize for TypeInstruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let instruction_code = self.as_ref().to_string();
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
            TypeInstruction::TypeDefinitionList(list) => {
                NextExpectedInstructions::Type(list.element_count)
            } // list elements

            TypeInstruction::TypeDefinitionImplType(_) => {
                NextExpectedInstructions::Type(1)
            } // impl type

            TypeInstruction::TypeDefinitionWithMetadata(_) => {
                NextExpectedInstructions::Type(1)
            } // metadata type instruction

            TypeInstruction::TypeDefinitionRange => {
                NextExpectedInstructions::Type(2)
            } // range has 2 type instructions

            _ => NextExpectedInstructions::None,
        }
    }

    pub fn metadata_string(&self) -> Option<String> {
        let mut string = String::new();

        match self {
            TypeInstruction::TypeDefinitionLiteral(data) => {
                write!(string, "{}", data)
            }
            TypeInstruction::TypeDefinitionList(data) => {
                write!(string, "{}", data.element_count)
            }
            TypeInstruction::TypeDefinitionSharedTypeReference(
                reference_data,
            ) => {
                write!(
                    string,
                    "(address: {})",
                    PointerAddress::from(reference_data.address.clone())
                )
            }
            TypeInstruction::TypeDefinitionImplType(data) => {
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

impl Display for TypeInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = self.as_ref().to_string();
        write!(f, "{} ", code)?;

        if let Some(metadata_string) = self.metadata_string() {
            write!(f, " {}", metadata_string)?;
        }

        Ok(())
    }
}
