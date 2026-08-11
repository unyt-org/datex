use crate::{
    global::protocol_structures::{
        instruction_data::{
            ImplTypeData, ListData, MapData, TypeReferenceData,
        },
        instructions::NextExpectedInstructions,
    },
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        type_definition_with_metadata::TypeMetadata,
    },
};
use binrw::{BinRead, BinWrite};
use core::fmt::{Display, Write as FmtWrite};
use serde::{Serialize, Serializer, ser::SerializeTuple};
use strum::AsRefStr;

#[derive(Clone, Debug, PartialEq, BinWrite, BinRead, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[brw(little)]
pub enum TypeInstruction {
    #[brw(magic = 0x0u8)]
    CoreType(CoreLibTypeId),
    #[brw(magic = 0x1u8)]
    ImplType(ImplTypeData),
    #[brw(magic = 0x2u8)]
    SharedTypeReference(TypeReferenceData),
    #[brw(magic = 0x3u8)]
    List(ListData),
    #[brw(magic = 0x4u8)]
    Literal(LiteralTypeDefinition),
    #[brw(magic = 0x5u8)]
    Range,
    #[brw(magic = 0x6u8)]
    DefinitionWithMetadata(TypeMetadata),

    #[brw(magic = 0x8u8)]
    Map(MapData),
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
            TypeInstruction::List(list) => {
                NextExpectedInstructions::Type(list.element_count)
            } // list elements

            TypeInstruction::ImplType(_) => NextExpectedInstructions::Type(1), // impl type

            TypeInstruction::DefinitionWithMetadata(_) => {
                NextExpectedInstructions::Type(1)
            } // metadata type instruction

            TypeInstruction::Range => NextExpectedInstructions::Type(2), // range has 2 type instructions

            _ => NextExpectedInstructions::None,
        }
    }

    pub fn metadata_string(&self) -> Option<String> {
        let mut string = String::new();

        match self {
            TypeInstruction::Literal(data) => {
                write!(string, "{}", data)
            }
            TypeInstruction::List(data) => {
                write!(string, "{}", data.element_count)
            }
            TypeInstruction::SharedTypeReference(reference_data) => {
                write!(string, "[address: {}]", reference_data.address.clone())
            }
            TypeInstruction::CoreType(data) => {
                write!(string, "{}", data)
            }
            TypeInstruction::ImplType(data) => {
                write!(string, "[{} impls]", data.impl_count)
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
