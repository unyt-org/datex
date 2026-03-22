use alloc::string::FromUtf8Error;
use crate::{
    global::{
        operators::AssignmentOperator,
        type_instruction_codes::TypeReferenceMutabilityCode,
    },
    values::core_values::{
        decimal::{Decimal, utils::decimal_to_string},
        endpoint::{Endpoint, EndpointParsingError},
        integer::Integer,
    },
};

use crate::{
    global::type_instruction_codes::{TypeLocalOrShared, TypeMutabilityCode},
    prelude::*,
    shared_values::pointer_address::{
        PointerAddress, ReferencedPointerAddress,
    },
    values::core_values::r#type::TypeMetadata,
};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use core::{fmt::Display, prelude::rust_2024::*};
use crate::std::io::{Seek, Read, Write};
use binrw::io::Cursor;
use modular_bitfield::{bitfield, specifiers::B4};
use serde::{Deserialize, Serialize};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::external_slot_type::ExternalSlotType;
use crate::shared_values::pointer::PointerReferenceMutability;
use crate::shared_values::pointer_address::OwnedPointerAddress;
use crate::shared_values::shared_container::SharedContainerMutability;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    // regular instruction
    RegularInstruction(RegularInstruction),
    // Type instruction that yields a type
    TypeInstruction(TypeInstruction),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Instruction::RegularInstruction(instr) => {
                write!(f, "{}", instr)
            }
            Instruction::TypeInstruction(instr) => {
                write!(f, "TYPE_INSTRUCTION {}", instr)
            }
        }
    }
}

impl From<RegularInstruction> for Instruction {
    fn from(instruction: RegularInstruction) -> Self {
        Instruction::RegularInstruction(instruction)
    }
}

impl From<TypeInstruction> for Instruction {
    fn from(instruction: TypeInstruction) -> Self {
        Instruction::TypeInstruction(instruction)
    }
}

#[derive(Clone, Debug, PartialEq, BinWrite)]
#[brw(little)]
pub enum RegularInstruction {
    // signed integers
    Int8(Int8Data),
    Int16(Int16Data),
    Int32(Int32Data),
    Int64(Int64Data),
    Int128(Int128Data),

    // unsigned integers
    UInt8(UInt8Data),
    UInt16(UInt16Data),
    UInt32(UInt32Data),
    UInt64(UInt64Data),
    UInt128(UInt128Data),

    // big integers
    BigInteger(IntegerData),

    // default integer
    Integer(IntegerData),
    Range,

    Endpoint(Endpoint),

    DecimalF32(Float32Data),
    DecimalF64(Float64Data),
    DecimalAsInt16(FloatAsInt16Data),
    DecimalAsInt32(FloatAsInt32Data),
    BigDecimal(DecimalData),
    // default decimal
    Decimal(DecimalData),

    RemoteExecution(InstructionBlockData),

    ShortText(ShortTextData),
    Text(TextData),
    True,
    False,
    Null,
    Statements(StatementsData),
    ShortStatements(StatementsData),
    UnboundedStatements,
    UnboundedStatementsEnd(UnboundedStatementsData),
    List(ListData),
    ShortList(ListData),
    Map(MapData),
    ShortMap(MapData),

    KeyValueDynamic,
    KeyValueShortText(ShortTextData),

    // binary operator
    Add,
    Subtract,
    Multiply,
    Divide,

    // unary operator
    // TODO #432 add missing unary operators
    UnaryMinus,
    // TODO #433: Do we need this for op overloading or can we avoid?
    UnaryPlus,
    BitwiseNot,

    Apply(ApplyData),

    GetPropertyText(ShortTextData),
    SetPropertyText(ShortTextData),
    TakePropertyText(ShortTextData),

    GetPropertyIndex(UInt32Data),
    SetPropertyIndex(UInt32Data),
    TakePropertyIndex(UInt32Data),

    GetPropertyDynamic,
    SetPropertyDynamic,
    TakePropertyDynamic,

    // comparison operator
    Is,
    Matches,
    StructuralEqual,
    Equal,
    NotStructuralEqual,
    NotEqual,

