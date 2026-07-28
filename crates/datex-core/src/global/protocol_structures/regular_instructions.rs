#[cfg(feature = "disassembler")]
use crate::disassembler::InnerInstructions;
use crate::{
    dxb_parser::body::DXBParserError,
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::{
            instruction_data::{
                ApplyData, Float32Data, Float64Data, FloatAsInt16Data,
                FloatAsInt32Data, InstantData, InstructionBlockData,
                InstructionBlockDataDebugFlat, InstructionBlockDataDebugTree,
                Int8Data, Int16Data, Int32Data, Int64Data, Int128Data,
                ListData, MapData, MoveWithValue, SharedRef,
                SharedRefWithValue, ShortListData, ShortMapData,
                ShortStatementsData, ShortTextData, SpliceData, StackIndex,
                StatementsData, TaggedValue, TextData, UInt8Data, UInt16Data,
                UInt32Data, UInt64Data, UInt128Data, UnboundedStatementsData,
            },
            instructions::NextExpectedInstructions,
        },
        root_properties::RootProperty,
    },
    libs::core::core_lib_id::CoreLibIdIndex,
    prelude::*,
    shared_values::{
        PointerAddress, RemotePointerAddress, SelfOwnedPointerAddress,
    },
    values::core_values::{
        Instant,
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        integer::Integer,
    },
};
use binrw::{
    BinRead, BinResult, BinWrite, Endian, binrw,
    io::{Read, Seek},
    meta::{EndianKind, ReadEndian},
};
use core::fmt::{Display, Write as FmtWrite};

#[binrw]
#[derive(Clone, Debug, PartialEq)]
#[brw(little)]
pub struct RegularInstruction {
    #[br(temp)]
    #[bw(calc = data.instruction_code())]
    code: InstructionCode,

    #[br(args(code))]
    data: RegularInstructionData,
}

impl RegularInstruction {
    // fn new(code: InstructionCode, data: RegularInstructionData) -> Self {
    //     Self { code, data }
    // }

    pub fn code(&self) -> InstructionCode {
        self.data.instruction_code()
    }

    pub fn data(&self) -> &RegularInstructionData {
        &self.data
    }
    fn new(data: RegularInstructionData) -> Self {
        Self { data }
    }
    pub fn into_data(self) -> RegularInstructionData {
        self.data
    }
}

