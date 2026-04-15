use alloc::string::FromUtf8Error;
use core::fmt::Display;
use core::ops::AddAssign;
use binrw::io::{Cursor, Read, Seek, Write};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use binrw::meta::{EndianKind, ReadEndian};
use cfg_if::cfg_if;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B4;
use serde::Serialize;
use crate::core_compiler::value_compiler::append_instruction;
use crate::global::operators::AssignmentOperator;
use crate::global::protocol_structures::injected_values::{InjectedValueDeclaration, InjectedValueType};
use crate::global::protocol_structures::instructions::Instruction;
use crate::global::type_instruction_codes::{TypeLocalOrShared, TypeMutabilityCode, TypeReferenceMutabilityCode};
use crate::serde::Deserialize;
use crate::shared_values::pointer_address::{SelfOwnedPointerAddress, PointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_containers::{ReferenceMutability, SharedContainerMutability};
use crate::values::core_values::decimal::Decimal;
use crate::values::core_values::endpoint::{Endpoint, EndpointParsingError};
use crate::values::core_values::integer::Integer;
use crate::prelude::*;
use crate::types::type_definition::TypeMetadata;

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Int8Data(pub i8);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Int16Data(pub i16);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Int32Data(pub i32);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Int64Data(pub i64);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Int128Data(pub i128);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UInt8Data(pub u8);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UInt16Data(pub u16);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UInt32Data(pub u32);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UInt64Data(pub u64);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UInt128Data(pub u128);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Float32Data(pub f32);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Float64Data(pub f64);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct FloatAsInt16Data(pub i16);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct FloatAsInt32Data(pub i32);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct DecimalData(pub Decimal);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct IntegerData(pub Integer);

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ShortTextDataRaw {
    pub length: u8,
    #[br(count = length)]
    pub text: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ShortTextData(pub String);

impl From<&ShortTextData> for ShortTextDataRaw {
    fn from(value: &ShortTextData) -> Self {
        let bytes = value.0.as_bytes();

        Self {
            length: bytes.len() as u8,
            text: bytes.to_vec(),
        }
    }
}

impl TryFrom<ShortTextDataRaw> for ShortTextData {
    type Error = FromUtf8Error;
    fn try_from(raw: ShortTextDataRaw) -> Result<Self, Self::Error> {
        let string = String::from_utf8(raw.text)?;
        Ok(ShortTextData(string))
    }
}

impl BinWrite for ShortTextData {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = ShortTextDataRaw::from(self);
        raw.write_options(writer, endian, ())
    }
}

impl BinRead for ShortTextData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let raw = ShortTextDataRaw::read_options(reader, endian, ())?;
        Ok(raw.try_into().map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: "Invalid UTF-8 string".to_string()
        })?)
    }
}

impl ReadEndian for ShortTextData {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}


#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct TextDataRaw {
    pub length: u32,
    #[br(count = length)]
    pub text: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextData(pub String);

impl From<&TextData> for TextDataRaw {
    fn from(value: &TextData) -> Self {
        let bytes = value.0.as_bytes();

        Self {
            length: bytes.len() as u32,
            text: bytes.to_vec(),
        }
    }
}

impl TryFrom<TextDataRaw> for TextData {
    type Error = FromUtf8Error;
    fn try_from(raw: TextDataRaw) -> Result<Self, Self::Error> {
        let string = String::from_utf8(raw.text)?;
        Ok(TextData(string))
    }
}

impl BinWrite for TextData {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        let raw = TextDataRaw::from(self);
        raw.write_options(writer, endian, ())
    }
}

impl BinRead for TextData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let raw = TextDataRaw::read_options(reader, endian, ())?;
        Ok(raw.try_into().map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: "Invalid UTF-8 string".to_string()
        })?)
    }
}

