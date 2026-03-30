use core::fmt::Write;
use core::fmt::Display;
use binrw::io::{Read, Seek};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use binrw::meta::{EndianKind, ReadEndian};
use crate::dxb_parser::body::DXBParserError;
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::instruction_data::{ApplyData, DecimalData, Float32Data, Float64Data, FloatAsInt16Data, FloatAsInt32Data, InstructionBlockData, Int128Data, Int16Data, Int32Data, Int64Data, Int8Data, IntegerData, ListData, MapData, ModifySlot, Move, PerformMove, PushToStackMultiple, RawInternalPointerAddress, RawLocalPointerAddress, RawRemotePointerAddress, SetSharedContainerValue, SharedRef, SharedRefWithValue, ShortListData, ShortMapData, ShortStatementsData, ShortTextData, ShortTextDataRaw, StackIndex, StatementsData, TextData, TextDataRaw, UInt128Data, UInt16Data, UInt32Data, UInt64Data, UInt8Data, UnboundedStatementsData};
use crate::global::protocol_structures::instructions::NextExpectedInstructions;
use crate::shared_values::pointer_address::PointerAddress;
use crate::values::core_values::decimal::utils::decimal_to_string;
use crate::values::core_values::endpoint::Endpoint;
use crate::prelude::*;

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
    ModifyStackValue(ModifySlot),

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

    PushToStack,
    PushToStackMultiple(PushToStackMultiple),
    CloneStackValue(StackIndex),
    BorrowStackValue(StackIndex),
    GetStackValueSharedRef(StackIndex),
    GetStackValueSharedRefMut(StackIndex),
    TakeStackValue(StackIndex),
    SetStackValue(StackIndex),

    GetInternalSlot(StackIndex), // FIXME slot address

    SetSharedContainerValue(SetSharedContainerValue),
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
            RegularInstruction::Integer(_) => InstructionCode::INT,
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
            RegularInstruction::PushToStack => InstructionCode::PUSH_TO_STACK,
            RegularInstruction::PushToStackMultiple(_) => InstructionCode::PUSH_TO_STACK_MULTIPLE,
            RegularInstruction::CloneStackValue(_) => InstructionCode::CLONE_STACK_VALUE,
            RegularInstruction::BorrowStackValue(_) => InstructionCode::BORROW_STACK_VALUE,
            RegularInstruction::GetStackValueSharedRef(_) => InstructionCode::GET_STACK_VALUE_SHARED_REF,
            RegularInstruction::GetStackValueSharedRefMut(_) => InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT,
            RegularInstruction::TakeStackValue(_) => InstructionCode::TAKE_STACK_VALUE,
            RegularInstruction::SetStackValue(_) => InstructionCode::SET_STACK_VALUE,
            RegularInstruction::ModifyStackValue(_) => InstructionCode::MODIFY_STACK_VALUE,
            RegularInstruction::GetInternalSlot(_) => InstructionCode::GET_INTERNAL_SLOT,
            RegularInstruction::SetSharedContainerValue(_) => InstructionCode::SET_SHARED_CONTAINER_VALUE,
            RegularInstruction::Unbox => InstructionCode::UNBOX,
            RegularInstruction::TypedValue => InstructionCode::TYPED_VALUE,
            RegularInstruction::TypeExpression => InstructionCode::TYPE_EXPRESSION,
        }
    }
}

impl RegularInstruction {