impl RegularInstruction {
    pub fn int8(value: i8) -> Self {
        Self::new(RegularInstructionData::Int8(Int8Data(value)))
    }
    pub fn int16(value: i16) -> Self {
        Self::new(RegularInstructionData::Int16(Int16Data(value)))
    }
    pub fn int32(value: i32) -> Self {
        Self::new(RegularInstructionData::Int32(Int32Data(value)))
    }
    pub fn int64(value: i64) -> Self {
        Self::new(RegularInstructionData::Int64(Int64Data(value)))
    }
    pub fn int128(value: i128) -> Self {
        Self::new(RegularInstructionData::Int128(Int128Data(value)))
    }
    pub fn uint8(value: u8) -> Self {
        Self::new(RegularInstructionData::UInt8(UInt8Data(value)))
    }
    pub fn uint16(value: u16) -> Self {
        Self::new(RegularInstructionData::UInt16(UInt16Data(value)))
    }
    pub fn uint32(value: u32) -> Self {
        Self::new(RegularInstructionData::UInt32(UInt32Data(value)))
    }
    pub fn uint64(value: u64) -> Self {
        Self::new(RegularInstructionData::UInt64(UInt64Data(value)))
    }
    pub fn uint128(value: u128) -> Self {
        Self::new(RegularInstructionData::UInt128(UInt128Data(value)))
    }
    pub fn decimal_f32(value: f32) -> Self {
        Self::new(RegularInstructionData::DecimalF32(Float32Data(value)))
    }
    pub fn decimal_f64(value: f64) -> Self {
        Self::new(RegularInstructionData::DecimalF64(Float64Data(value)))
    }
    pub fn decimal_as_int16(value: i16) -> Self {
        Self::new(RegularInstructionData::DecimalAsInt16(FloatAsInt16Data(
            value,
        )))
    }
    pub fn decimal_as_int32(value: i32) -> Self {
        Self::new(RegularInstructionData::DecimalAsInt32(FloatAsInt32Data(
            value,
        )))
    }
    pub fn decimal_big(value: Decimal) -> Self {
        Self::new(RegularInstructionData::BigDecimal(value))
    }
    pub fn decimal(value: Decimal) -> Self {
        Self::new(RegularInstructionData::Decimal(value))
    }
    pub fn integer(value: Integer) -> Self {
        Self::new(RegularInstructionData::Integer(value))
    }
    pub fn big_integer(value: Integer) -> Self {
        Self::new(RegularInstructionData::BigInteger(value))
    }
    pub fn endpoint(value: Endpoint) -> Self {
        Self::new(RegularInstructionData::Endpoint(value))
    }
    pub fn instant(value: i128) -> Self {
        Self::new(RegularInstructionData::Instant(InstantData(value)))
    }
    pub fn text(value: String) -> Self {
        Self::new(RegularInstructionData::Text(TextData(value)))
    }
    pub fn short_text(value: String) -> Self {
        Self::new(RegularInstructionData::ShortText(ShortTextData(value)))
    }
    pub fn tagged_value(tag: String, is_empty: bool) -> Self {
        Self::new(RegularInstructionData::TaggedValue(TaggedValue {
            tag: ShortTextData(tag),
            is_empty,
        }))
    }
    pub fn list(count: u32) -> Self {
        let data = RegularInstructionData::list(count);
        Self::new(data)
    }
    pub fn map(count: u32) -> Self {
        let data = match count {
            0..=255 => RegularInstructionData::ShortMap(ShortMapData {
                element_count: count as u8,
            }),
            _ => RegularInstructionData::Map(MapData {
                element_count: count,
            }),
        };
        Self::new(data)
    }
    pub fn statements(count: u32, terminated: bool) -> Self {
        Self::new(RegularInstructionData::statements(count, terminated))
    }
    pub fn unbounded_statements() -> Self {
        Self::new(RegularInstructionData::UnboundedStatements)
    }
    pub fn unbounded_statements_end(terminated: bool) -> Self {
        Self::new(RegularInstructionData::UnboundedStatementsEnd(
            UnboundedStatementsData { terminated },
        ))
    }
    pub fn apply(arg_count: u16) -> Self {
        let data = RegularInstructionData::Apply(ApplyData { arg_count });
        Self::new(data)
    }
    pub fn get_property_text(key: String) -> Self {
        Self::new(RegularInstructionData::GetPropertyText(ShortTextData(key)))
    }
    pub fn get_property_index(index: u32) -> Self {
        Self::new(RegularInstructionData::GetPropertyIndex(UInt32Data(index)))
    }
    pub fn get_property_dynamic() -> Self {
        Self::new(RegularInstructionData::GetPropertyDynamic)
    }
    pub fn take_property_text(key: String) -> Self {
        Self::new(RegularInstructionData::TakeEntryText(ShortTextData(key)))
    }
    pub fn take_property_index(index: u32) -> Self {
        Self::new(RegularInstructionData::TakeEntryIndex(UInt32Data(index)))
    }
    pub fn take_property_dynamic() -> Self {
        Self::new(RegularInstructionData::TakeEntryDynamic)
    }
    pub fn set_property_text(key: String) -> Self {
        Self::new(RegularInstructionData::SetEntryText(ShortTextData(key)))
    }
    pub fn set_property_index(index: u32) -> Self {
        Self::new(RegularInstructionData::SetEntryIndex(UInt32Data(index)))
    }
    pub fn set_property_dynamic() -> Self {
        Self::new(RegularInstructionData::SetEntryDynamic)
    }
    pub fn matches() -> Self {
        Self::new(RegularInstructionData::Matches)
    }
    pub fn structural_equal() -> Self {
        Self::new(RegularInstructionData::StructuralEqual)
    }
    pub fn not_structural_equal() -> Self {
        Self::new(RegularInstructionData::NotStructuralEqual)
    }
    pub fn equal() -> Self {
        Self::new(RegularInstructionData::Equal)
    }
    pub fn not_equal() -> Self {
        Self::new(RegularInstructionData::NotEqual)
    }
    pub fn is() -> Self {
        Self::new(RegularInstructionData::Is)
    }
    pub fn add() -> Self {
        Self::new(RegularInstructionData::Add)
    }
    pub fn subtract() -> Self {
        Self::new(RegularInstructionData::Subtract)
    }
    pub fn multiply() -> Self {
        Self::new(RegularInstructionData::Multiply)
    }
    pub fn divide() -> Self {
        Self::new(RegularInstructionData::Divide)
    }
    pub fn unary_plus() -> Self {
        Self::new(RegularInstructionData::UnaryPlus)
    }
    pub fn unary_minus() -> Self {
        Self::new(RegularInstructionData::UnaryMinus)
    }
    pub fn bitwise_not() -> Self {
        Self::new(RegularInstructionData::BitwiseNot)
    }
    pub fn increment() -> Self {
        Self::new(RegularInstructionData::Increment)
    }
    pub fn decrement() -> Self {
        Self::new(RegularInstructionData::Decrement)
    }
    pub fn append_entry() -> Self {
        Self::new(RegularInstructionData::AppendEntry)
    }
    pub fn clear() -> Self {
        Self::new(RegularInstructionData::Clear)
    }
    pub fn splice(
        start_index: u32,
        delete_count: u32,
        insert_count: u32,
    ) -> Self {
        Self::new(RegularInstructionData::Splice(SpliceData {
            start_index,
            delete_count,
            insert_count,
        }))
    }
    pub fn splice_dynamic() -> Self {
        Self::new(RegularInstructionData::SpliceDynamic)
    }
    pub fn set_shared_container_value() -> Self {
        Self::new(RegularInstructionData::SetSharedContainerValue)
    }
    pub fn take_entry_text(key: String) -> Self {
        Self::new(RegularInstructionData::TakeEntryText(ShortTextData(key)))
    }
    pub fn take_entry_index(index: u32) -> Self {
        Self::new(RegularInstructionData::TakeEntryIndex(UInt32Data(index)))
    }
    pub fn take_entry_dynamic() -> Self {
        Self::new(RegularInstructionData::TakeEntryDynamic)
    }
    pub fn set_entry_text(key: String) -> Self {
        Self::new(RegularInstructionData::SetEntryText(ShortTextData(key)))
    }
    pub fn set_entry_index(index: u32) -> Self {
        Self::new(RegularInstructionData::SetEntryIndex(UInt32Data(index)))
    }
    pub fn set_entry_dynamic() -> Self {
        Self::new(RegularInstructionData::SetEntryDynamic)
    }
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        self.data.get_next_expected_instructions()
    }
    pub fn null() -> Self {
        Self::new(RegularInstructionData::Null)
    }
    pub fn r#true() -> Self {
        Self::new(RegularInstructionData::True)
    }
    pub fn r#false() -> Self {
        Self::new(RegularInstructionData::False)
    }
    pub fn set_stack_value(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::SetStackValue(stack_index))
    }
    pub fn borrow_stack_value(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::BorrowStackValue(stack_index))
    }
    pub fn clone_stack_value(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::CloneStackValue(stack_index))
    }
    pub fn key_value_dynamic() -> Self {
        Self::new(RegularInstructionData::KeyValueDynamic)
    }
    pub fn key_value_short_text(key: String) -> Self {
        Self::new(RegularInstructionData::KeyValueShortText(ShortTextData(
            key,
        )))
    }
    pub fn push_to_stack() -> Self {
        Self::new(RegularInstructionData::PushToStack)
    }
    pub fn push_list_to_stack() -> Self {
        Self::new(RegularInstructionData::PushListToStack)
    }
    pub fn get_stack_value_shared_ref(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::GetStackValueSharedRef(stack_index))
    }
    pub fn get_stack_value_shared_ref_mut(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::GetStackValueSharedRefMut(
            stack_index,
        ))
    }
    pub fn take_stack_value(stack_index: StackIndex) -> Self {
        Self::new(RegularInstructionData::TakeStackValue(stack_index))
    }
    pub fn get_root_property(root_property: RootProperty) -> Self {
        Self::new(RegularInstructionData::GetRootProperty(root_property))
    }
    pub fn unbox() -> Self {
        Self::new(RegularInstructionData::Unbox)
    }
    pub fn typed_value() -> Self {
        Self::new(RegularInstructionData::TypedValue)
    }
    pub fn type_expression() -> Self {
        Self::new(RegularInstructionData::TypeExpression)
    }
    pub fn derive_shared_reference() -> Self {
        Self::new(RegularInstructionData::DeriveSharedReference)
    }
    pub fn derive_shared_reference_mut() -> Self {
        Self::new(RegularInstructionData::DeriveSharedReferenceMut)
    }
    pub fn create_shared() -> Self {
        Self::new(RegularInstructionData::CreateShared)
    }
    pub fn create_shared_mut() -> Self {
        Self::new(RegularInstructionData::CreateSharedMut)
    }
    pub fn request_remote_shared_ref(address: RemotePointerAddress) -> Self {
        Self::new(RegularInstructionData::RequestRemoteSharedRef(address))
    }
    pub fn request_remote_shared_ref_mut(
        address: RemotePointerAddress,
    ) -> Self {
        Self::new(RegularInstructionData::RequestRemoteSharedRefMut(address))
    }
    pub fn get_local_shared_ref(address: SelfOwnedPointerAddress) -> Self {
        Self::new(RegularInstructionData::GetLocalSharedRef(address))
    }
    pub fn get_core_lib_value(core_lib_id: CoreLibIdIndex) -> Self {
        Self::new(RegularInstructionData::GetCoreLibValue(core_lib_id))
    }
    pub fn shared_ref(shared_ref: SharedRef) -> Self {
        Self::new(RegularInstructionData::SharedRef(shared_ref))
    }
    pub fn shared_ref_with_value(
        shared_ref_with_value: SharedRefWithValue,
    ) -> Self {
        Self::new(RegularInstructionData::SharedRefWithValue(
            shared_ref_with_value,
        ))
    }
    pub fn move_with_value(move_with_value: MoveWithValue) -> Self {
        Self::new(RegularInstructionData::MoveWithValue(move_with_value))
    }
    pub fn remote_execution(instruction_block: InstructionBlockData) -> Self {
        Self::new(RegularInstructionData::RemoteExecution(instruction_block))
    }
    pub fn range() -> Self {
        Self::new(RegularInstructionData::Range)
    }

    pub fn remote_execution_debug_tree(
        tree: InstructionBlockDataDebugTree,
    ) -> Self {
        Self::new(RegularInstructionData::_RemoteExecutionDebugTree(tree))
    }

    pub fn remote_execution_debug_flat(
        tree: InstructionBlockDataDebugFlat,
    ) -> Self {
        Self::new(RegularInstructionData::_RemoteExecutionDebugFlat(tree))
    }
}