    // assignment operator
    AddAssign(SlotAddress),
    SubtractAssign(SlotAddress),
    MultiplyAssign(SlotAddress),
    DivideAssign(SlotAddress),

    GetSharedReference,
    GetSharedReferenceMut,

    CreateShared,
    CreateSharedMut,

    // ' $ABCDE
    RequestRemoteSharedRef(RawRemotePointerAddress),
    // 'mut $ABCDE
    RequestRemoteSharedRefMut(RawRemotePointerAddress),
    GetLocalSharedRef(RawLocalPointerAddress),
    GetInternalSharedRef(RawInternalPointerAddress),

    SharedRef(SharedRef),
    SharedRefWithValue(SharedRefWithValue), // shared ref with current value (only if caller owns the pointer)

    PerformMove(PerformMove),
    Move(Move),

    AllocateSlot(SlotAddress),
    CloneSlot(SlotAddress),
    BorrowSlot(SlotAddress),
    GetSlotSharedRef(SlotAddress),
    GetSlotSharedRefMut(SlotAddress),
    PopSlot(SlotAddress),
    SetSlot(SlotAddress),

    GetInternalSlot(SlotAddress),

    SetSharedContainerValue(AssignmentOperator),
    Unbox,

    TypedValue,
    TypeExpression,
}

