use crate::{
    core_compiler::value_compiler::append_instruction,
    global::{
        operators::ModificationOperator,
        protocol_structures::{
            injected_values::InjectedValueDeclaration,
            instructions::Instruction,
        },
        type_instruction_codes::{
            TypeLocalOrShared, TypeMutabilityCode, TypeOwnershipCode,
        },
    },
    prelude::*,
    shared_values::{
        PointerAddress, ReferenceMutability, RemotePointerAddress,
        SelfOwnedPointerAddress, SharedContainerMutability,
    },
    types::{
        type_definition::callable::CallableKind,
        type_definition_with_metadata::TypeMetadata,
    },
    values::core_values::{decimal::Decimal, integer::Integer},
};
use alloc::string::FromUtf8Error;
use binrw::{
    BinRead, BinResult, BinWrite, Endian,
    io::{Cursor, Read, Seek, Write},
    meta::{EndianKind, ReadEndian},
};
use cfg_if::cfg_if;
use core::{fmt::Display, ops::AddAssign};
use itertools::Itertools;
use modular_bitfield::{bitfield, prelude::B4};

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
pub struct InstantData(pub i128);

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
        raw.try_into().map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: "Invalid UTF-8 string".to_string(),
        })
    }
}

impl ReadEndian for ShortTextData {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct TaggedValue {
    pub(crate) tag: ShortTextData,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub(crate) is_empty: bool,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableSignatureData {
    pub name: ShortTextData, // empty string if anonymous
    pub kind: CallableKind,
    pub parameter_count: u8,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub requires_async: bool,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub has_rest_parameter: bool,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub has_return_type: bool,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |b: &bool| if *b { 1u8 } else { 0u8 })]
    pub has_yeet_type: bool,
    #[br(count = parameter_count)]
    pub parameter_names: Vec<ShortTextData>,
    #[br(if(has_rest_parameter))]
    pub rest_parameter_name: Option<ShortTextData>,
}

impl Display for CallableSignatureData {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> Result<(), core::fmt::Error> {
        write!(formatter, "[")?;
        write!(formatter, "kind: {}, ", self.kind)?;
        write!(formatter, "requires_async: {}, ", self.requires_async)?;
        write!(
            formatter,
            "parameters: [{}], ",
            self.parameter_names.iter().map(|n| &n.0).join(", ")
        )?;
        if let Some(rest) = &self.rest_parameter_name {
            write!(formatter, ", rest_parameter: {}, ", rest.0)?;
        }
        write!(formatter, "has_return_type: {}, ", self.has_return_type)?;
        write!(formatter, "has_yeet_type: {}", self.has_yeet_type)?;
        write!(formatter, "]")?;
        Ok(())
    }
}

impl CallableSignatureData {
    /// Returns the total number of types in the signature, including parameters, rest parameter, return type, and yeet type.
    pub fn total_type_count(&self) -> u32 {
        let mut count = self.parameter_count as u32;
        if self.has_rest_parameter {
            count += 1;
        }
        if self.has_return_type {
            count += 1;
        }
        if self.has_yeet_type {
            count += 1;
        }
        count
    }
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDeclarationData {
    pub signature: CallableSignatureData,
    pub body: InstructionBlockData,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableData {
    pub signature: CallableSignatureData,
    pub body: CallableDataBody,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct CallableDataBody {
    pub injected_value_count: u32,
    pub length: u32, // if length is 0, the body has a native implementation
    #[br(count = length)]
    pub body: Vec<u8>,
}

impl Display for CallableDataBody {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[length: {}, injected_value_count: {}, body_len: {} bytes]",
            self.length,
            self.injected_value_count,
            self.body.len()
        )
    }
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

impl From<String> for TextData {
    fn from(value: String) -> Self {
        TextData(value)
    }
}
impl From<&str> for TextData {
    fn from(value: &str) -> Self {
        TextData(value.to_string())
    }
}

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
        raw.try_into().map_err(|_| binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: "Invalid UTF-8 string".to_string(),
        })
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

#[derive(
    BinRead, BinWrite, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[brw(little)]
pub struct StackIndex(pub u32);

impl Display for StackIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{{}}}", self.0)
    }
}