// impl From<&RegularInstruction> for InstructionCode {
//     fn from(instruction: &RegularInstruction) -> Self {
//         instruction.code()
//     }
// }

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, BinRead, BinWrite)]
#[brw(little)]
#[br(import(code: InstructionCode))]
#[br(return_unexpected_error)]
pub enum RegularInstructionData {
    // signed integers
    #[br(pre_assert(code == InstructionCode::INT_8))]
    Int8(Int8Data) = InstructionCode::INT_8.as_u8(),
    #[br(pre_assert(code == InstructionCode::INT_16))]
    Int16(Int16Data) = InstructionCode::INT_16.as_u8(),
    #[br(pre_assert(code == InstructionCode::INT_32))]
    Int32(Int32Data) = InstructionCode::INT_32.as_u8(),
    #[br(pre_assert(code == InstructionCode::INT_64))]
    Int64(Int64Data) = InstructionCode::INT_64.as_u8(),
    #[br(pre_assert(code == InstructionCode::INT_128))]
    Int128(Int128Data) = InstructionCode::INT_128.as_u8(),

    // unsigned integers
    #[br(pre_assert(code == InstructionCode::UINT_8))]
    UInt8(UInt8Data) = InstructionCode::UINT_8.as_u8(),
    #[br(pre_assert(code == InstructionCode::UINT_16))]
    UInt16(UInt16Data) = InstructionCode::UINT_16.as_u8(),
    #[br(pre_assert(code == InstructionCode::UINT_32))]
    UInt32(UInt32Data) = InstructionCode::UINT_32.as_u8(),
    #[br(pre_assert(code == InstructionCode::UINT_64))]
    UInt64(UInt64Data) = InstructionCode::UINT_64.as_u8(),
    #[br(pre_assert(code == InstructionCode::UINT_128))]
    UInt128(UInt128Data) = InstructionCode::UINT_128.as_u8(),

    // big integers
    #[br(pre_assert(code == InstructionCode::INT_BIG))]
    BigInteger(Integer) = InstructionCode::INT_BIG.as_u8(),

    // default integer
    #[br(pre_assert(code == InstructionCode::INT))]
    Integer(Integer) = InstructionCode::INT.as_u8(),
    #[br(pre_assert(code == InstructionCode::RANGE))]
    Range = InstructionCode::RANGE.as_u8(),

    #[br(pre_assert(code == InstructionCode::ENDPOINT))]
    Endpoint(Endpoint) = InstructionCode::ENDPOINT.as_u8(),

    #[br(pre_assert(code == InstructionCode::INSTANT))]
    Instant(InstantData) = InstructionCode::INSTANT.as_u8(),

    #[br(pre_assert(code == InstructionCode::DECIMAL_F32))]
    DecimalF32(Float32Data) = InstructionCode::DECIMAL_F32.as_u8(),
    #[br(pre_assert(code == InstructionCode::DECIMAL_F64))]
    DecimalF64(Float64Data) = InstructionCode::DECIMAL_F64.as_u8(),
    #[br(pre_assert(code == InstructionCode::DECIMAL_AS_INT_16))]
    DecimalAsInt16(FloatAsInt16Data) = InstructionCode::DECIMAL_AS_INT_16.as_u8(),
    #[br(pre_assert(code == InstructionCode::DECIMAL_AS_INT_32))]
    DecimalAsInt32(FloatAsInt32Data) = InstructionCode::DECIMAL_AS_INT_32.as_u8(),
    #[br(pre_assert(code == InstructionCode::DECIMAL_BIG))]
    BigDecimal(Decimal) = InstructionCode::DECIMAL_BIG.as_u8(),
    // default decimal
    #[br(pre_assert(code == InstructionCode::DECIMAL))]
    Decimal(Decimal) = InstructionCode::DECIMAL.as_u8(),

    #[br(pre_assert(code == InstructionCode::REMOTE_EXECUTION))]
    RemoteExecution(InstructionBlockData) = InstructionCode::REMOTE_EXECUTION.as_u8(),
    #[br(pre_assert(code == InstructionCode::SHORT_TEXT))]
    ShortText(ShortTextData) = InstructionCode::SHORT_TEXT.as_u8(),
    #[br(pre_assert(code == InstructionCode::TEXT))]
    Text(TextData) = InstructionCode::TEXT.as_u8(),

    #[br(pre_assert(code == InstructionCode::TRUE))]
    True = InstructionCode::TRUE.as_u8(),
    #[br(pre_assert(code == InstructionCode::FALSE))]
    False = InstructionCode::FALSE.as_u8(),
    #[br(pre_assert(code == InstructionCode::NULL))]
    Null = InstructionCode::NULL.as_u8(),
    #[br(pre_assert(code == InstructionCode::STATEMENTS))]
    Statements(StatementsData) = InstructionCode::STATEMENTS.as_u8(),
    #[br(pre_assert(code == InstructionCode::SHORT_STATEMENTS))]
    ShortStatements(ShortStatementsData) = InstructionCode::SHORT_STATEMENTS.as_u8(),
    #[br(pre_assert(code == InstructionCode::UNBOUNDED_STATEMENTS))]
    UnboundedStatements = InstructionCode::UNBOUNDED_STATEMENTS.as_u8(),
    #[br(pre_assert(code == InstructionCode::UNBOUNDED_STATEMENTS_END))]
    UnboundedStatementsEnd(UnboundedStatementsData) = InstructionCode::UNBOUNDED_STATEMENTS_END.as_u8(),
    #[br(pre_assert(code == InstructionCode::LIST))]
    List(ListData) = InstructionCode::LIST.as_u8(),
    #[br(pre_assert(code == InstructionCode::SHORT_LIST))]
    ShortList(ShortListData) = InstructionCode::SHORT_LIST.as_u8(),
    #[br(pre_assert(code == InstructionCode::MAP))]
    Map(MapData) = InstructionCode::MAP.as_u8(),
    #[br(pre_assert(code == InstructionCode::SHORT_MAP))]
    ShortMap(ShortMapData) = InstructionCode::SHORT_MAP.as_u8(),