impl ReadEndian for TextData {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ShortListData {
    pub element_count: u8,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct StatementsData {
    pub statements_count: u32,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub terminated: bool,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct UnboundedStatementsData {
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub terminated: bool,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ShortStatementsData {
    pub statements_count: u8,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub terminated: bool,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ListData {
    pub element_count: u32,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ShortMapData {
    pub element_count: u8,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct MapData {
    pub element_count: u32,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct InstructionCloseAndStore {
    pub instruction: Int8Data,
}

#[derive(BinRead, BinWrite, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[brw(little)]
pub struct StackIndex(pub u32);

impl Display for StackIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AddAssign<u32> for StackIndex {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

#[derive(BinRead, BinWrite, Clone, Copy, Debug, PartialEq)]
#[brw(little)]
pub struct PushToStackMultiple {
    pub count: u32
}

#[derive(
    BinRead, BinWrite, Clone, Debug, PartialEq, Serialize, Deserialize,
)]
#[brw(little)]
pub struct RawRemotePointerAddress {
    pub id: [u8; 26],
}
impl RawRemotePointerAddress {
    pub fn endpoint(&self) -> Result<Endpoint, EndpointParsingError> {
        let mut endpoint = [0u8; 21];
        endpoint.copy_from_slice(&self.id[0..21]);
        Endpoint::from_slice(endpoint)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerAddressConversionError;

impl TryFrom<PointerAddress> for RawRemotePointerAddress {
    type Error = PointerAddressConversionError;
    fn try_from(ptr: PointerAddress) -> Result<Self, Self::Error> {
        match ptr {
            PointerAddress::External(ExternalPointerAddress::Remote(
                                           bytes,
                                       )) => Ok(RawRemotePointerAddress { id: bytes }),
            _ => Err(PointerAddressConversionError),
        }
    }
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct RawLocalPointerAddress {
    pub bytes: [u8; 5],
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct RawInternalPointerAddress {
    pub id: [u8; 3],
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub enum RawPointerAddress {
    #[brw(magic = 0u8)]
    Local(RawLocalPointerAddress),
    #[brw(magic = 1u8)]
    Internal(RawInternalPointerAddress),
    #[brw(magic = 2u8)]
    Remote(RawRemotePointerAddress),
}

impl RawPointerAddress {
    fn get_size(&self) -> usize {
        match self {
            RawPointerAddress::Remote(_) => 26,
            RawPointerAddress::Internal(_) => 3,
            RawPointerAddress::Local(_) => 5,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::with_capacity(1 + self.get_size()));
        self.write_le(&mut writer).expect("Failed to write raw pointer address");
        writer.into_inner()
    }
}

impl From<PointerAddress> for RawPointerAddress {
    fn from(ptr: PointerAddress) -> Self {
        match ptr {
            PointerAddress::External(ExternalPointerAddress::Remote(bytes)) => {
                RawPointerAddress::Remote(RawRemotePointerAddress { id: bytes })
            }
            PointerAddress::External(ExternalPointerAddress::Builtin(bytes)) => {
                RawPointerAddress::Internal(RawInternalPointerAddress { id: bytes })
            }
            PointerAddress::EndpointOwned(SelfOwnedPointerAddress {address} ) => {
                RawPointerAddress::Local(RawLocalPointerAddress { bytes: address })
            }
        }
    }
}


#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct GetOrCreateRemoteRefData {
    pub address: RawRemotePointerAddress,
    pub create_block_size: u64,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct PerformMove {
    pub pointer_count: u32,
    #[br(count = pointer_count)]
    pub pointers: Vec<(u8, RawLocalPointerAddress)>, // FIXME: bool instead of u8
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SharedRef {
    pub address: RawPointerAddress,
    pub ref_mutability: ReferenceMutability,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SharedRefWithValue {
    pub address: RawLocalPointerAddress, // address of the caller
    pub ref_mutability: ReferenceMutability,
    pub container_mutability: SharedContainerMutability,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SetSharedContainerValue {
    pub operator: Option<AssignmentOperator>
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ModifyStackValue {
    pub index: StackIndex,
    pub operator: AssignmentOperator,
}


#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct Move {
    pub pointer_count: u32,
    #[br(count = pointer_count)]
    pub address_mappings: Vec<(RawLocalPointerAddress, RawLocalPointerAddress)>,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct InstructionBlockData {
    pub length: u32,
    pub injected_value_count: u32,
    #[br(count = injected_value_count)]
    pub injected_values: Vec<InjectedValueDeclaration>,
    #[br(count = length)]
    pub body: Vec<u8>,
}


cfg_if! {
    if #[cfg(feature = "disassembler")]{
        use crate::disassembler::InstructionTree;

        #[derive(Clone, Debug, PartialEq)]
        pub struct InstructionBlockDataDebugTree {
            pub length: u32,
            pub injected_variable_count: u32,
            pub injected_values: Vec<InjectedValueDeclaration>,
            pub body: InstructionTree<Instruction>,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct InstructionBlockDataDebugFlat {
            pub length: u32,
            pub injected_variable_count: u32,
            pub injected_values: Vec<InjectedValueDeclaration>,
            pub body: Vec<Instruction>,
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
                    append_instruction(&mut cursor, instruction.clone());
                }
                Self {
                    length: value.length,
                    injected_value_count: value.injected_variable_count,
                    injected_values: value.injected_values.clone(),
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

    }
}


#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ApplyData {
    pub arg_count: u16,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ImplTypeData {
    pub metadata: TypeMetadataBin,
    pub impl_count: u8,
    #[br(count = impl_count)]
    pub impls: Vec<RawPointerAddress>,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct TypeReferenceData {
    pub metadata: TypeMetadataBin,
    pub address: RawPointerAddress,
}

#[bitfield]
#[derive(BinWrite, BinRead, Clone, Copy, Debug, PartialEq)]
#[bw(map = |&x| Self::into_bytes(x))]
#[br(map = Self::from_bytes)]
#[brw(little)]
pub struct TypeMetadataBin {
    pub reference_mutability: TypeReferenceMutabilityCode,
    pub mutability: TypeMutabilityCode,
    pub type_local_or_shared: TypeLocalOrShared,
    _unused: B4,
}

impl From<&TypeMetadataBin> for TypeMetadata {
    fn from(value: &TypeMetadataBin) -> Self {
        match value.type_local_or_shared() {
            TypeLocalOrShared::Local => TypeMetadata::Local {
                mutability: (&value.mutability()).into(),
                reference_mutability: (&value.reference_mutability()).into(),
            },
            TypeLocalOrShared::Shared => TypeMetadata::Shared {
                mutability: (&value.mutability()).into(),
                reference_mutability: (&value.reference_mutability()).into(),
            },
        }
    }
}

impl From<&TypeMetadata> for TypeMetadataBin {
    fn from(value: &TypeMetadata) -> Self {
        match value {
            TypeMetadata::Local {
                mutability,
                reference_mutability,
            } => Self::new()
                .with_type_local_or_shared(TypeLocalOrShared::Local)
                .with_mutability(mutability.into())
                .with_reference_mutability(reference_mutability.into()),
            TypeMetadata::Shared {
                mutability,
                reference_mutability,
            } => Self::new()
                .with_type_local_or_shared(TypeLocalOrShared::Shared)
                .with_mutability(mutability.into())
                .with_reference_mutability(reference_mutability.into()),
        }
    }
}