impl AddAssign<u32> for StackIndex {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct GetOrCreateRemoteRefData {
    pub address: RemotePointerAddress,
    pub create_block_size: u64,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct MoveWithValue {
    pub mutability: SharedContainerMutability,
    pub previous_address: SelfOwnedPointerAddress,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SharedRef {
    pub address: PointerAddress,
    pub ref_mutability: ReferenceMutability,
    pub container_mutability: SharedContainerMutability,
    // TODO: hash
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SharedRefWithValue {
    pub address: SelfOwnedPointerAddress, // address of the caller
    pub ref_mutability: ReferenceMutability,
    pub container_mutability: SharedContainerMutability,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ModifySharedContainerValue {
    pub operator: ModificationOperator,
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

impl Display for InstructionBlockData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[length: {}, injected_value_count: {}, injected_values: [{}], body_len: {} bytes]",
            self.length,
            self.injected_value_count,
            self.injected_values
                .iter()
                .map(|v| format!("{}", v))
                .join(", "),
            self.body.len()
        )
    }
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SpliceData {
    pub start_index: u32,
    pub delete_count: u32,
    pub insert_count: u32,
}

cfg_if! {
    if #[cfg(feature = "disassembler")]{
        use crate::disassembler::InstructionTree;

        #[derive(Clone, Debug, PartialEq, Default)]
        pub struct InstructionBlockDataDebugTree {
            pub length: u32,
            pub injected_variable_count: u32,
            pub injected_values: Vec<InjectedValueDeclaration>,
            pub body: InstructionTree<Instruction>,
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
            pub body: Vec<Instruction>,
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
pub struct JumpData {
    pub offset: i32,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct JumpWithValueData {
    pub offset: i32,
    pub value: i32, // For now it will be just "i32", to test loops and maybe some easy inline functions, but later will be changed to universal value
                    // and maybe changed to something like "Vec<Value>", so we can send more then one Value at time
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ApplyData {
    pub arg_count: u16,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct ImplTypeData {
    pub impl_count: u8,
    #[br(count = impl_count)]
    pub impls: Vec<PointerAddress>,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct TypeReferenceData {
    pub address: PointerAddress,
}

#[bitfield]
#[derive(BinWrite, BinRead, Clone, Copy, Debug, PartialEq)]
#[bw(map = |&x| Self::into_bytes(x))]
#[br(map = Self::from_bytes)]
#[brw(little)]
pub struct TypeMetadataBin {
    pub ownership: TypeOwnershipCode,
    pub mutability: TypeMutabilityCode,
    pub type_local_or_shared: TypeLocalOrShared,
    #[skip]
    _unused: B4,
}

impl From<&TypeMetadataBin> for TypeMetadata {
    fn from(value: &TypeMetadataBin) -> Self {
        match value.type_local_or_shared() {
            TypeLocalOrShared::Local => TypeMetadata::Local {
                mutability: (&value.mutability()).into(),
                ownership: (&value.ownership()).into(),
            },
            TypeLocalOrShared::Shared => TypeMetadata::Shared {
                mutability: (&value.mutability()).into(),
                ownership: (&value.ownership()).into(),
            },
        }
    }
}

impl From<&TypeMetadata> for TypeMetadataBin {
    fn from(value: &TypeMetadata) -> Self {
        match value {
            TypeMetadata::Local {
                mutability,
                ownership: reference_mutability,
            } => Self::new()
                .with_type_local_or_shared(TypeLocalOrShared::Local)
                .with_mutability(mutability.into())
                .with_ownership(reference_mutability.into()),
            TypeMetadata::Shared {
                mutability,
                ownership,
            } => Self::new()
                .with_type_local_or_shared(TypeLocalOrShared::Shared)
                .with_mutability(mutability.into())
                .with_ownership(ownership.into()),
        }
    }
}