    #[br(pre_assert(code == InstructionCode::KEY_VALUE_DYNAMIC))]
    KeyValueDynamic = InstructionCode::KEY_VALUE_DYNAMIC.as_u8(),
    #[br(pre_assert(code == InstructionCode::KEY_VALUE_SHORT_TEXT))]
    KeyValueShortText(ShortTextData) = InstructionCode::KEY_VALUE_SHORT_TEXT.as_u8(),

    #[br(pre_assert(code == InstructionCode::TAGGED_VALUE))]
    TaggedValue(TaggedValue) = InstructionCode::TAGGED_VALUE.as_u8(),

    // binary operator
    #[br(pre_assert(code == InstructionCode::ADD))]
    Add = InstructionCode::ADD.as_u8(),
    #[br(pre_assert(code == InstructionCode::SUBTRACT))]
    Subtract = InstructionCode::SUBTRACT.as_u8(),
    #[br(pre_assert(code == InstructionCode::MULTIPLY))]
    Multiply = InstructionCode::MULTIPLY.as_u8(),
    #[br(pre_assert(code == InstructionCode::DIVIDE))]
    Divide = InstructionCode::DIVIDE.as_u8(),

    // unary operator
    // TODO #432 add missing unary operators
    #[br(pre_assert(code == InstructionCode::UNARY_MINUS))]
    UnaryMinus = InstructionCode::UNARY_MINUS.as_u8(),
    // TODO #433: Do we need this for op overloading or can we avoid?
    #[br(pre_assert(code == InstructionCode::UNARY_PLUS))]
    UnaryPlus = InstructionCode::UNARY_PLUS.as_u8(),
    #[br(pre_assert(code == InstructionCode::BITWISE_NOT))]
    BitwiseNot = InstructionCode::BITWISE_NOT.as_u8(),

    #[br(pre_assert(code == InstructionCode::APPLY_ZERO))]
    Apply(ApplyData) = InstructionCode::APPLY_ZERO.as_u8(),

    #[br(pre_assert(code == InstructionCode::GET_PROPERTY_TEXT))]
    GetPropertyText(ShortTextData) = InstructionCode::GET_PROPERTY_TEXT.as_u8(),

    #[br(pre_assert(code == InstructionCode::GET_PROPERTY_INDEX))]
    GetPropertyIndex(UInt32Data) = InstructionCode::GET_PROPERTY_INDEX.as_u8(),

    #[br(pre_assert(code == InstructionCode::GET_PROPERTY_DYNAMIC))]
    GetPropertyDynamic = InstructionCode::GET_PROPERTY_DYNAMIC.as_u8(),

    // comparison operator
    #[br(pre_assert(code == InstructionCode::IS))]
    Is = InstructionCode::IS.as_u8(),
    #[br(pre_assert(code == InstructionCode::MATCHES))]
    Matches = InstructionCode::MATCHES.as_u8(),
    #[br(pre_assert(code == InstructionCode::STRUCTURAL_EQUAL))]
    StructuralEqual = InstructionCode::STRUCTURAL_EQUAL.as_u8(),
    #[br(pre_assert(code == InstructionCode::EQUAL))]
    Equal = InstructionCode::EQUAL.as_u8(),
    #[br(pre_assert(code == InstructionCode::NOT_STRUCTURAL_EQUAL))]
    NotStructuralEqual = InstructionCode::NOT_STRUCTURAL_EQUAL.as_u8(),
    #[br(pre_assert(code == InstructionCode::NOT_EQUAL))]
    NotEqual = InstructionCode::NOT_EQUAL.as_u8(),

    #[br(pre_assert(code == InstructionCode::DERIVE_SHARED_REF))]
    DeriveSharedReference = InstructionCode::DERIVE_SHARED_REF.as_u8(),
    #[br(pre_assert(code == InstructionCode::DERIVE_SHARED_REF_MUT))]
    DeriveSharedReferenceMut = InstructionCode::DERIVE_SHARED_REF_MUT.as_u8(),

    #[br(pre_assert(code == InstructionCode::CREATE_SHARED))]
    CreateShared = InstructionCode::CREATE_SHARED.as_u8(),
    #[br(pre_assert(code == InstructionCode::CREATE_SHARED_MUT))]
    CreateSharedMut = InstructionCode::CREATE_SHARED_MUT.as_u8(),

    // ' $ABCDE
    #[br(pre_assert(code == InstructionCode::REQUEST_REMOTE_SHARED_REF))]
    RequestRemoteSharedRef(RemotePointerAddress) = InstructionCode::REQUEST_REMOTE_SHARED_REF.as_u8(),
    // 'mut $ABCDE
    #[br(pre_assert(code == InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT))]
    RequestRemoteSharedRefMut(RemotePointerAddress) = InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT.as_u8(),
    #[br(pre_assert(code == InstructionCode::GET_LOCAL_SHARED_REF))]
    GetLocalSharedRef(SelfOwnedPointerAddress) = InstructionCode::GET_LOCAL_SHARED_REF.as_u8(),
    // get a core lib value, e.g. integer or print by id
    #[br(pre_assert(code == InstructionCode::GET_CORE_LIB_VALUE))]
    GetCoreLibValue(CoreLibIdIndex) = InstructionCode::GET_CORE_LIB_VALUE.as_u8(),

    #[br(pre_assert(code == InstructionCode::SHARED_REF))]
    SharedRef(SharedRef) = InstructionCode::SHARED_REF.as_u8(),
    #[br(pre_assert(code == InstructionCode::SHARED_REF_WITH_VALUE))]
    SharedRefWithValue(SharedRefWithValue) = InstructionCode::SHARED_REF_WITH_VALUE.as_u8(), // shared ref with current value (only if caller owns the pointer)

    #[br(pre_assert(code == InstructionCode::MOVE_WITH_VALUE))]
    MoveWithValue(MoveWithValue) = InstructionCode::MOVE_WITH_VALUE.as_u8(),

    #[br(pre_assert(code == InstructionCode::PUSH_TO_STACK))]
    PushToStack = InstructionCode::PUSH_TO_STACK.as_u8(),
    #[br(pre_assert(code == InstructionCode::PUSH_LIST_TO_STACK))]
    PushListToStack = InstructionCode::PUSH_LIST_TO_STACK.as_u8(),
    #[br(pre_assert(code == InstructionCode::CLONE_STACK_VALUE))]
    CloneStackValue(StackIndex) = InstructionCode::CLONE_STACK_VALUE.as_u8(),
    #[br(pre_assert(code == InstructionCode::BORROW_STACK_VALUE))]
    BorrowStackValue(StackIndex) = InstructionCode::BORROW_STACK_VALUE.as_u8(),
    #[br(pre_assert(code == InstructionCode::GET_STACK_VALUE_SHARED_REF))]
    GetStackValueSharedRef(StackIndex) = InstructionCode::GET_STACK_VALUE_SHARED_REF.as_u8(),
    #[br(pre_assert(code == InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT))]
    GetStackValueSharedRefMut(StackIndex) = InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT.as_u8(),
    #[br(pre_assert(code == InstructionCode::TAKE_STACK_VALUE))]
    TakeStackValue(StackIndex) = InstructionCode::TAKE_STACK_VALUE.as_u8(),
    #[br(pre_assert(code == InstructionCode::SET_STACK_VALUE))]
    SetStackValue(StackIndex) = InstructionCode::SET_STACK_VALUE.as_u8(),