    /// Returns how many (if any) regular or type instructions are expected as child instructions for a given instructions
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        match self {
            RegularInstruction::RemoteExecution(_) => NextExpectedInstructions::Regular(1), // receivers

            RegularInstruction::ShortList(list) | RegularInstruction::List(list) =>
                NextExpectedInstructions::Regular(list.element_count), // list elements

            RegularInstruction::ShortMap(map) | RegularInstruction::Map(map) =>
                NextExpectedInstructions::Regular(map.element_count), // map entries

            RegularInstruction::ShortStatements(statements) | RegularInstruction::Statements(statements) =>
                NextExpectedInstructions::Regular(statements.statements_count), // statements in block

            RegularInstruction::UnboundedStatements =>
                NextExpectedInstructions::UnboundedStart,

            RegularInstruction::UnboundedStatementsEnd(_) =>
                NextExpectedInstructions::UnboundedEnd,

            RegularInstruction::Apply(apply_data) =>
                NextExpectedInstructions::Regular(apply_data.arg_count as u32 + 1), // arguments plus base to apply to

            RegularInstruction::GetPropertyText(_) |
            RegularInstruction::GetPropertyIndex(_) |
            RegularInstruction::TakePropertyText(_) |
            RegularInstruction::TakePropertyIndex(_) =>
                NextExpectedInstructions::Regular(1), // value to get property from

            RegularInstruction::GetPropertyDynamic |
            RegularInstruction::TakePropertyDynamic =>
                NextExpectedInstructions::Regular(2), // value to get property from + property key

            RegularInstruction::SetPropertyText(_) | RegularInstruction::SetPropertyIndex(_) =>
                NextExpectedInstructions::Regular(2), // value to set property on and new value

            RegularInstruction::SetPropertyDynamic => NextExpectedInstructions::Regular(3),  // value to set property on + property key + new value

            RegularInstruction::Unbox => NextExpectedInstructions::Regular(1), // value to unbox

            RegularInstruction::SetSharedContainerValue(_) => NextExpectedInstructions::Regular(2), // container to set value on + new value

            RegularInstruction::KeyValueDynamic => NextExpectedInstructions::Regular(2), // key + value

            RegularInstruction::KeyValueShortText(_) => NextExpectedInstructions::Regular(1), // value

            RegularInstruction::Matches => NextExpectedInstructions::RegularAndType(1,1),

            RegularInstruction::Add |
            RegularInstruction::Multiply |
            RegularInstruction::Subtract |
            RegularInstruction::Divide => NextExpectedInstructions::Regular(2), // left and right operand

            RegularInstruction::StructuralEqual |
            RegularInstruction::NotStructuralEqual |
            RegularInstruction::Equal |
            RegularInstruction::NotEqual |
            RegularInstruction::Is => NextExpectedInstructions::Regular(2), // left and right operand

            RegularInstruction::UnaryMinus |
            RegularInstruction::UnaryPlus |
            RegularInstruction::BitwiseNot => NextExpectedInstructions::Regular(1),

            RegularInstruction::GetSharedReference |
            RegularInstruction::GetSharedReferenceMut |
            RegularInstruction::CreateShared |
            RegularInstruction::CreateSharedMut => NextExpectedInstructions::Regular(1),

            RegularInstruction::PushToStack |
            RegularInstruction::PushToStackMultiple(_) |
            RegularInstruction::SetStackValue(_) => NextExpectedInstructions::Regular(1),
            RegularInstruction::ModifyStackValue(_) => NextExpectedInstructions::Regular(1),

            RegularInstruction::TypedValue => NextExpectedInstructions::RegularAndType(1,1),

            RegularInstruction::TypeExpression => NextExpectedInstructions::Type(1),

            RegularInstruction::Range => NextExpectedInstructions::Regular(2),

            RegularInstruction::SharedRefWithValue(_) => NextExpectedInstructions::Regular(1),

            _ => NextExpectedInstructions::None,
        }
    }

    /// Based on the instruction code, read the corresponding instruction data and construct the RegularInstruction variant
    fn read_instruction<R: Read + Seek>(
        reader: &mut R,
        instruction_code: InstructionCode,
    ) -> BinResult<Self> {
        match instruction_code {
            InstructionCode::UINT_8 => {
                UInt8Data::read(reader).map(RegularInstruction::UInt8)
            }
            InstructionCode::UINT_16 => {
                UInt16Data::read(reader).map(RegularInstruction::UInt16)
            }
            InstructionCode::UINT_32 => {
                UInt32Data::read(reader).map(RegularInstruction::UInt32)
            }
            InstructionCode::UINT_64 => {
                UInt64Data::read(reader).map(RegularInstruction::UInt64)
            }
            InstructionCode::UINT_128 => {
                UInt128Data::read(reader).map(RegularInstruction::UInt128)
            }
            InstructionCode::INT_8 => {
                Int8Data::read(reader).map(RegularInstruction::Int8)
            }
            InstructionCode::INT_16 => {
                Int16Data::read(reader).map(RegularInstruction::Int16)
            }
            InstructionCode::INT_32 => {
                Int32Data::read(reader).map(RegularInstruction::Int32)
            }
            InstructionCode::INT_64 => {
                Int64Data::read(reader).map(RegularInstruction::Int64)
            }
            InstructionCode::INT_128 => {
                Int128Data::read(reader).map(RegularInstruction::Int128)
            }
            InstructionCode::INT_BIG => {
                IntegerData::read(reader).map(RegularInstruction::BigInteger)
            }
            InstructionCode::INT => {
                IntegerData::read(reader).map(RegularInstruction::Integer)
            }
            InstructionCode::DECIMAL_F32 => {
                Float32Data::read(reader).map(RegularInstruction::DecimalF32)
            }
            InstructionCode::DECIMAL_F64 => {
                Float64Data::read(reader).map(RegularInstruction::DecimalF64)
            }
            InstructionCode::DECIMAL_BIG => {
                DecimalData::read(reader).map(RegularInstruction::BigDecimal)
            }
            InstructionCode::DECIMAL_AS_INT_16 => {
                FloatAsInt16Data::read(reader).map(RegularInstruction::DecimalAsInt16)
            }
            InstructionCode::DECIMAL_AS_INT_32 => {
                FloatAsInt32Data::read(reader).map(RegularInstruction::DecimalAsInt32)
            }
            InstructionCode::DECIMAL => {
                DecimalData::read(reader).map(RegularInstruction::Decimal)
            }
            InstructionCode::REMOTE_EXECUTION => {
                InstructionBlockData::read(reader).map(RegularInstruction::RemoteExecution)
            }
            InstructionCode::SHORT_TEXT => {
                ShortTextData::read(reader).map(RegularInstruction::ShortText)
            }
            InstructionCode::ENDPOINT => {
                Endpoint::read(reader).map(RegularInstruction::Endpoint)
            }
            InstructionCode::TEXT => {
                TextData::read(reader).map(RegularInstruction::Text)
            }
            InstructionCode::TRUE => Ok(RegularInstruction::True),
            InstructionCode::FALSE => Ok(RegularInstruction::False),
            InstructionCode::NULL => Ok(RegularInstruction::Null),

            // collections
            InstructionCode::LIST => {
                ListData::read(reader).map(RegularInstruction::List)
            }
            InstructionCode::SHORT_LIST => {
                ShortListData::read(reader)
                    .map(|list| {
                        ListData { element_count: list.element_count as u32}
                    })
                    .map(RegularInstruction::ShortList)
            }
            InstructionCode::MAP => {
                MapData::read(reader).map(RegularInstruction::Map)
            }
            InstructionCode::SHORT_MAP => {
                ShortMapData::read(reader)
                    .map(|map| {
                        MapData { element_count: map.element_count as u32}
                    })
                    .map(RegularInstruction::ShortMap)
            }

            InstructionCode::STATEMENTS => {
                StatementsData::read(reader).map(RegularInstruction::Statements)
            }
            InstructionCode::SHORT_STATEMENTS => {
                ShortStatementsData::read(reader)
                    .map(|data|
                        StatementsData {
                            statements_count: data.statements_count as u32,
                            terminated: data.terminated,
                        }
                    )
                    .map(RegularInstruction::ShortStatements)
            }

            InstructionCode::UNBOUNDED_STATEMENTS => Ok(RegularInstruction::UnboundedStatements),

            InstructionCode::UNBOUNDED_STATEMENTS_END => {
                UnboundedStatementsData::read(reader).map(RegularInstruction::UnboundedStatementsEnd)
            }

            InstructionCode::APPLY_ZERO => {
                Ok(RegularInstruction::Apply(ApplyData {
                    arg_count: 0,
                }))
            }
            InstructionCode::APPLY_SINGLE => {
                Ok(RegularInstruction::Apply(ApplyData {
                    arg_count: 1,
                }))
            }

            InstructionCode::APPLY => {
                ApplyData::read(reader).map(RegularInstruction::Apply)
            }

            InstructionCode::GET_PROPERTY_TEXT => {
                ShortTextData::read(reader).map(RegularInstruction::GetPropertyText)
            }

            InstructionCode::GET_PROPERTY_INDEX => {
                UInt32Data::read(reader).map(RegularInstruction::GetPropertyIndex)
            }

            InstructionCode::GET_PROPERTY_DYNAMIC => {
                Ok(RegularInstruction::GetPropertyDynamic)
            }

            InstructionCode::TAKE_PROPERTY_TEXT => {
                ShortTextData::read(reader).map(RegularInstruction::TakePropertyText)
            }

            InstructionCode::TAKE_PROPERTY_INDEX => {
                UInt32Data::read(reader).map(RegularInstruction::TakePropertyIndex)
            }

            InstructionCode::TAKE_PROPERTY_DYNAMIC => {
                Ok(RegularInstruction::TakePropertyDynamic)
            }

            InstructionCode::SET_PROPERTY_TEXT => {
                ShortTextData::read(reader).map(RegularInstruction::SetPropertyText)
            }

            InstructionCode::SET_PROPERTY_INDEX => {
                UInt32Data::read(reader).map(RegularInstruction::SetPropertyIndex)
            }

            InstructionCode::SET_PROPERTY_DYNAMIC => {
                Ok(RegularInstruction::SetPropertyDynamic)
            }

            InstructionCode::UNBOX => {
                Ok(RegularInstruction::Unbox)
            }
            InstructionCode::SET_SHARED_CONTAINER_VALUE => {
                SetSharedContainerValue::read(reader).map(RegularInstruction::SetSharedContainerValue)
            }

            InstructionCode::KEY_VALUE_SHORT_TEXT => {
                ShortTextData::read(reader).map(RegularInstruction::KeyValueShortText)
            }

            InstructionCode::KEY_VALUE_DYNAMIC => {
                Ok(RegularInstruction::KeyValueDynamic)
            }

            InstructionCode::ADD => {
                Ok(RegularInstruction::Add)
            }
            InstructionCode::SUBTRACT => {
                Ok(RegularInstruction::Subtract)
            }
            InstructionCode::MULTIPLY => {
                Ok(RegularInstruction::Multiply)
            }
            InstructionCode::DIVIDE => {
                Ok(RegularInstruction::Divide)
            }

            InstructionCode::UNARY_MINUS => {
                Ok(RegularInstruction::UnaryMinus)
            }
            InstructionCode::UNARY_PLUS => {
                Ok(RegularInstruction::UnaryPlus)
            }
            InstructionCode::BITWISE_NOT => {
                Ok(RegularInstruction::BitwiseNot)
            }

            InstructionCode::STRUCTURAL_EQUAL => {
                Ok(RegularInstruction::StructuralEqual)
            }
            InstructionCode::EQUAL => {
                Ok(RegularInstruction::Equal)
            }
            InstructionCode::NOT_STRUCTURAL_EQUAL => {
                Ok(RegularInstruction::NotStructuralEqual)
            }
            InstructionCode::NOT_EQUAL => {
                Ok(RegularInstruction::NotEqual)
            }
            InstructionCode::IS => {
                Ok(RegularInstruction::Is)
            }
            InstructionCode::MATCHES => {
                Ok(RegularInstruction::Matches)
            }
            InstructionCode::GET_SHARED_REF => {
                Ok(RegularInstruction::GetSharedReference)
            }
            InstructionCode::GET_SHARED_REF_MUT => {
                Ok(RegularInstruction::GetSharedReferenceMut)
            }

            InstructionCode::SHARED_REF => {
                SharedRef::read(reader).map(RegularInstruction::SharedRef)
            }
            InstructionCode::SHARED_REF_WITH_VALUE => {
                SharedRefWithValue::read(reader).map(RegularInstruction::SharedRefWithValue)
            }

            InstructionCode::CREATE_SHARED => {
                Ok(RegularInstruction::CreateShared)
            }
            InstructionCode::CREATE_SHARED_MUT => {
                Ok(RegularInstruction::CreateSharedMut)
            }

            InstructionCode::GET_INTERNAL_SLOT => {
                StackIndex::read(reader).map(RegularInstruction::GetInternalSlot)
            }

            InstructionCode::PUSH_TO_STACK => {
                Ok(RegularInstruction::PushToStack)
            }
            InstructionCode::PUSH_TO_STACK_MULTIPLE => {
                PushToStackMultiple::read(reader).map(RegularInstruction::PushToStackMultiple)
            }
            InstructionCode::CLONE_STACK_VALUE => {
                StackIndex::read(reader).map(RegularInstruction::CloneStackValue)
            }
            InstructionCode::BORROW_STACK_VALUE => {
                StackIndex::read(reader).map(RegularInstruction::BorrowStackValue)
            }
            InstructionCode::GET_STACK_VALUE_SHARED_REF => {
                StackIndex::read(reader).map(RegularInstruction::GetStackValueSharedRef)
            }
            InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT => {
                StackIndex::read(reader).map(RegularInstruction::GetStackValueSharedRefMut)
            }
            InstructionCode::TAKE_STACK_VALUE => {
                StackIndex::read(reader).map(RegularInstruction::TakeStackValue)
            }
            InstructionCode::SET_STACK_VALUE => {
                StackIndex::read(reader).map(RegularInstruction::SetStackValue)
            }

            InstructionCode::REQUEST_REMOTE_SHARED_REF => {
                RawRemotePointerAddress::read(reader).map(RegularInstruction::RequestRemoteSharedRef)
            }

            InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT => {
                RawRemotePointerAddress::read(reader).map(RegularInstruction::RequestRemoteSharedRefMut)
            }

            InstructionCode::GET_LOCAL_SHARED_REF => {
                RawLocalPointerAddress::read(reader).map(RegularInstruction::GetLocalSharedRef)
            }

            InstructionCode::GET_INTERNAL_SHARED_REF => {
                RawInternalPointerAddress::read(reader).map(RegularInstruction::GetInternalSharedRef)
            }

            InstructionCode::PERFORM_MOVE => {
                PerformMove::read(reader).map(RegularInstruction::PerformMove)
            }

            InstructionCode::MOVE => {
                Move::read(reader).map(RegularInstruction::Move)
            }

            InstructionCode::MODIFY_STACK_VALUE => {
                ModifySlot::read(reader).map(RegularInstruction::ModifyStackValue)
            }

            InstructionCode::TYPED_VALUE => {
                Ok(RegularInstruction::TypedValue)
            }
            InstructionCode::TYPE_EXPRESSION => {
                Ok(RegularInstruction::TypeExpression)
            }

            InstructionCode::RANGE => {
                Ok(RegularInstruction::Range)
            }

            InstructionCode::MODULO => todo!(),
            InstructionCode::POWER => todo!(),
            InstructionCode::AND => todo!(),
            InstructionCode::OR => todo!(),
            InstructionCode::NOT => todo!(),
            InstructionCode::INCREMENT => todo!(),
            InstructionCode::DECREMENT => todo!(),
        }
    }

    fn read_regular_instruction_code<R: Read + Seek>(
        mut reader: &mut R,
    ) -> Result<InstructionCode, DXBParserError> {
        let instruction_code = u8::read(&mut reader)
            .map_err(|_| DXBParserError::FailedToReadInstructionCode)?;

        InstructionCode::try_from(instruction_code)
            .map_err(|_| DXBParserError::InvalidInstructionCode(instruction_code))
    }

    pub fn to_formatted_string(&self) -> String {
        let mut string = String::new();
        let code = InstructionCode::from(self);
        write!(&mut string, "\x1b[38;2;39;149;245m{}\x1b[0m", code).unwrap();

        if let Some(metadata_string) = self.metadata_string() {
            write!(&mut string, " {}", metadata_string).unwrap();
        }

        string
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
                write!(string, "(arg_count: {})", count.arg_count)
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
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::DecimalF64(data) => {
                write!(
                    string,
                    "{}",
                    decimal_to_string(data.0, false)
                )
            }
            RegularInstruction::BigDecimal(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Decimal(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::ShortText(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Text(data) => {
                write!(string, "{}", data.0)
            }
            RegularInstruction::Statements(data) => {
                write!(string, "(count: {}, terminated: {})", data.statements_count, data.terminated)
            }
            RegularInstruction::ShortStatements(data) => {
                write!(string, "(count: {}, terminated: {})", data.statements_count, data.terminated)
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

            RegularInstruction::PushToStackMultiple(push_to_stack_multiple) => {
                write!(string, "{}", push_to_stack_multiple.count)
            }
            RegularInstruction::CloneStackValue(address) => {
                write!(string, "{}", address.0)
            }
            RegularInstruction::GetInternalSlot(address) => {
                write!(string, "{}", address.0)
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
            RegularInstruction::SetSharedContainerValue(set_shared_container_value) => {
                write!(string, "{}", &set_shared_container_value.operator.map(|o|o.to_string()).unwrap_or("".to_string()))
            }
            RegularInstruction::RequestRemoteSharedRef(address) => {
                write!(
                    string,
                    "({}:{})",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::RequestRemoteSharedRefMut(address) => {
                write!(
                    string,
                    "({}:{})",
                    address.endpoint().expect("Invalid endpoint"),
                    hex::encode(address.id)
                )
            }
            RegularInstruction::GetLocalSharedRef(address) => {
                write!(
                    string,
                    "(origin_id: {})",
                    hex::encode(address.bytes)
                )
            }
            RegularInstruction::GetInternalSharedRef(address) => {
                write!(
                    string,
                    "(internal_id: {})",
                    hex::encode(address.id)
                )
            }
            RegularInstruction::SharedRef(shared_ref) => {
                write!(
                    string,
                    "(ref_mutability: {:?}, address: {})",
                    shared_ref.ref_mutability, PointerAddress::from(&shared_ref.address)
                )
            }
            RegularInstruction::SharedRefWithValue(shared_ref) => {
                write!(
                    string,
                    "(ref_mutability: {:?}, address: {}, container_mutability: {:?})",
                    shared_ref.ref_mutability,
                    PointerAddress::from(&shared_ref.address),
                    shared_ref.container_mutability
                )
            }
            RegularInstruction::PerformMove(perform_move) => {
                write!(
                    string,
                    "(pointers: {})",
                    perform_move.pointers.iter().map(|(_mut, addr)| hex::encode(addr.bytes)).collect::<Vec<_>>().join(", ")
                )
            }
            RegularInstruction::Move(mv) => {
                write!(
                    string,
                    "(pointer_count: {}, mappings: {:?})",
                    mv.pointer_count, mv.address_mappings
                )
            }
            RegularInstruction::RemoteExecution(block) => {
                write!(
                    string,
                    "(length: {}, injected_slot_count: {})",
                    block.length,
                    block.injected_slot_count
                )
            }
            RegularInstruction::ModifyStackValue(modify_slot) => {
                write!(string, "{:?} {}", modify_slot.address, modify_slot.operator)
            }
            RegularInstruction::GetPropertyIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::SetPropertyIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::TakePropertyIndex(uint_32_data) => {
                write!(string, "{}", uint_32_data.0)
            }
            RegularInstruction::GetPropertyText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstruction::TakePropertyText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            RegularInstruction::SetPropertyText(short_text_data) => {
                write!(string, "{}", short_text_data.0)
            }
            _ => {
                // no custom disassembly
                return None;
            }
        }.unwrap();

        Some(string)
    }
}


impl BinRead for RegularInstruction {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let instruction_code = RegularInstruction::read_regular_instruction_code(reader)
            .map_err(|e| binrw::Error::AssertFail {
                pos: reader.stream_position().unwrap_or(0),
                message: e.to_string()
            })?;
        RegularInstruction::read_instruction(reader, instruction_code)
    }
}

impl ReadEndian for RegularInstruction {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}


impl Display for RegularInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = InstructionCode::from(self);
        write!(f, "{}", code)?;

        if let Some(metadata_string) = self.metadata_string() {
            write!(f, " {}", metadata_string)?;
        }

        Ok(())
    }
}