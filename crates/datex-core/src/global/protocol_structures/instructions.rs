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
use binrw::{BinRead, BinWrite};
use core::{fmt::Display, prelude::rust_2024::*};
use binrw::io::Cursor;
use modular_bitfield::{bitfield, specifiers::B4};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, PartialEq)]
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
    UnboundedStatementsEnd(bool),
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
    RequestSharedRef(RawRemotePointerAddress),
    // 'mut $ABCDE
    RequestSharedRefMut(RawRemotePointerAddress),
    GetLocalRef(RawLocalPointerAddress),
    GetInternalRef(RawInternalPointerAddress),

    SharedRef(SharedRef),
    SharedRefWithValue(SharedRefWithValue),

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

impl Display for RegularInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegularInstruction::Int8(data) => {
                write!(f, "INT_8 {}", data.0)
            }
            RegularInstruction::Int16(data) => {
                write!(f, "INT_16 {}", data.0)
            }
            RegularInstruction::Int32(data) => {
                write!(f, "INT_32 {}", data.0)
            }
            RegularInstruction::Int64(data) => {
                write!(f, "INT_64 {}", data.0)
            }
            RegularInstruction::Int128(data) => {
                write!(f, "INT_128 {}", data.0)
            }

            RegularInstruction::UInt8(data) => {
                write!(f, "UINT_8 {}", data.0)
            }
            RegularInstruction::UInt16(data) => {
                write!(f, "UINT_16 {}", data.0)
            }
            RegularInstruction::UInt32(data) => {
                write!(f, "UINT_32 {}", data.0)
            }
            RegularInstruction::UInt64(data) => {
                write!(f, "UINT_64 {}", data.0)
            }
            RegularInstruction::UInt128(data) => {
                write!(f, "UINT_128 {}", data.0)
            }
            RegularInstruction::Range => {
                write!(f, "RANGE regular instruction")
            }
            RegularInstruction::Apply(count) => {
                write!(f, "APPLY {}", count.arg_count)
            }
            RegularInstruction::BigInteger(data) => {
                write!(f, "BIG_INTEGER {}", data.0)
            }
            RegularInstruction::Integer(data) => {
                write!(f, "INTEGER {}", data.0)
            }
            RegularInstruction::Endpoint(data) => {
                write!(f, "ENDPOINT {data}")
            }

            RegularInstruction::DecimalAsInt16(data) => {
                write!(f, "DECIMAL_AS_INT_16 {}", data.0)
            }
            RegularInstruction::DecimalAsInt32(data) => {
                write!(f, "DECIMAL_AS_INT_32 {}", data.0)
            }
            RegularInstruction::DecimalF32(data) => {
                write!(
                    f,
                    "DECIMAL_F32 {}",
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::DecimalF64(data) => {
                write!(
                    f,
                    "DECIMAL_F64 {}",
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::BigDecimal(data) => {
                write!(f, "DECIMAL_BIG {}", data.0)
            }
            RegularInstruction::Decimal(data) => {
                write!(f, "DECIMAL {}", data.0)
            }
            RegularInstruction::ShortText(data) => {
                write!(f, "SHORT_TEXT {}", data.0)
            }
            RegularInstruction::Text(data) => {
                write!(f, "TEXT {}", data.0)
            }
            RegularInstruction::True => write!(f, "TRUE"),
            RegularInstruction::False => write!(f, "FALSE"),
            RegularInstruction::Null => write!(f, "NULL"),
            RegularInstruction::Statements(data) => {
                write!(f, "STATEMENTS {}", data.statements_count)
            }
            RegularInstruction::ShortStatements(data) => {
                write!(f, "SHORT_STATEMENTS {}", data.statements_count)
            }
            RegularInstruction::UnboundedStatements => {
                write!(f, "UNBOUNDED_STATEMENTS")
            }
            RegularInstruction::UnboundedStatementsEnd(_) => {
                write!(f, "STATEMENTS_END")
            }
            RegularInstruction::List(data) => {
                write!(f, "LIST {}", data.element_count)
            }
            RegularInstruction::ShortList(data) => {
                write!(f, "SHORT_LIST {}", data.element_count)
            }
            RegularInstruction::Map(data) => {
                write!(f, "MAP {}", data.element_count)
            }
            RegularInstruction::ShortMap(data) => {
                write!(f, "SHORT_MAP {}", data.element_count)
            }
            RegularInstruction::KeyValueDynamic => {
                write!(f, "KEY_VALUE_DYNAMIC")
            }
            RegularInstruction::KeyValueShortText(data) => {
                write!(f, "KEY_VALUE_SHORT_TEXT {}", data.0)
            }
            // operations
            RegularInstruction::Add => write!(f, "ADD"),
            RegularInstruction::Subtract => write!(f, "SUBTRACT"),
            RegularInstruction::Multiply => write!(f, "MULTIPLY"),
            RegularInstruction::Divide => write!(f, "DIVIDE"),

            // equality checks
            RegularInstruction::StructuralEqual => {
                write!(f, "STRUCTURAL_EQUAL")
            }
            RegularInstruction::Equal => write!(f, "EQUAL"),
            RegularInstruction::NotStructuralEqual => {
                write!(f, "NOT_STRUCTURAL_EQUAL")
            }
            RegularInstruction::NotEqual => write!(f, "NOT_EQUAL"),
            RegularInstruction::Is => write!(f, "IS"),
            RegularInstruction::Matches => write!(f, "MATCHES"),

            RegularInstruction::AllocateSlot(address) => {
                write!(f, "ALLOCATE_SLOT {}", address.0)
            }
            RegularInstruction::CloneSlot(address) => {
                write!(f, "GET_SLOT {}", address.0)
            }
            RegularInstruction::GetInternalSlot(address) => {
                write!(f, "GET_INTERNAL_SLOT {}", address.0)
            }
            RegularInstruction::BorrowSlot(address) => {
                write!(f, "GET_SLOT_LOCAL_REF {}", address.0)
            }
            RegularInstruction::GetSlotSharedRef(address) => {
                write!(f, "GET_SLOT_SHARED_REF {}", address.0)
            }
            RegularInstruction::GetSlotSharedRefMut(address) => {
                write!(f, "GET_SLOT_SHARED_REF_MUT {}", address.0)
            }
            RegularInstruction::PopSlot(address) => {
                write!(f, "DROP_SLOT {}", address.0)
            }
            RegularInstruction::SetSlot(address) => {
                write!(f, "SET_SLOT {}", address.0)
            }
            RegularInstruction::SetSharedContainerValue(operator) => {
                write!(f, "SET_REFERENCE_VALUE ({})", operator)
            }
            RegularInstruction::Unbox => write!(f, "UNBOX"),
            RegularInstruction::RequestSharedRef(address) => {
                write!(
                    f,
                    "REQUEST_SHARED_REF [{}:{}]",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::RequestSharedRefMut(address) => {
                write!(
                    f,
                    "REQUEST_SHARED_REF_MUT [{}:{}]",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetLocalRef(address) => {
                write!(
                    f,
                    "GET_LOCAL_REF [origin_id: {}]",
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetInternalRef(address) => {
                write!(
                    f,
                    "GET_INTERNAL_REF [internal_id: {}]",
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetSharedReference => {
                write!(f, "GET_SHARED_REF")
            }
            RegularInstruction::GetSharedReferenceMut => {
                write!(f, "GET_SHARED_REF_MUT")
            }
            RegularInstruction::CreateShared => {
                write!(f, "CREATE_SHARED")
            }
            RegularInstruction::CreateSharedMut => {
                write!(f, "CREATE_SHARED_MUT")
            }
            RegularInstruction::SharedRef(shared_ref) => {
                write!(
                    f,
                    "SHARED_REF (ref_mutability: {:?}, address: {})",
                    shared_ref.ref_mutability, PointerAddress::from(&shared_ref.address)
                )
            }
            RegularInstruction::SharedRefWithValue(shared_ref) => {
                write!(
                    f,
                    "SHARED_REF_WITH_VALUE (ref_mutability: {:?}, address: {}, container_mutability: {:?})",
                    shared_ref.ref_mutability,
                    PointerAddress::from(&shared_ref.address),
                    shared_ref.container_mutability
                )
            }
            RegularInstruction::PerformMove(perform_move) => {
                write!(
                    f,
                    "PERFORM_MOVE (pointers: {})",
                    perform_move.pointers.iter().map(|(_mut, addr)| hex::encode(addr.id)).collect::<Vec<_>>().join(", ")
                )
            }
            RegularInstruction::Move(mv) => {
                write!(
                    f,
                    "MOVE (pointer_count: {}, mappings: {:?})",
                    mv.pointer_count, mv.address_mappings
                )
            }
            RegularInstruction::RemoteExecution(block) => {
                write!(
                    f,
                    "REMOTE_EXECUTION (length: {}, injected_slot_count: {})",
                    block.length,
                    block.injected_slot_count
                )
            }
            RegularInstruction::AddAssign(address) => {
                write!(f, "ADD_ASSIGN {}", address.0)
            }
            RegularInstruction::SubtractAssign(address) => {
                write!(f, "SUBTRACT_ASSIGN {}", address.0)
            }
            RegularInstruction::MultiplyAssign(address) => {
                write!(f, "MULTIPLY_ASSIGN {}", address.0)
            }
            RegularInstruction::DivideAssign(address) => {
                write!(f, "DIVIDE_ASSIGN {}", address.0)
            }
            RegularInstruction::UnaryMinus => write!(f, "-"),
            RegularInstruction::UnaryPlus => write!(f, "+"),
            RegularInstruction::BitwiseNot => write!(f, "BITWISE_NOT"),
            RegularInstruction::TypedValue => write!(f, "TYPED_VALUE"),
            RegularInstruction::TypeExpression => {
                write!(f, "TYPE_EXPRESSION")
            }
            RegularInstruction::GetPropertyIndex(uint_32_data) => {
                write!(f, "GET_PROPERTY_INDEX {}", uint_32_data.0)
            }
            RegularInstruction::SetPropertyIndex(uint_32_data) => {
                write!(f, "SET_PROPERTY_INDEX {}", uint_32_data.0)
            }
            RegularInstruction::TakePropertyIndex(uint_32_data) => {
                write!(f, "TAKE_PROPERTY_INDEX {}", uint_32_data.0)
            }
            RegularInstruction::GetPropertyText(short_text_data) => {
                write!(f, "GET_PROPERTY_TEXT {}", short_text_data.0)
            }
            RegularInstruction::TakePropertyText(short_text_data) => {
                write!(f, "TAKE_PROPERTY_TEXT {}", short_text_data.0)
            }
            RegularInstruction::SetPropertyText(short_text_data) => {
                write!(f, "SET_PROPERTY_TEXT {}", short_text_data.0)
            }
            RegularInstruction::GetPropertyDynamic => {
                write!(f, "GET_PROPERTY_DYNAMIC")
            }
            RegularInstruction::TakePropertyDynamic => {
                write!(f, "TAKE_PROPERTY_DYNAMIC")
            }
            RegularInstruction::SetPropertyDynamic => {
                write!(f, "SET_PROPERTY_DYNAMIC")
            },
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

#[derive(BinRead, BinWrite, Clone, Debug, PartialEq)]
#[brw(little)]
pub struct TextDataRaw {
    pub length: u32,
    #[br(count = length)]
    pub text: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShortTextData(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct TextData(pub String);

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
    pub address: RawPointerAddress,
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