/// Maps each regular instruction to its corresponding instruction code
impl From<&RegularInstruction> for InstructionCode {
    fn from(instruction: &RegularInstruction) -> Self {
        match instruction {
            RegularInstruction::Int8(_) => InstructionCode::INT_8,
            RegularInstruction::Int16(_) => InstructionCode::INT_16,
            RegularInstruction::Int32(_) => InstructionCode::INT_32,
            RegularInstruction::Int64(_) => InstructionCode::INT_64,
            RegularInstruction::Int128(_) => InstructionCode::INT_128,
            RegularInstruction::UInt8(_) => InstructionCode::UINT_8,
            RegularInstruction::UInt16(_) => InstructionCode::UINT_16,
            RegularInstruction::UInt32(_) => InstructionCode::UINT_32,
            RegularInstruction::UInt64(_) => InstructionCode::UINT_64,
            RegularInstruction::UInt128(_) => InstructionCode::UINT_128,
            RegularInstruction::BigInteger(_) => InstructionCode::INT_BIG,
            RegularInstruction::Integer(_) => InstructionCode::INT_32,
            RegularInstruction::Endpoint(_) => InstructionCode::ENDPOINT,
            RegularInstruction::DecimalF32(_) => InstructionCode::DECIMAL_F32,
            RegularInstruction::DecimalF64(_) => InstructionCode::DECIMAL_F64,
            RegularInstruction::DecimalAsInt16(_) => InstructionCode::DECIMAL_AS_INT_16,
            RegularInstruction::DecimalAsInt32(_) => InstructionCode::DECIMAL_AS_INT_32,
            RegularInstruction::BigDecimal(_) => InstructionCode::DECIMAL_BIG,
            RegularInstruction::Decimal(_) => InstructionCode::DECIMAL,
            RegularInstruction::Range => InstructionCode::RANGE,
            RegularInstruction::RemoteExecution(_) => InstructionCode::REMOTE_EXECUTION,
            RegularInstruction::ShortText(_) => InstructionCode::SHORT_TEXT,
            RegularInstruction::Text(_) => InstructionCode::TEXT,
            RegularInstruction::True => InstructionCode::TRUE,
            RegularInstruction::False => InstructionCode::FALSE,
            RegularInstruction::Null => InstructionCode::NULL,
            RegularInstruction::Statements(_) => InstructionCode::STATEMENTS,
            RegularInstruction::ShortStatements(_) => InstructionCode::SHORT_STATEMENTS,
            RegularInstruction::UnboundedStatements => InstructionCode::UNBOUNDED_STATEMENTS,
            RegularInstruction::UnboundedStatementsEnd(_) => InstructionCode::UNBOUNDED_STATEMENTS_END,
            RegularInstruction::List(_) => InstructionCode::LIST,
            RegularInstruction::ShortList(_) => InstructionCode::SHORT_LIST,
            RegularInstruction::Map(_) => InstructionCode::MAP,
            RegularInstruction::ShortMap(_) => InstructionCode::SHORT_MAP,
            RegularInstruction::KeyValueDynamic => InstructionCode::KEY_VALUE_DYNAMIC,
            RegularInstruction::KeyValueShortText(_) => InstructionCode::KEY_VALUE_SHORT_TEXT,
            RegularInstruction::Add => InstructionCode::ADD,
            RegularInstruction::Subtract => InstructionCode::SUBTRACT,
            RegularInstruction::Multiply => InstructionCode::MULTIPLY,
            RegularInstruction::Divide => InstructionCode::DIVIDE,
            RegularInstruction::UnaryMinus => InstructionCode::UNARY_MINUS,
            RegularInstruction::UnaryPlus => InstructionCode::UNARY_PLUS,
            RegularInstruction::BitwiseNot => InstructionCode::BITWISE_NOT,
            RegularInstruction::Apply(_) => InstructionCode::APPLY,
            RegularInstruction::GetPropertyText(_) => InstructionCode::GET_PROPERTY_TEXT,
            RegularInstruction::SetPropertyText(_) => InstructionCode::SET_PROPERTY_TEXT,
            RegularInstruction::TakePropertyText(_) => InstructionCode::TAKE_PROPERTY_TEXT,
            RegularInstruction::GetPropertyIndex(_) => InstructionCode::GET_PROPERTY_INDEX,
            RegularInstruction::SetPropertyIndex(_) => InstructionCode::SET_PROPERTY_INDEX,
            RegularInstruction::TakePropertyIndex(_) => InstructionCode::TAKE_PROPERTY_INDEX,
            RegularInstruction::GetPropertyDynamic => InstructionCode::GET_PROPERTY_DYNAMIC,
            RegularInstruction::SetPropertyDynamic => InstructionCode::SET_PROPERTY_DYNAMIC,
            RegularInstruction::TakePropertyDynamic => InstructionCode::TAKE_PROPERTY_DYNAMIC,
            RegularInstruction::Is => InstructionCode::IS,
            RegularInstruction::Matches => InstructionCode::MATCHES,
            RegularInstruction::StructuralEqual => InstructionCode::STRUCTURAL_EQUAL,
            RegularInstruction::Equal => InstructionCode::EQUAL,
            RegularInstruction::NotStructuralEqual => InstructionCode::NOT_STRUCTURAL_EQUAL,
            RegularInstruction::NotEqual => InstructionCode::NOT_EQUAL,
            RegularInstruction::AddAssign(_) => InstructionCode::ADD_ASSIGN,
            RegularInstruction::SubtractAssign(_) => InstructionCode::SUBTRACT_ASSIGN,
            RegularInstruction::MultiplyAssign(_) => InstructionCode::MULTIPLY_ASSIGN,
            RegularInstruction::DivideAssign(_) => InstructionCode::DIVIDE_ASSIGN,
            RegularInstruction::GetSharedReference => InstructionCode::GET_SHARED_REF,
            RegularInstruction::GetSharedReferenceMut => InstructionCode::GET_SHARED_REF_MUT,
            RegularInstruction::CreateShared => InstructionCode::CREATE_SHARED,
            RegularInstruction::CreateSharedMut => InstructionCode::CREATE_SHARED_MUT,
            RegularInstruction::RequestRemoteSharedRef(_) => InstructionCode::REQUEST_REMOTE_SHARED_REF,
            RegularInstruction::RequestRemoteSharedRefMut(_) => InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT,
            RegularInstruction::GetLocalSharedRef(_) => InstructionCode::GET_LOCAL_SHARED_REF,
            RegularInstruction::GetInternalSharedRef(_) => InstructionCode::GET_INTERNAL_SHARED_REF,
            RegularInstruction::SharedRef(_) => InstructionCode::SHARED_REF,
            RegularInstruction::SharedRefWithValue(_) => InstructionCode::SHARED_REF_WITH_VALUE,
            RegularInstruction::PerformMove(_) => InstructionCode::PERFORM_MOVE,
            RegularInstruction::Move(_) => InstructionCode::MOVE,
            RegularInstruction::AllocateSlot(_) => InstructionCode::ALLOCATE_SLOT,
            RegularInstruction::CloneSlot(_) => InstructionCode::CLONE_SLOT,
            RegularInstruction::BorrowSlot(_) => InstructionCode::BORROW_SLOT,
            RegularInstruction::GetSlotSharedRef(_) => InstructionCode::GET_SLOT_SHARED_REF,
            RegularInstruction::GetSlotSharedRefMut(_) => InstructionCode::GET_SLOT_SHARED_REF_MUT,
            RegularInstruction::PopSlot(_) => InstructionCode::POP_SLOT,
            RegularInstruction::SetSlot(_) => InstructionCode::SET_SLOT,
            RegularInstruction::GetInternalSlot(_) => InstructionCode::GET_INTERNAL_SLOT,
            RegularInstruction::SetSharedContainerValue(_) => InstructionCode::SET_SHARED_CONTAINER_VALUE,
            RegularInstruction::Unbox => InstructionCode::UNBOX,
            RegularInstruction::TypedValue => InstructionCode::TYPED_VALUE,
            RegularInstruction::TypeExpression => InstructionCode::TYPE_EXPRESSION,
        }
    }
}