    #[br(pre_assert(code == InstructionCode::GET_ROOT_PROPERTY))]
    GetRootProperty(RootProperty) = InstructionCode::GET_ROOT_PROPERTY.as_u8(),

    #[br(pre_assert(code == InstructionCode::UNBOX))]
    Unbox = InstructionCode::UNBOX.as_u8(),

    #[br(pre_assert(code == InstructionCode::TYPED_VALUE))]
    TypedValue = InstructionCode::TYPED_VALUE.as_u8(),
    #[br(pre_assert(code == InstructionCode::TYPE_EXPRESSION))]
    TypeExpression = InstructionCode::TYPE_EXPRESSION.as_u8(),

    // modification instructions: will later be mapped to trait impls
    // UpdateOperation::Replace
    #[br(pre_assert(code == InstructionCode::SET_SHARED_CONTAINER_VALUE))]
    SetSharedContainerValue = InstructionCode::SET_SHARED_CONTAINER_VALUE.as_u8(),

    // UpdateOperation::AppendEntry
    #[br(pre_assert(code == InstructionCode::APPEND_ENTRY))]
    AppendEntry = InstructionCode::APPEND_ENTRY.as_u8(),
    // UpdateOperation::Clear
    #[br(pre_assert(code == InstructionCode::CLEAR))]
    Clear = InstructionCode::CLEAR.as_u8(),
    // UpdateOperation::Splice
    #[br(pre_assert(code == InstructionCode::SPLICE))]
    Splice(SpliceData) = InstructionCode::SPLICE.as_u8(),
    #[br(pre_assert(code == InstructionCode::SPLICE_DYNAMIC))]
    SpliceDynamic = InstructionCode::SPLICE_DYNAMIC.as_u8(),

    // UpdateOperation::Increment
    #[br(pre_assert(code == InstructionCode::INCREMENT))]
    Increment = InstructionCode::INCREMENT.as_u8(),
    // UpdateOperation::Decrement
    #[br(pre_assert(code == InstructionCode::DECREMENT))]
    Decrement = InstructionCode::DECREMENT.as_u8(),

    // UpdateOperation::DeleteEntry
    #[br(pre_assert(code == InstructionCode::TAKE_PROPERTY_TEXT))]
    TakeEntryText(ShortTextData) = InstructionCode::TAKE_PROPERTY_TEXT.as_u8(),
    #[br(pre_assert(code == InstructionCode::TAKE_PROPERTY_INDEX))]
    TakeEntryIndex(UInt32Data) = InstructionCode::TAKE_PROPERTY_INDEX.as_u8(),
    #[br(pre_assert(code == InstructionCode::TAKE_PROPERTY_DYNAMIC))]
    TakeEntryDynamic = InstructionCode::TAKE_PROPERTY_DYNAMIC.as_u8(),

    // UpdateOperation::SetEntry
    #[br(pre_assert(code == InstructionCode::SET_PROPERTY_TEXT))]
    SetEntryText(ShortTextData) = InstructionCode::SET_PROPERTY_TEXT.as_u8(),
    #[br(pre_assert(code == InstructionCode::SET_PROPERTY_INDEX))]
    SetEntryIndex(UInt32Data) = InstructionCode::SET_PROPERTY_INDEX.as_u8(),
    #[br(pre_assert(code == InstructionCode::SET_PROPERTY_DYNAMIC))]
    SetEntryDynamic = InstructionCode::SET_PROPERTY_DYNAMIC.as_u8(),

