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
    BinRead, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
    meta::{EndianKind, ReadEndian},
};
use core::fmt::{Display, Write as FmtWrite};

impl RegularInstruction {
    pub fn int8(value: i8) -> Self {
        RegularInstruction::Int8(Int8Data(value))
    }
    pub fn int16(value: i16) -> Self {
        RegularInstruction::Int16(Int16Data(value))
    }
    pub fn int32(value: i32) -> Self {
        RegularInstruction::Int32(Int32Data(value))
    }
    pub fn int64(value: i64) -> Self {
        RegularInstruction::Int64(Int64Data(value))
    }
    pub fn int128(value: i128) -> Self {
        RegularInstruction::Int128(Int128Data(value))
    }
    pub fn uint8(value: u8) -> Self {
        RegularInstruction::UInt8(UInt8Data(value))
    }
    pub fn uint16(value: u16) -> Self {
        RegularInstruction::UInt16(UInt16Data(value))
    }
    pub fn uint32(value: u32) -> Self {
        RegularInstruction::UInt32(UInt32Data(value))
    }
    pub fn uint64(value: u64) -> Self {
        RegularInstruction::UInt64(UInt64Data(value))
    }
    pub fn uint128(value: u128) -> Self {
        RegularInstruction::UInt128(UInt128Data(value))
    }
    pub fn decimal_f32(value: f32) -> Self {
        RegularInstruction::DecimalF32(Float32Data(value))
    }
    pub fn decimal_f64(value: f64) -> Self {
        RegularInstruction::DecimalF64(Float64Data(value))
    }
    pub fn decimal_as_int16(value: i16) -> Self {
        RegularInstruction::DecimalAsInt16(FloatAsInt16Data(value))
    }
    pub fn decimal_as_int32(value: i32) -> Self {
        RegularInstruction::DecimalAsInt32(FloatAsInt32Data(value))
    }
    pub fn decimal_big(value: Decimal) -> Self {
        RegularInstruction::BigDecimal(value)
    }
    pub fn decimal(value: Decimal) -> Self {
        RegularInstruction::Decimal(value)
    }
    pub fn integer(value: Integer) -> Self {
        RegularInstruction::Integer(value)
    }
    pub fn big_integer(value: Integer) -> Self {
        RegularInstruction::BigInteger(value)
    }
    pub fn endpoint(value: Endpoint) -> Self {
        RegularInstruction::Endpoint(value)
    }
    pub fn instant(value: i128) -> Self {
        RegularInstruction::Instant(InstantData(value))
    }
    pub fn text(value: String) -> Self {
        RegularInstruction::Text(TextData(value))
    }
    pub fn short_text(value: String) -> Self {
        RegularInstruction::ShortText(ShortTextData(value))
    }
    pub fn tagged_value(tag: String, is_empty: bool) -> Self {
        RegularInstruction::TaggedValue(TaggedValue {
            tag: ShortTextData(tag),
            is_empty,
        })
    }
    pub fn list(count: u32) -> Self {
        match count {
            0..=255 => RegularInstruction::ShortList(ShortListData {
                element_count: count as u8,
            }),
            _ => RegularInstruction::List(ListData {
                element_count: count,
            }),
        }
    }
    pub fn map(count: u32) -> Self {
        match count {
            0..=255 => RegularInstruction::ShortMap(ShortMapData {
                element_count: count as u8,
            }),
            _ => RegularInstruction::Map(MapData {
                element_count: count,
            }),
        }
    }
    pub fn statements(count: u32, terminated: bool) -> Self {
        match count {
            0..=255 => {
                RegularInstruction::ShortStatements(ShortStatementsData {
                    statements_count: count as u8,
                    terminated,
                })
            }
            _ => RegularInstruction::Statements(StatementsData {
                statements_count: count,
                terminated,
            }),
        }
    }
    pub fn unbounded_statements() -> Self {
        RegularInstruction::UnboundedStatements
    }
    pub fn unbounded_statements_end(terminated: bool) -> Self {
        RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {
            terminated,
        })
    }
    pub fn apply(arg_count: u16) -> Self {
        RegularInstruction::Apply(ApplyData { arg_count })
    }
    pub fn get_property_text(key: String) -> Self {
        RegularInstruction::GetPropertyText(ShortTextData(key))
    }
    pub fn get_property_index(index: u32) -> Self {
        RegularInstruction::GetPropertyIndex(UInt32Data(index))
    }
    pub fn get_property_dynamic() -> Self {
        RegularInstruction::GetPropertyDynamic
    }
    pub fn take_property_text(key: String) -> Self {
        RegularInstruction::TakeEntryText(ShortTextData(key))
    }
    pub fn take_property_index(index: u32) -> Self {
        RegularInstruction::TakeEntryIndex(UInt32Data(index))
    }
    pub fn take_property_dynamic() -> Self {
        RegularInstruction::TakeEntryDynamic
    }
    pub fn set_property_text(key: String) -> Self {
        RegularInstruction::SetEntryText(ShortTextData(key))
    }
    pub fn set_property_index(index: u32) -> Self {
        RegularInstruction::SetEntryIndex(UInt32Data(index))
    }
    pub fn set_property_dynamic() -> Self {
        RegularInstruction::SetEntryDynamic
    }
    pub fn matches() -> Self {
        RegularInstruction::Matches
    }
    pub fn structural_equal() -> Self {
        RegularInstruction::StructuralEqual
    }
    pub fn not_structural_equal() -> Self {
        RegularInstruction::NotStructuralEqual
    }
    pub fn equal() -> Self {
        RegularInstruction::Equal
    }
    pub fn not_equal() -> Self {
        RegularInstruction::NotEqual
    }
    pub fn is() -> Self {
        RegularInstruction::Is
    }
    pub fn add() -> Self {
        RegularInstruction::Add
    }
    pub fn subtract() -> Self {
        RegularInstruction::Subtract
    }
    pub fn multiply() -> Self {
        RegularInstruction::Multiply
    }
    pub fn divide() -> Self {
        RegularInstruction::Divide
    }
    pub fn unary_plus() -> Self {
        RegularInstruction::UnaryPlus
    }
    pub fn unary_minus() -> Self {
        RegularInstruction::UnaryMinus
    }
    pub fn bitwise_not() -> Self {
        RegularInstruction::BitwiseNot
    }
    pub fn increment() -> Self {
        RegularInstruction::Increment
    }
    pub fn decrement() -> Self {
        RegularInstruction::Decrement
    }
    pub fn append_entry() -> Self {
        RegularInstruction::AppendEntry
    }
    pub fn clear() -> Self {
        RegularInstruction::Clear
    }
    pub fn splice(
        start_index: u32,
        delete_count: u32,
        insert_count: u32,
    ) -> Self {
        RegularInstruction::Splice(SpliceData {
            start_index,
            delete_count,
            insert_count,
        })
    }
    pub fn splice_dynamic() -> Self {
        RegularInstruction::SpliceDynamic
    }
    pub fn set_shared_container_value() -> Self {
        RegularInstruction::SetSharedContainerValue
    }
    pub fn take_entry_text(key: String) -> Self {
        RegularInstruction::TakeEntryText(ShortTextData(key))
    }
    pub fn take_entry_index(index: u32) -> Self {
        RegularInstruction::TakeEntryIndex(UInt32Data(index))
    }
    pub fn take_entry_dynamic() -> Self {
        RegularInstruction::TakeEntryDynamic
    }
    pub fn set_entry_text(key: String) -> Self {
        RegularInstruction::SetEntryText(ShortTextData(key))
    }
    pub fn set_entry_index(index: u32) -> Self {
        RegularInstruction::SetEntryIndex(UInt32Data(index))
    }
    pub fn set_entry_dynamic() -> Self {
        RegularInstruction::SetEntryDynamic
    }
    pub fn null() -> Self {
        RegularInstruction::Null
    }
    pub fn r#true() -> Self {
        RegularInstruction::True
    }
    pub fn r#false() -> Self {
        RegularInstruction::False
    }
    pub fn set_stack_value(stack_index: StackIndex) -> Self {
        RegularInstruction::SetStackValue(stack_index)
    }
    pub fn borrow_stack_value(stack_index: StackIndex) -> Self {
        RegularInstruction::BorrowStackValue(stack_index)
    }
    pub fn clone_stack_value(stack_index: StackIndex) -> Self {
        RegularInstruction::CloneStackValue(stack_index)
    }
    pub fn key_value_dynamic() -> Self {
        RegularInstruction::KeyValueDynamic
    }
    pub fn key_value_short_text(key: String) -> Self {
        RegularInstruction::KeyValueShortText(ShortTextData(key))
    }
    pub fn push_to_stack() -> Self {
        RegularInstruction::PushToStack
    }
    pub fn push_list_to_stack() -> Self {
        RegularInstruction::PushListToStack
    }
    pub fn get_stack_value_shared_ref(stack_index: StackIndex) -> Self {
        RegularInstruction::GetStackValueSharedRef(stack_index)
    }
    pub fn get_stack_value_shared_ref_mut(stack_index: StackIndex) -> Self {
        RegularInstruction::GetStackValueSharedRefMut(stack_index)
    }
    pub fn take_stack_value(stack_index: StackIndex) -> Self {
        RegularInstruction::TakeStackValue(stack_index)
    }
    pub fn get_root_property(root_property: RootProperty) -> Self {
        RegularInstruction::GetRootProperty(root_property)
    }
    pub fn unbox() -> Self {
        RegularInstruction::Unbox
    }
    pub fn typed_value() -> Self {
        RegularInstruction::TypedValue
    }
    pub fn type_expression() -> Self {
        RegularInstruction::TypeExpression
    }
    pub fn derive_shared_reference() -> Self {
        RegularInstruction::DeriveSharedReference
    }
    pub fn derive_shared_reference_mut() -> Self {
        RegularInstruction::DeriveSharedReferenceMut
    }
    pub fn create_shared() -> Self {
        RegularInstruction::CreateShared
    }
    pub fn create_shared_mut() -> Self {
        RegularInstruction::CreateSharedMut
    }
    pub fn request_remote_shared_ref(address: RemotePointerAddress) -> Self {
        RegularInstruction::RequestRemoteSharedRef(address)
    }
    pub fn request_remote_shared_ref_mut(
        address: RemotePointerAddress,
    ) -> Self {
        RegularInstruction::RequestRemoteSharedRefMut(address)
    }
    pub fn get_local_shared_ref(address: SelfOwnedPointerAddress) -> Self {
        RegularInstruction::GetLocalSharedRef(address)
    }
    pub fn get_core_lib_value(core_lib_id: CoreLibIdIndex) -> Self {
        RegularInstruction::GetCoreLibValue(core_lib_id)
    }
    pub fn shared_ref(shared_ref: SharedRef) -> Self {
        RegularInstruction::SharedRef(shared_ref)
    }
    pub fn shared_ref_with_value(
        shared_ref_with_value: SharedRefWithValue,
    ) -> Self {
        RegularInstruction::SharedRefWithValue(shared_ref_with_value)
    }
    pub fn move_with_value(move_with_value: MoveWithValue) -> Self {
        RegularInstruction::MoveWithValue(move_with_value)
    }
    pub fn remote_execution(instruction_block: InstructionBlockData) -> Self {
        RegularInstruction::RemoteExecution(instruction_block)
    }
    pub fn range() -> Self {
        RegularInstruction::Range
    }

    pub fn remote_execution_debug_tree(
        tree: InstructionBlockDataDebugTree,
    ) -> Self {
        RegularInstruction::_RemoteExecutionDebugTree(tree)
    }

    pub fn remote_execution_debug_flat(
        tree: InstructionBlockDataDebugFlat,
    ) -> Self {
        RegularInstruction::_RemoteExecutionDebugFlat(tree)
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
pub enum RegularInstruction {
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
    #[br(pre_assert(false))]
    _RemoteExecutionDebugFlat(#[brw(ignore)] crate::global::protocol_structures::instruction_data::InstructionBlockDataDebugFlat) = 253,
    /// Debug variant for RemoteExecution, includes full remote execution instruction tree instead of raw dxb
    /// This variant is only used by the disassembler
    #[cfg(feature = "disassembler")]
    #[br(pre_assert(false))]
    _RemoteExecutionDebugTree(#[brw(ignore)] crate::global::protocol_structures::instruction_data::InstructionBlockDataDebugTree) = 254,
}

impl RegularInstruction {
    #[inline]
    pub fn instruction_code(&self) -> InstructionCode {
        // SAFETY:
        //
        // RegularInstructionData has #[repr(u8)], so we can guarantee
        // that its discriminant can be read as a u8 from the addr
        let raw = unsafe { *(self as *const Self).cast::<u8>() };

        InstructionCode::try_from(raw).unwrap_or_else(|_| InstructionCode::ADD)
    }
}

impl RegularInstruction {
    /// Returns how many (if any) regular or type instructions are expected as child instructions for a given instructions
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        match self {
            RegularInstruction::RemoteExecution(_) => {
                NextExpectedInstructions::Regular(1)
            } // receivers

            #[cfg(feature = "disassembler")]
            RegularInstruction::_RemoteExecutionDebugTree(_)
            | RegularInstruction::_RemoteExecutionDebugFlat(_) => {
                NextExpectedInstructions::Regular(1)
            } // receivers

            RegularInstruction::ShortList(list) => {
                NextExpectedInstructions::Regular(list.element_count as u32)
            } // list elements

            RegularInstruction::List(list) => {
                NextExpectedInstructions::Regular(list.element_count)
            } // list elements

            RegularInstruction::ShortMap(map) => {
                NextExpectedInstructions::Regular(map.element_count as u32)
            } // map entries

            RegularInstruction::Map(map) => {
                NextExpectedInstructions::Regular(map.element_count)
            } // map entries

            RegularInstruction::ShortStatements(statements) => {
                NextExpectedInstructions::Regular(
                    statements.statements_count as u32,
                )
            }
            RegularInstruction::Statements(statements) => {
                NextExpectedInstructions::Regular(statements.statements_count)
            } // statements in block

            RegularInstruction::UnboundedStatements => {
                NextExpectedInstructions::UnboundedStart
            }

            RegularInstruction::UnboundedStatementsEnd(_) => {
                NextExpectedInstructions::UnboundedEnd
            }

            RegularInstruction::Apply(apply_data) => {
                NextExpectedInstructions::Regular(
                    apply_data.arg_count as u32 + 1,
                )
            } // arguments plus base to apply to

            RegularInstruction::GetPropertyText(_)
            | RegularInstruction::GetPropertyIndex(_)
            | RegularInstruction::TakeEntryText(_)
            | RegularInstruction::TakeEntryIndex(_) => {
                NextExpectedInstructions::Regular(1)
            } // value to get property from

            RegularInstruction::GetPropertyDynamic
            | RegularInstruction::TakeEntryDynamic => {
                NextExpectedInstructions::Regular(2)
            } // value to get property from + property key

            RegularInstruction::SetEntryText(_)
            | RegularInstruction::SetEntryIndex(_) => {
                NextExpectedInstructions::Regular(2)
            } // value to set property on and new value

            RegularInstruction::SetEntryDynamic => {
                NextExpectedInstructions::Regular(3)
            } // value to set property on + property key + new value

            RegularInstruction::Unbox => NextExpectedInstructions::Regular(1), // value to unbox

            RegularInstruction::AppendEntry => {
                NextExpectedInstructions::Regular(2)
            }
            RegularInstruction::Splice(SpliceData { insert_count, .. }) => {
                NextExpectedInstructions::Regular(*insert_count + 1)
            }
            RegularInstruction::SpliceDynamic => {
                NextExpectedInstructions::Regular(4)
            }

            RegularInstruction::SetSharedContainerValue => {
                NextExpectedInstructions::Regular(2)
            } // container to set value on + new value

            RegularInstruction::KeyValueDynamic => {
                NextExpectedInstructions::Regular(2)
            } // key + value

            RegularInstruction::KeyValueShortText(_) => {
                NextExpectedInstructions::Regular(1)
            } // value

            RegularInstruction::Matches => {
                NextExpectedInstructions::RegularAndType(1, 1)
            }

            RegularInstruction::Add
            | RegularInstruction::Multiply
            | RegularInstruction::Subtract
            | RegularInstruction::Divide => {
                NextExpectedInstructions::Regular(2)
            } // left and right operand

            RegularInstruction::StructuralEqual
            | RegularInstruction::NotStructuralEqual
            | RegularInstruction::Equal
            | RegularInstruction::NotEqual
            | RegularInstruction::Is => NextExpectedInstructions::Regular(2), // left and right operand

            RegularInstruction::UnaryMinus
            | RegularInstruction::UnaryPlus
            | RegularInstruction::BitwiseNot => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstruction::DeriveSharedReference
            | RegularInstruction::DeriveSharedReferenceMut
            | RegularInstruction::CreateShared
            | RegularInstruction::CreateSharedMut => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstruction::PushToStack
            | RegularInstruction::PushListToStack
            | RegularInstruction::SetStackValue(_) => {
                NextExpectedInstructions::Regular(1)
            }
            RegularInstruction::TypedValue => {
                NextExpectedInstructions::RegularAndType(1, 1)
            }

            RegularInstruction::TypeExpression => {
                NextExpectedInstructions::Type(1)
            }

            RegularInstruction::Range => NextExpectedInstructions::Regular(2),
            RegularInstruction::TaggedValue(TaggedValue {
                is_empty, ..
            }) => {
                if *is_empty {
                    NextExpectedInstructions::None
                } else {
                    NextExpectedInstructions::Regular(1)
                }
            }

            RegularInstruction::SharedRefWithValue(_) => {
                NextExpectedInstructions::Regular(1)
            }
            RegularInstruction::MoveWithValue(_) => {
                NextExpectedInstructions::Regular(1)
            }

            RegularInstruction::Increment => {
                NextExpectedInstructions::Regular(2)
            }
            RegularInstruction::Decrement => {
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
            RegularInstruction::Int8(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Int16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Int32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Int64(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Int128(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::UInt8(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::UInt16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::UInt32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::UInt64(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::UInt128(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Apply(count) => {
                write!(string, "[arg_count: {}]", count.arg_count)
            }
            RegularInstruction::BigInteger(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Integer(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Endpoint(data) => {
                write!(string, "{data}")
            }
            RegularInstruction::Instant(data) => {
                write!(string, "{}", Instant(data.0).to_iso_string())
            }

            RegularInstruction::DecimalAsInt16(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::DecimalAsInt32(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::DecimalF32(data) => {
                write!(
                    string,
                    "{}",
                    TypedDecimal::F32(data.0.into())
                )
            }
            RegularInstruction::DecimalF64(data) => {
                write!(
                    string,
                    "{}",
                    TypedDecimal::F64(data.0.into())
                )
            }
            RegularInstruction::BigDecimal(data) => {
                write!(string, "{}", data)
            }
            RegularInstruction::Decimal(data) => {
                write!(string, "{}", data)
            }
            RegularInstruction::ShortText(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Text(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Statements(data) => {
                write!(string, "[count: {}, terminated: {}]", data.statements_count, data.terminated)
            }
            RegularInstruction::ShortStatements(data) => {
                write!(string, "[count: {}, terminated: {}]", data.statements_count, data.terminated)
            }
            RegularInstruction::List(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstruction::ShortList(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstruction::Map(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstruction::ShortMap(data) => {
                write!(string, "{}", data.element_count)
            }
            RegularInstruction::KeyValueShortText(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::CloneStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::GetRootProperty(property) => {
                write!(string, "$.{}", property)
            }
            RegularInstruction::BorrowStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::GetStackValueSharedRef(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::GetStackValueSharedRefMut(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::TakeStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::SetStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::Splice(splice_data) => {
                write!(string, "[start: {}, delete_count: {}, insert_count: {}]", splice_data.start_index, splice_data.delete_count, splice_data.insert_count)
            }
            RegularInstruction::RequestRemoteSharedRef(address) => {
                write!(
                    string,
                    "[endpoint: {}, address:{}]",
                    address.endpoint(),
                    address
                )
            }
            RegularInstruction::RequestRemoteSharedRefMut(address) => {
                write!(
                    string,
                    "[endpoint: {}, address:{}]",
                    address.endpoint(),
                    address
                )
            }
            RegularInstruction::GetLocalSharedRef(address) => {
                write!(
                    string,
                    "[address:{}]",
                    address
                )
            }
            RegularInstruction::GetCoreLibValue(address) => {
                write!(
                    string,
                    "[id: {:4x}]",
                    &address.0
                )
            }
            RegularInstruction::SharedRef(shared_ref) => {
                write!(
                    string,
                    "[ref_mutability: {:?}, address: {}]",
                    shared_ref.ref_mutability, shared_ref.address.clone()
                )
            }
            RegularInstruction::SharedRefWithValue(shared_ref) => {
                write!(
                    string,
                    "[ref_mutability: {:?}, address: {}, container_mutability: {:?}]",
                    shared_ref.ref_mutability,
                    PointerAddress::from(shared_ref.address.clone()),
                    shared_ref.container_mutability
                )
            }
            RegularInstruction::MoveWithValue(move_with_value) => {
                write!(
                    string,
                    "[mutability: {:?}, previous address: {}]",
                    move_with_value.mutability,
                    move_with_value.previous_address
                )
            }
            RegularInstruction::RemoteExecution(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            #[cfg(feature = "disassembler")]
            RegularInstruction::_RemoteExecutionDebugTree(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            #[cfg(feature = "disassembler")]
            RegularInstruction::_RemoteExecutionDebugFlat(data) => {
                write!(
                    string,
                    "[length: {}, injected_variables: {:?}]",
                    data.length,
                    data.injected_values
                )
            }
            RegularInstruction::GetPropertyIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::SetEntryIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::TakeEntryIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::GetPropertyText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstruction::TakeEntryText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstruction::SetEntryText(short_text_data) => {
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
            RegularInstruction::_RemoteExecutionDebugTree(data) => {
                InnerInstructions::Tree(&data.body)
            }
            RegularInstruction::_RemoteExecutionDebugFlat(data) => {
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
impl Serialize for RegularInstruction {
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

impl RegularInstruction {
    pub fn read_from<R>(reader: &mut R) -> BinResult<Self>
    where
        R: Read + Seek,
    {
        let code = InstructionCode::read_options(reader, Endian::Little, ())?;
        <Self as BinRead>::read_options(reader, Endian::Little, (code,))
    }

    pub fn write_to<W>(&self, writer: &mut W) -> BinResult<()>
    where
        W: Write + Seek,
    {
        self.instruction_code()
            .write_options(writer, Endian::Little, ())?;
        <Self as BinWrite>::write_options(self, writer, Endian::Little, ())
    }
}

impl Display for RegularInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.instruction_code())?;

        if let Some(metadata_string) = self.metadata_string() {
            write!(f, " {}", metadata_string)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn encode(instruction: &RegularInstruction) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        instruction
            .write_to(&mut writer)
            .expect("instruction should serialize");
        writer.into_inner()
    }

    fn decode(bytes: &[u8]) -> BinResult<RegularInstruction> {
        let mut reader = Cursor::new(bytes);
        let instruction = RegularInstruction::read_from(&mut reader)?;
        assert_eq!(
            reader.position() as usize,
            bytes.len(),
            "decoder did not consume the complete instruction"
        );
        Ok(instruction)
    }

    fn assert_round_trip(instruction: RegularInstruction) {
        let encoded = encode(&instruction);
        let decoded =
            decode(&encoded).expect("encoded instruction should deserialize");

        assert_eq!(decoded, instruction);
    }

    #[test]
    fn int8() {
        let instruction = RegularInstruction::int8(42);
        let encoded = encode(&instruction);
        assert_eq!(encoded, vec![InstructionCode::INT_8.as_u8(), 42u8,]);
    }

    #[test]
    fn negative_int8() {
        let instruction = RegularInstruction::int8(-5);
        let encoded = encode(&instruction);
        assert_eq!(
            encoded,
            vec![InstructionCode::INT_8.as_u8(), (-5i8) as u8,]
        );
        assert_eq!(decode(&encoded).unwrap(), instruction,);
    }

    #[test]
    fn int16() {
        let instruction = RegularInstruction::int16(0x1234);
        let encoded = encode(&instruction);
        assert_eq!(encoded, vec![InstructionCode::INT_16.as_u8(), 0x34, 0x12,]);
    }

    #[test]
    fn instruction_code_matches_variant() {
        assert_eq!(
            RegularInstruction::int8(1).instruction_code(),
            InstructionCode::INT_8,
        );
        assert_eq!(
            RegularInstruction::int16(1).instruction_code(),
            InstructionCode::INT_16,
        );
        assert_eq!(
            RegularInstruction::Int32(Int32Data(1),).instruction_code(),
            InstructionCode::INT_32,
        );
        assert_eq!(
            RegularInstruction::UInt8(UInt8Data(1),).instruction_code(),
            InstructionCode::UINT_8,
        );
    }

    #[test]
    fn integer_variants_round_trip() {
        let instructions = [
            RegularInstruction::Int8(Int8Data(i8::MIN)),
            RegularInstruction::Int8(Int8Data(i8::MAX)),
            RegularInstruction::Int16(Int16Data(i16::MIN)),
            RegularInstruction::Int16(Int16Data(i16::MAX)),
            RegularInstruction::Int32(Int32Data(i32::MIN)),
            RegularInstruction::Int32(Int32Data(i32::MAX)),
            RegularInstruction::Int64(Int64Data(i64::MIN)),
            RegularInstruction::Int64(Int64Data(i64::MAX)),
            RegularInstruction::Int128(Int128Data(i128::MIN)),
            RegularInstruction::Int128(Int128Data(i128::MAX)),
        ];

        for instruction in instructions {
            assert_round_trip(instruction);
        }
    }

    #[test]
    fn unsigned_integer_variants_round_trip() {
        let instructions = [
            RegularInstruction::UInt8(UInt8Data(u8::MAX)),
            RegularInstruction::UInt16(UInt16Data(u16::MAX)),
            RegularInstruction::UInt32(UInt32Data(u32::MAX)),
            RegularInstruction::UInt64(UInt64Data(u64::MAX)),
            RegularInstruction::UInt128(UInt128Data(u128::MAX)),
        ];

        for instruction in instructions {
            assert_round_trip(instruction);
        }
    }

    #[test]
    fn stream() {
        let instructions = vec![
            RegularInstruction::int8(-12),
            RegularInstruction::int16(1234),
            RegularInstruction::UInt32(UInt32Data(987_654)),
        ];
        let mut stream = Cursor::new(Vec::new());
        for instruction in &instructions {
            instruction
                .write_to(&mut stream)
                .expect("instruction should serialize");
        }

        stream.set_position(0);

        let decoded: Vec<_> = (0..instructions.len())
            .map(|_| {
                RegularInstruction::read_from(&mut stream)
                    .expect("instruction should deserialize")
            })
            .collect();

        assert_eq!(decoded, instructions);
        assert_eq!(stream.position() as usize, stream.get_ref().len(),);
    }

    #[test]
    fn missing_bytes() {
        let bytes = [
            InstructionCode::INT_16.as_u8(),
            0x34,
            // Missing the second i16 byte
        ];

        let result = RegularInstruction::read_from(&mut Cursor::new(bytes));
        assert!(result.is_err());
    }

    #[test]
    fn unknown_opcode() {
        let bytes = [0xff];
        let result = RegularInstruction::read_from(&mut Cursor::new(bytes));
        assert!(result.is_err());
    }

    #[test]
    fn reencoding_decoded() {
        let original =
            vec![InstructionCode::INT_32.as_u8(), 0x78, 0x56, 0x34, 0x12];
        let decoded = decode(&original).expect("valid INT_32");
        let reencoded = encode(&decoded);
        assert_eq!(reencoded, original);
    }

    #[test]
    fn test() {
        let ins = RegularInstruction::SetStackValue(StackIndex(4));
        let encoded = encode(&ins);
        println!("encoded: {:?}", encoded);
        let mut a = Cursor::new(Vec::new());
        let endoded2 = ins.write(&mut a).unwrap();
        println!("encoded2: {:?}", a.into_inner());
    }
}