impl Display for RegularInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = InstructionCode::from(self);
        write!(f, "{} ", code)?;

        match self {
            RegularInstruction::Int8(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Int16(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Int32(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Int64(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Int128(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::UInt8(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::UInt16(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::UInt32(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::UInt64(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::UInt128(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Apply(count) => {
                write!(f, "(arg_count: {})", count.arg_count)
            }
            RegularInstruction::BigInteger(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Integer(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Endpoint(data) => {
                write!(f, "{data}")
            }

            RegularInstruction::DecimalAsInt16(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::DecimalAsInt32(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::DecimalF32(data) => {
                write!(
                    f,
                    "{}",
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::DecimalF64(data) => {
                write!(
                    f,
                    "{}",
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::BigDecimal(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Decimal(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::ShortText(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Text(data) => {
                write!(f, "{}", data.0)
            }
            RegularInstruction::Statements(data) => {
                write!(f, "{}", data.statements_count)
            }
            RegularInstruction::ShortStatements(data) => {
                write!(f, "{}", data.statements_count)
            }
            RegularInstruction::List(data) => {
                write!(f, "{}", data.element_count)
            }
            RegularInstruction::ShortList(data) => {
                write!(f, "{}", data.element_count)
            }
            RegularInstruction::Map(data) => {
                write!(f, "{}", data.element_count)
            }
            RegularInstruction::ShortMap(data) => {
                write!(f, "{}", data.element_count)
            }
            RegularInstruction::KeyValueShortText(data) => {
                write!(f, "{}", data.0)
            }

            RegularInstruction::AllocateSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::CloneSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::GetInternalSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::BorrowSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::GetSlotSharedRef(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::GetSlotSharedRefMut(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::PopSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::SetSlot(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::SetSharedContainerValue(operator) => {
                write!(f, "{}", operator)
            }
            RegularInstruction::RequestRemoteSharedRef(address) => {
                write!(
                    f,
                    "({}:{})",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::RequestRemoteSharedRefMut(address) => {
                write!(
                    f,
                    "({}:{})",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetLocalSharedRef(address) => {
                write!(
                    f,
                    "(origin_id: {})",
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetInternalSharedRef(address) => {
                write!(
                    f,
                    "(internal_id: {})",
                    hex::encode(address.id)
                )
            }
            RegularInstruction::SharedRef(shared_ref) => {
                write!(
                    f,
                    "(ref_mutability: {:?}, address: {})",
                    shared_ref.ref_mutability, PointerAddress::from(&shared_ref.address)
                )
            }
            RegularInstruction::SharedRefWithValue(shared_ref) => {
                write!(
                    f,
                    "(ref_mutability: {:?}, address: {}, container_mutability: {:?})",
                    shared_ref.ref_mutability,
                    PointerAddress::from(&shared_ref.address),
                    shared_ref.container_mutability
                )
            }
            RegularInstruction::PerformMove(perform_move) => {
                write!(
                    f,
                    "(pointers: {})",
                    perform_move.pointers.iter().map(|(_mut, addr)| hex::encode(addr.id)).collect::<Vec<_>>().join(", ")
                )
            }
            RegularInstruction::Move(mv) => {
                write!(
                    f,
                    "(pointer_count: {}, mappings: {:?})",
                    mv.pointer_count, mv.address_mappings
                )
            }
            RegularInstruction::RemoteExecution(block) => {
                write!(
                    f,
                    "(length: {}, injected_slot_count: {})",
                    block.length,
                    block.injected_slot_count
                )
            }
            RegularInstruction::AddAssign(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::SubtractAssign(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::MultiplyAssign(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::DivideAssign(address) => {
                write!(f, "{}", address.0)
            }
            RegularInstruction::GetPropertyIndex(uint_32_data) => {
                write!(f, "{}", uint_32_data.0)
            }
            RegularInstruction::SetPropertyIndex(uint_32_data) => {
                write!(f, "{}", uint_32_data.0)
            }
            RegularInstruction::TakePropertyIndex(uint_32_data) => {
                write!(f, "{}", uint_32_data.0)
            }
            RegularInstruction::GetPropertyText(short_text_data) => {
                write!(f, "{}", short_text_data.0)
            }
            RegularInstruction::TakePropertyText(short_text_data) => {
                write!(f, "{}", short_text_data.0)
            }
            RegularInstruction::SetPropertyText(short_text_data) => {
                write!(f, "{}", short_text_data.0)
            }
            _ => {
                // no custom disassembly
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeInstruction {
    ImplType(ImplTypeData),
    SharedTypeReference(TypeReferenceData),
    LiteralText(TextData),
    LiteralInteger(IntegerData),
    List(ListData),
    Range, // TODO #670: add more type instructions
}

impl Display for TypeInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeInstruction::LiteralText(data) => {
                write!(f, "LITERAL_TEXT {}", data.0)
            }
            TypeInstruction::LiteralInteger(data) => {
                write!(f, "LITERAL_INTEGER {}", data.0)
            }
            TypeInstruction::List(data) => {
                write!(f, "LIST {}", data.element_count)
            }
            TypeInstruction::SharedTypeReference(reference_data) => {
                write!(
                    f,
                    "TYPE_REFERENCE mutability: {:?}, address: {}",
                    TypeMetadata::from(&reference_data.metadata),
                    PointerAddress::from(&reference_data.address)
                )
            }
            TypeInstruction::ImplType(data) => {
                write!(f, "IMPL_TYPE ({} impls)", data.impl_count)
            }
            TypeInstruction::Range => {
                write!(f, "Range type instruction")
            }
        }
    }
}

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
            pos: 0, // TODO
            message: "Invalid UTF-8 string".to_string()
        })?)
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
            pos: 0, // TODO
            message: "Invalid UTF-8 string".to_string()
        })?)
    }
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

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SlotAddress(pub u32);

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
            PointerAddress::Referenced(ReferencedPointerAddress::Remote(
                bytes,
            )) => Ok(RawRemotePointerAddress { id: bytes }),
            _ => Err(PointerAddressConversionError),
        }
    }
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct RawLocalPointerAddress {
    pub id: [u8; 5],
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
            PointerAddress::Referenced(ReferencedPointerAddress::Remote(bytes)) => {
                RawPointerAddress::Remote(RawRemotePointerAddress { id: bytes })
            }
            PointerAddress::Referenced(ReferencedPointerAddress::Internal(bytes)) => {
                RawPointerAddress::Internal(RawInternalPointerAddress { id: bytes })
            }
            PointerAddress::Owned(OwnedPointerAddress {address} ) => {
                RawPointerAddress::Local(RawLocalPointerAddress { id: address })
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
    pub ref_mutability: PointerReferenceMutability,
}

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct SharedRefWithValue {
    pub address: RawLocalPointerAddress, // address of the caller
    pub ref_mutability: PointerReferenceMutability,
    pub container_mutability: SharedContainerMutability,
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
    pub injected_slot_count: u32,
    #[br(count = injected_slot_count)]
    pub injected_slots: Vec<(u32, ExternalSlotType)>,
    #[br(count = length)]
    pub body: Vec<u8>,
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