    /// Debug variant for RemoteExecution, includes full remote execution instruction list (flat) instead of raw dxb
    /// This variant is only used by the disassembler
    #[cfg(feature = "disassembler")]
    _RemoteExecutionDebugFlat(#[brw(ignore)] crate::global::protocol_structures::instruction_data::InstructionBlockDataDebugFlat) = 253,
    /// Debug variant for RemoteExecution, includes full remote execution instruction tree instead of raw dxb
    /// This variant is only used by the disassembler
    #[cfg(feature = "disassembler")]
    _RemoteExecutionDebugTree(#[brw(ignore)] crate::global::protocol_structures::instruction_data::InstructionBlockDataDebugTree) = 254,
}

impl RegularInstructionData {
    #[inline]
    pub fn instruction_code(&self) -> InstructionCode {
        // SAFETY:
        //
        // RegularInstructionData has #[repr(u8)], so we can guarantee
        // that its discriminant can be read as a u8 from the addr
        let raw = unsafe { *(self as *const Self).cast::<u8>() };

        InstructionCode::try_from(raw).unwrap_or_else(|_| {
            panic!("Invalid instruction code for RegularInstructionData: {raw}")
        })
    }
}

impl RegularInstructionData {
    pub fn statements(count: u32, terminated: bool) -> RegularInstructionData {
        match count {
            0..=255 => {
                RegularInstructionData::ShortStatements(ShortStatementsData {
                    statements_count: count as u8,
                    terminated,
                })
            }
            _ => RegularInstructionData::Statements(StatementsData {
                statements_count: count,
                terminated,
            }),
        }
    }

    pub fn list(count: u32) -> RegularInstructionData {
        match count {
            0..=255 => RegularInstructionData::ShortList(ShortListData {
                element_count: count as u8,
            }),
            _ => RegularInstructionData::List(ListData {
                element_count: count,
            }),
        }
    }
}

// Maps each regular instruction to its corresponding instruction code
// impl From<&RegularInstructionData> for InstructionCode {
//     fn from(instruction: &RegularInstructionData) -> Self {
//         match instruction {
//             RegularInstructionData::Int8(_) => InstructionCode::INT_8,
//             RegularInstructionData::Int16(_) => InstructionCode::INT_16,
//             RegularInstructionData::Int32(_) => InstructionCode::INT_32,
//             RegularInstructionData::Int64(_) => InstructionCode::INT_64,
//             RegularInstructionData::Int128(_) => InstructionCode::INT_128,
//             RegularInstructionData::UInt8(_) => InstructionCode::UINT_8,
//             RegularInstructionData::UInt16(_) => InstructionCode::UINT_16,
//             RegularInstructionData::UInt32(_) => InstructionCode::UINT_32,
//             RegularInstructionData::UInt64(_) => InstructionCode::UINT_64,
//             RegularInstructionData::UInt128(_) => InstructionCode::UINT_128,
//             RegularInstructionData::BigInteger(_) => InstructionCode::INT_BIG,
//             RegularInstructionData::Integer(_) => InstructionCode::INT,
//             RegularInstructionData::Endpoint(_) => InstructionCode::ENDPOINT,
//             RegularInstructionData::Instant(_) => InstructionCode::INSTANT,
//             RegularInstructionData::DecimalF32(_) => {
//                 InstructionCode::DECIMAL_F32
//             }
//             RegularInstructionData::DecimalF64(_) => {
//                 InstructionCode::DECIMAL_F64
//             }
//             RegularInstructionData::DecimalAsInt16(_) => {
//                 InstructionCode::DECIMAL_AS_INT_16
//             }
//             RegularInstructionData::DecimalAsInt32(_) => {
//                 InstructionCode::DECIMAL_AS_INT_32
//             }
//             RegularInstructionData::BigDecimal(_) => {
//                 InstructionCode::DECIMAL_BIG
//             }
//             RegularInstructionData::Decimal(_) => InstructionCode::DECIMAL,
//             RegularInstructionData::Range => InstructionCode::RANGE,
//             RegularInstructionData::RemoteExecution(_) => {
//                 InstructionCode::REMOTE_EXECUTION
//             }
//             RegularInstructionData::ShortText(_) => InstructionCode::SHORT_TEXT,
//             RegularInstructionData::Text(_) => InstructionCode::TEXT,
//             RegularInstructionData::True => InstructionCode::TRUE,
//             RegularInstructionData::False => InstructionCode::FALSE,
//             RegularInstructionData::Null => InstructionCode::NULL,
//             RegularInstructionData::Statements(_) => {
//                 InstructionCode::STATEMENTS
//             }
//             RegularInstructionData::ShortStatements(_) => {
//                 InstructionCode::SHORT_STATEMENTS
//             }
//             RegularInstructionData::UnboundedStatements => {
//                 InstructionCode::UNBOUNDED_STATEMENTS
//             }
//             RegularInstructionData::UnboundedStatementsEnd(_) => {
//                 InstructionCode::UNBOUNDED_STATEMENTS_END
//             }
//             RegularInstructionData::List(_) => InstructionCode::LIST,
//             RegularInstructionData::ShortList(_) => InstructionCode::SHORT_LIST,
//             RegularInstructionData::Map(_) => InstructionCode::MAP,
//             RegularInstructionData::ShortMap(_) => InstructionCode::SHORT_MAP,
//             RegularInstructionData::KeyValueDynamic => {
//                 InstructionCode::KEY_VALUE_DYNAMIC
//             }
//             RegularInstructionData::KeyValueShortText(_) => {
//                 InstructionCode::KEY_VALUE_SHORT_TEXT
//             }
//             RegularInstructionData::Add => InstructionCode::ADD,
//             RegularInstructionData::Subtract => InstructionCode::SUBTRACT,
//             RegularInstructionData::Multiply => InstructionCode::MULTIPLY,
//             RegularInstructionData::Divide => InstructionCode::DIVIDE,
//             RegularInstructionData::UnaryMinus => InstructionCode::UNARY_MINUS,
//             RegularInstructionData::UnaryPlus => InstructionCode::UNARY_PLUS,
//             RegularInstructionData::BitwiseNot => InstructionCode::BITWISE_NOT,
//             RegularInstructionData::Apply(_) => InstructionCode::APPLY,
//             RegularInstructionData::GetPropertyText(_) => {
//                 InstructionCode::GET_PROPERTY_TEXT
//             }
//             RegularInstructionData::SetEntryText(_) => {
//                 InstructionCode::SET_PROPERTY_TEXT
//             }
//             RegularInstructionData::TakeEntryText(_) => {
//                 InstructionCode::TAKE_PROPERTY_TEXT
//             }
//             RegularInstructionData::GetPropertyIndex(_) => {
//                 InstructionCode::GET_PROPERTY_INDEX
//             }
//             RegularInstructionData::SetEntryIndex(_) => {
//                 InstructionCode::SET_PROPERTY_INDEX
//             }
//             RegularInstructionData::TakeEntryIndex(_) => {
//                 InstructionCode::TAKE_PROPERTY_INDEX
//             }
//             RegularInstructionData::GetPropertyDynamic => {
//                 InstructionCode::GET_PROPERTY_DYNAMIC
//             }
//             RegularInstructionData::SetEntryDynamic => {
//                 InstructionCode::SET_PROPERTY_DYNAMIC
//             }
//             RegularInstructionData::TakeEntryDynamic => {
//                 InstructionCode::TAKE_PROPERTY_DYNAMIC
//             }
//             RegularInstructionData::Is => InstructionCode::IS,
//             RegularInstructionData::Matches => InstructionCode::MATCHES,
//             RegularInstructionData::StructuralEqual => {
//                 InstructionCode::STRUCTURAL_EQUAL
//             }
//             RegularInstructionData::Equal => InstructionCode::EQUAL,
//             RegularInstructionData::NotStructuralEqual => {
//                 InstructionCode::NOT_STRUCTURAL_EQUAL
//             }
//             RegularInstructionData::NotEqual => InstructionCode::NOT_EQUAL,
//             RegularInstructionData::DeriveSharedReference => {
//                 InstructionCode::DERIVE_SHARED_REF
//             }
//             RegularInstructionData::DeriveSharedReferenceMut => {
//                 InstructionCode::DERIVE_SHARED_REF_MUT
//             }
//             RegularInstructionData::CreateShared => {
//                 InstructionCode::CREATE_SHARED
//             }
//             RegularInstructionData::CreateSharedMut => {
//                 InstructionCode::CREATE_SHARED_MUT
//             }
//             RegularInstructionData::RequestRemoteSharedRef(_) => {
//                 InstructionCode::REQUEST_REMOTE_SHARED_REF
//             }
//             RegularInstructionData::RequestRemoteSharedRefMut(_) => {
//                 InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT
//             }
//             RegularInstructionData::GetLocalSharedRef(_) => {
//                 InstructionCode::GET_LOCAL_SHARED_REF
//             }
//             RegularInstructionData::GetCoreLibValue(_) => {
//                 InstructionCode::GET_CORE_LIB_VALUE
//             }
//             RegularInstructionData::SharedRef(_) => InstructionCode::SHARED_REF,
//             RegularInstructionData::SharedRefWithValue(_) => {
//                 InstructionCode::SHARED_REF_WITH_VALUE
//             }
//             RegularInstructionData::MoveWithValue(_) => {
//                 InstructionCode::MOVE_WITH_VALUE
//             }
//             RegularInstructionData::PushToStack => {
//                 InstructionCode::PUSH_TO_STACK
//             }
//             RegularInstructionData::PushListToStack => {
//                 InstructionCode::PUSH_LIST_TO_STACK
//             }
//             RegularInstructionData::CloneStackValue(_) => {
//                 InstructionCode::CLONE_STACK_VALUE
//             }
//             RegularInstructionData::BorrowStackValue(_) => {
//                 InstructionCode::BORROW_STACK_VALUE
//             }
//             RegularInstructionData::GetStackValueSharedRef(_) => {
//                 InstructionCode::GET_STACK_VALUE_SHARED_REF
//             }
//             RegularInstructionData::GetStackValueSharedRefMut(_) => {
//                 InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT
//             }
//             RegularInstructionData::TakeStackValue(_) => {
//                 InstructionCode::TAKE_STACK_VALUE
//             }
//             RegularInstructionData::SetStackValue(_) => {
//                 InstructionCode::SET_STACK_VALUE
//             }
//             RegularInstructionData::GetRootProperty(_) => {
//                 InstructionCode::GET_ROOT_PROPERTY
//             }
//             RegularInstructionData::SetSharedContainerValue => {
//                 InstructionCode::SET_SHARED_CONTAINER_VALUE
//             }
//             RegularInstructionData::Unbox => InstructionCode::UNBOX,
//             RegularInstructionData::TypedValue => InstructionCode::TYPED_VALUE,
//             RegularInstructionData::TypeExpression => {
//                 InstructionCode::TYPE_EXPRESSION
//             }
//             RegularInstructionData::TaggedValue(_) => {
//                 InstructionCode::TAGGED_VALUE
//             }
//             #[cfg(feature = "disassembler")]
//             RegularInstructionData::_RemoteExecutionDebugFlat(_)
//             | RegularInstructionData::_RemoteExecutionDebugTree(_) => {
//                 InstructionCode::REMOTE_EXECUTION
//             }
//             RegularInstructionData::AppendEntry => {
//                 InstructionCode::APPEND_ENTRY
//             }
//             RegularInstructionData::Clear => InstructionCode::CLEAR,
//             RegularInstructionData::Splice(_) => InstructionCode::SPLICE,
//             RegularInstructionData::SpliceDynamic => {
//                 InstructionCode::SPLICE_DYNAMIC
//             }
//             RegularInstructionData::Increment => InstructionCode::INCREMENT,
//             RegularInstructionData::Decrement => InstructionCode::DECREMENT,
//         }
//     }
// }

impl RegularInstructionData {
    /// Returns how many (if any) regular or type instructions are expected as child instructions for a given instructions
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        match self {
            RegularInstructionData::RemoteExecution(_) => {
                NextExpectedInstructions::Regular(1)
            } // receivers

            #[cfg(feature = "disassembler")]
            RegularInstructionData::_RemoteExecutionDebugTree(_)
            | RegularInstructionData::_RemoteExecutionDebugFlat(_) => {
                NextExpectedInstructions::Regular(1)
            } // receivers

            RegularInstructionData::ShortList(list) => {
                NextExpectedInstructions::Regular(list.element_count as u32)
            } // list elements

            RegularInstructionData::List(list) => {
                NextExpectedInstructions::Regular(list.element_count)
            } // list elements

            RegularInstructionData::ShortMap(map) => {
                NextExpectedInstructions::Regular(map.element_count as u32)
            } // map entries

            RegularInstructionData::Map(map) => {
                NextExpectedInstructions::Regular(map.element_count)
            } // map entries

            RegularInstructionData::ShortStatements(statements) => {
                NextExpectedInstructions::Regular(
                    statements.statements_count as u32,
                )
            }
            RegularInstructionData::Statements(statements) => {
                NextExpectedInstructions::Regular(statements.statements_count)
            } // statements in block

            RegularInstructionData::UnboundedStatements => {
                NextExpectedInstructions::UnboundedStart
            }

            RegularInstructionData::UnboundedStatementsEnd(_) => {
                NextExpectedInstructions::UnboundedEnd
            }

            RegularInstructionData::Apply(apply_data) => {
                NextExpectedInstructions::Regular(
                    apply_data.arg_count as u32 + 1,
                )
            } // arguments plus base to apply to

            RegularInstructionData::GetPropertyText(_)
            | RegularInstructionData::GetPropertyIndex(_)
            | RegularInstructionData::TakeEntryText(_)
            | RegularInstructionData::TakeEntryIndex(_) => {
                NextExpectedInstructions::Regular(1)
            } // value to get property from

            RegularInstructionData::GetPropertyDynamic
            | RegularInstructionData::TakeEntryDynamic => {
                NextExpectedInstructions::Regular(2)
            } // value to get property from + property key

            RegularInstructionData::SetEntryText(_)
            | RegularInstructionData::SetEntryIndex(_) => {
                NextExpectedInstructions::Regular(2)
            } // value to set property on and new value

            RegularInstructionData::SetEntryDynamic => {
                NextExpectedInstructions::Regular(3)
            } // value to set property on + property key + new value

            RegularInstructionData::Unbox => {
                NextExpectedInstructions::Regular(1)
            } // value to unbox

            RegularInstructionData::AppendEntry => {
                NextExpectedInstructions::Regular(2)
            }
            RegularInstructionData::Splice(SpliceData {
                insert_count, ..
            }) => NextExpectedInstructions::Regular(*insert_count + 1),
            RegularInstructionData::SpliceDynamic => {
                NextExpectedInstructions::Regular(4)
            }

            RegularInstructionData::SetSharedContainerValue => {
                NextExpectedInstructions::Regular(2)
            } // container to set value on + new value

            RegularInstructionData::KeyValueDynamic => {
                NextExpectedInstructions::Regular(2)
            } // key + value

            RegularInstructionData::KeyValueShortText(_) => {
                NextExpectedInstructions::Regular(1)
            } // value

            RegularInstructionData::Matches => {
                NextExpectedInstructions::RegularAndType(1, 1)
            }

            RegularInstructionData::Add
            | RegularInstructionData::Multiply
            | RegularInstructionData::Subtract
            | RegularInstructionData::Divide => {
                NextExpectedInstructions::Regular(2)
            } // left and right operand

            RegularInstructionData::StructuralEqual
            | RegularInstructionData::NotStructuralEqual
            | RegularInstructionData::Equal
            | RegularInstructionData::NotEqual
            | RegularInstructionData::Is => {
                NextExpectedInstructions::Regular(2)
            } // left and right operand

            RegularInstructionData::UnaryMinus
            | RegularInstructionData::UnaryPlus
            | RegularInstructionData::BitwiseNot => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstructionData::DeriveSharedReference
            | RegularInstructionData::DeriveSharedReferenceMut
            | RegularInstructionData::CreateShared
            | RegularInstructionData::CreateSharedMut => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstructionData::PushToStack
            | RegularInstructionData::PushListToStack
            | RegularInstructionData::SetStackValue(_) => {
                NextExpectedInstructions::Regular(1)
            }
            RegularInstructionData::TypedValue => {
                NextExpectedInstructions::RegularAndType(1, 1)
            }

            RegularInstructionData::TypeExpression => {
                NextExpectedInstructions::Type(1)
            }

            RegularInstructionData::Range => {
                NextExpectedInstructions::Regular(2)
            }
            RegularInstructionData::TaggedValue(TaggedValue {
                is_empty,
                ..
            }) => {
                if *is_empty {
                    NextExpectedInstructions::None
                } else {
                    NextExpectedInstructions::Regular(1)
                }
            }

            RegularInstructionData::SharedRefWithValue(_) => {
                NextExpectedInstructions::Regular(1)
            }
            RegularInstructionData::MoveWithValue(_) => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstructionData::Increment => {
                NextExpectedInstructions::Regular(2)
            }
            RegularInstructionData::Decrement => {
                NextExpectedInstructions::Regular(2)
            }

            _ => NextExpectedInstructions::None,
        }
    }

    fn read_regular_instruction_code<R: Read + Seek>(
        mut reader: &mut R,
    ) -> Result<InstructionCode, DXBParserError> {
        let instruction_code = u8::read(&mut reader)
            .map_err(|_| DXBParserError::FailedToReadInstructionCode)?;

        InstructionCode::try_from(instruction_code).map_err(|_| {
            DXBParserError::InvalidInstructionCode(instruction_code)
        })
    }

    pub fn metadata_string(&self) -> Option<String> {
        let mut string = String::new();

        match self {
            RegularInstructionData::Int8(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Int16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Int32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Int64(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Int128(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::UInt8(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::UInt16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::UInt32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::UInt64(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::UInt128(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Apply(count) => {
                write!(string, "[arg_count: {}]", count.arg_count)
            }
            RegularInstructionData::BigInteger(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Integer(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Endpoint(data) => {
                write!(string, "{data}")
            }
            RegularInstructionData::Instant(data) => {
                write!(string, "{}", Instant(data.0).to_iso_string())
            }

            RegularInstructionData::DecimalAsInt16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::DecimalAsInt32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::DecimalF32(data) => {
                write!(
                    string,
                    "{}",
                    TypedDecimal::F32(data.0.into())
                )
            }
            RegularInstructionData::DecimalF64(data) => {
                write!(
                    string,
                    "{}",
                    TypedDecimal::F64(data.0.into())
                )
            }
            RegularInstructionData::BigDecimal(data) => {
                write!(string, "{}", data)
            }
            RegularInstructionData::Decimal(data) => {
                write!(string, "{}", data)
            }
            RegularInstructionData::ShortText(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Text(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::Statements(data) => {
                write!(string, "[count: {}, terminated: {}]", data.statements_count, data.terminated)
            }
            RegularInstructionData::ShortStatements(data) => {
                write!(string, "[count: {}, terminated: {}]", data.statements_count, data.terminated)
            }
            RegularInstructionData::List(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstructionData::ShortList(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstructionData::Map(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstructionData::ShortMap(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstructionData::KeyValueShortText(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstructionData::CloneStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::GetRootProperty(property) => {
                write!(string, "$.{}", property)
            }
            RegularInstructionData::BorrowStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::GetStackValueSharedRef(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::GetStackValueSharedRefMut(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::TakeStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::SetStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstructionData::Splice(splice_data) => {
                write!(string, "[start: {}, delete_count: {}, insert_count: {}]", splice_data.start_index, splice_data.delete_count, splice_data.insert_count)
            }
            RegularInstructionData::RequestRemoteSharedRef(address) => {
                write!(
                    string,
                    "[endpoint: {}, address:{}]",
                    address.endpoint(),
                    address
                )
            }
            RegularInstructionData::RequestRemoteSharedRefMut(address) => {
                write!(
                    string,
                    "[endpoint: {}, address:{}]",
                    address.endpoint(),
                    address
                )
            }
            RegularInstructionData::GetLocalSharedRef(address) => {
                write!(
                    string,
                    "[address:{}]",
                    address
                )
            }
            RegularInstructionData::GetCoreLibValue(address) => {
                write!(
                    string,
                    "[id: {:4x}]",
                    &address.0
                )
            }
            RegularInstructionData::SharedRef(shared_ref) => {
                write!(
                    string,
                    "[ref_mutability: {:?}, address: {}]",
                    shared_ref.ref_mutability, shared_ref.address.clone()
                )
            }
            RegularInstructionData::SharedRefWithValue(shared_ref) => {
                write!(
                    string,
                    "[ref_mutability: {:?}, address: {}, container_mutability: {:?}]",
                    shared_ref.ref_mutability,
                    PointerAddress::from(shared_ref.address.clone()),
                    shared_ref.container_mutability
                )
            }
            RegularInstructionData::MoveWithValue(move_with_value) => {
                write!(
                    string,
                    "[mutability: {:?}, previous address: {}]",
                    move_with_value.mutability,
                    move_with_value.previous_address
                )
            }
            RegularInstructionData::RemoteExecution(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            #[cfg(feature = "disassembler")]
            RegularInstructionData::_RemoteExecutionDebugTree(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            #[cfg(feature = "disassembler")]
            RegularInstructionData::_RemoteExecutionDebugFlat(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            RegularInstructionData::GetPropertyIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstructionData::SetEntryIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstructionData::TakeEntryIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstructionData::GetPropertyText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstructionData::TakeEntryText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstructionData::SetEntryText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            _ => {
                // no custom disassembly
                return None;
            }
        }.unwrap();

        Some(string)
    }

    #[cfg(feature = "disassembler")]
    pub fn inner_instructions(&self) -> InnerInstructions<'_> {
        match self {
            RegularInstructionData::_RemoteExecutionDebugTree(data) => {
                InnerInstructions::Tree(&data.body)
            }
            RegularInstructionData::_RemoteExecutionDebugFlat(data) => {
                InnerInstructions::Flat(&data.body)
            }
            _ => InnerInstructions::None,
        }
    }
}

/// Serializes RegularInstruction to tuple (instruction code as string, optional metadata as string)
#[cfg(feature = "disassembler")]
use serde::{Serialize, Serializer, ser::SerializeTuple};
#[cfg(feature = "disassembler")]
impl Serialize for RegularInstructionData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let instruction_code = self.instruction_code().to_string();
        let metadata_string = self.metadata_string();

        if let Some(metadata_string) = metadata_string {
            let inner_instructions = self.inner_instructions();
            let count = if inner_instructions == InnerInstructions::None {
                2
            } else {
                3
            };

            let mut state = serializer.serialize_tuple(count)?;
            state.serialize_element(&instruction_code)?;
            state.serialize_element(&metadata_string)?;
            match inner_instructions {
                InnerInstructions::Tree(data) => {
                    state.serialize_element(&data)?
                }
                InnerInstructions::Flat(data) => {
                    state.serialize_element(&data)?
                }
                InnerInstructions::None => {}
            }
            state.end()
        } else {
            serializer.serialize_str(&instruction_code)
        }
    }
}

// impl BinRead for RegularInstructionData {
//     type Args<'a> = ();

//     fn read_options<R: Read + Seek>(
//         reader: &mut R,
//         _endian: Endian,
//         _: Self::Args<'_>,
//     ) -> BinResult<Self> {
//         let instruction_code =
//             RegularInstructionData::read_regular_instruction_code(reader)
//                 .map_err(|e| binrw::Error::AssertFail {
//                     pos: reader.stream_position().unwrap_or(0),
//                     message: e.to_string(),
//                 })?;
//         RegularInstructionData::read_instruction(reader, instruction_code)
//     }
// }

// impl ReadEndian for RegularInstructionData {
//     const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
// }

impl Display for RegularInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.code())?;

        if let Some(metadata_string) = self.data.metadata_string() {
            write!(f, " {}", metadata_string)?;
        }

        Ok(())
    }
}
impl Display for RegularInstructionData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(metadata_string) = self.metadata_string() {
            write!(f, "{}", metadata_string)?;
        }

        Ok(())
    }
}
