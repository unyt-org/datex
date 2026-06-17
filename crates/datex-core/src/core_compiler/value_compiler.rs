use crate::{
    core_compiler::type_compiler::{append_type, append_type_instruction},
    global::instruction_codes::InstructionCode,
    utils::buffers::{append_i16, append_i32, append_u8, append_u32},
    values::{
        core_value::CoreValue,
        core_values::{
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
        },
        value::Value,
        value_container::ValueContainer,
    },
};
use binrw::{BinWrite, io::Write};

use crate::{
    core_compiler::core_compilation_context::{
        ByteCursor, CoreCompilationContext,
    },
    global::protocol_structures::{
        instruction_data::{
            CallableData, Float32Data, Float64Data, Int8Data, Int16Data,
            Int32Data, Int64Data, Int128Data, ListData, MapData,
            RawPointerAddress, ShortTextData, TaggedValue, UInt8Data,
            UInt16Data, UInt32Data, UInt64Data, UInt128Data,
        },
        instructions::Instruction,
        regular_instructions::RegularInstruction,
    },
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdIndex},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    },
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainer,
        SharedContainerOwnership,
    },
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, tagged_type::TaggedTypeDefinition},
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum SharedValueCompilationError {
    ExpectedOwnedSharedValue,
    ExpectedSharedValue,
    ExpectedLocalValue,
}

impl From<SharedValueCompilationError> for ExecutionError {
    fn from(error: SharedValueCompilationError) -> ExecutionError {
        match error {
            SharedValueCompilationError::ExpectedOwnedSharedValue => {
                ExecutionError::ExpectedOwnedSharedValue
            }
            SharedValueCompilationError::ExpectedSharedValue => {
                ExecutionError::ExpectedSharedValue
            }
            SharedValueCompilationError::ExpectedLocalValue => {
                ExecutionError::ExpectedLocalValue
            }
        }
    }
}

/// Compiles a given value container to a DXB body
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
pub fn compile_value_container(
    value_container: ValueContainer,
) -> Result<Vec<u8>, SharedValueCompilationError> {
    let mut context = CoreCompilationContext::new(Vec::with_capacity(256));
    append_value_container(&mut context, value_container)?;

    Ok(context.into_buffer())
}

pub fn compile_value(
    value_container: Value,
) -> Result<Vec<u8>, SharedValueCompilationError> {
    let mut context = CoreCompilationContext::new(Vec::with_capacity(256));
    append_value(&mut context, value_container)?;

    Ok(context.into_buffer())
}

/// Appends a value container.
/// For local values, the value is just serialized
/// For shared values, the container is registered in the context shared value tracking
pub fn append_value_container(
    context: &mut CoreCompilationContext,
    value_container: ValueContainer,
) -> Result<(), SharedValueCompilationError> {
    match value_container {
        ValueContainer::Local(value) => append_value(context, value),
        ValueContainer::Shared(reference) => {
            append_inline_shared_container(context, reference);
            Ok(())
        }
    }
}

/// Appends a shared container to the buffer by registering it in the shared value tracking and appending the stack index
pub fn append_inline_shared_container(
    context: &mut CoreCompilationContext,
    shared_container: SharedContainer,
) {
    let ownership = shared_container.ownership();
    let index = context
        .shared_value_tracking
        .register_shared_value(shared_container);
    append_regular_instruction(
        context.cursor_mut(),
        match ownership {
            SharedContainerOwnership::Owned => {
                RegularInstruction::TakeStackValue(index)
            }
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => RegularInstruction::GetStackValueSharedRef(index),
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => RegularInstruction::GetStackValueSharedRefMut(index),
        },
    );
}

pub fn append_raw_pointer_address(
    cursor: &mut ByteCursor,
    raw_address: &RawPointerAddress,
) {
    cursor.write_all(&raw_address.to_bytes()).unwrap();
}

pub fn append_local_pointer_address(
    cursor: &mut ByteCursor,
    local_address: [u8; 5],
) {
    cursor.write_all(&local_address).unwrap();
}

pub fn append_value(
    context: &mut CoreCompilationContext,
    value: Value,
) -> Result<(), SharedValueCompilationError> {
    // append non-default type information
    if let Some(custom_type) = &value.custom_type {
        // special case: tagged value with default type, no type cast needed
        match custom_type {
            // unit tagged value (e.g. #Example)
            TypeDefinition::TaggedType(TaggedTypeDefinition {
                ty:
                    Some(box Type::Alias(TypeDefinitionWithMetadata {
                        definition:
                            TypeDefinition::CoreType(CoreLibTypeId::Base(
                                CoreLibBaseTypeId::Unit,
                            )),
                        ..
                    })),
                tag,
            }) => {
                append_regular_instruction(
                    context.cursor_mut(),
                    RegularInstruction::TaggedValue(TaggedValue {
                        tag: ShortTextData(tag.clone()),
                        is_empty: true,
                    }),
                );
                return Ok(()); // early return, don't append null value; TODO: assert that value is actually null?
            }
            // tagged value with actual value (e.g. #Example(null))
            TypeDefinition::TaggedType(TaggedTypeDefinition {
                ty: Option::None,
                tag,
            }) => {
                append_regular_instruction(
                    context.cursor_mut(),
                    RegularInstruction::TaggedValue(TaggedValue {
                        tag: ShortTextData(tag.clone()),
                        is_empty: false,
                    }),
                );
            }
            _ => append_type_cast(context, custom_type)?,
        }
    }
    let _: () = match value.inner {
        CoreValue::Type(ty) => {
            if let Some(core_id) = ty.try_as_core_lib_type() {
                append_get_core_lib_value(
                    context.cursor_mut(),
                    CoreLibId::Type(core_id),
                );
            } else {
                append_regular_instruction(
                    context.cursor_mut(),
                    RegularInstruction::TypeExpression,
                );
                append_type(context, &ty);
            }
        }
        CoreValue::Callable(callable) => {
            match &callable.body {
                crate::values::core_values::callable::CallableBody::Native(_) => {
                    core::todo!(
                        "Serializing native callable values is not supported"
                    );
                }
                crate::values::core_values::callable::CallableBody::DatexBytecode(body) => {
                    append_regular_instruction(
                        context.cursor_mut(),
                        RegularInstruction::Callable(
                            crate::global::protocol_structures::instruction_data::CallableData {
                                name: ShortTextData(
                                    callable.name.clone().unwrap_or_default(),
                                ),
                                parameter_count: callable.signature.parameter_types.len() as u32,
                                body_length: body.len() as u32,
                                body: body.clone(),
                            },
                        ),
                    );
                }
            }
        }
        CoreValue::Integer(integer) => {
            // NOTE: we might optimize this later, but using INT with big integer encoding
            // for all integers for now
            // let integer = integer.to_smallest_fitting();
            // append_encoded_integer(buffer, &integer);
            append_integer(context.cursor_mut(), &integer)
        }
        CoreValue::TypedInteger(integer) => {
            append_encoded_integer(context.cursor_mut(), &integer)
        }

        CoreValue::Endpoint(endpoint) => {
            append_endpoint(context.cursor_mut(), &endpoint)
        }
        CoreValue::Decimal(decimal) => {
            append_decimal(context.cursor_mut(), &decimal)
        }
        CoreValue::TypedDecimal(val) => {
            append_encoded_decimal(context.cursor_mut(), &val)
        }
        CoreValue::Boolean(val) => append_boolean(context.cursor_mut(), val.0),
        CoreValue::Null => append_regular_instruction(
            context.cursor_mut(),
            RegularInstruction::Null,
        ),
        CoreValue::Text(val) => append_text(context.cursor_mut(), &val.0),
        CoreValue::List(val) => {
            // if list size < 256, use SHORT_LIST
            match val.len() {
                0..=255 => {
                    append_instruction_code_new(
                        context.cursor_mut(),
                        InstructionCode::SHORT_LIST,
                    );
                    append_u8(context.cursor_mut(), val.len() as u8);
                }
                _ => {
                    append_regular_instruction(
                        context.cursor_mut(),
                        RegularInstruction::List(ListData {
                            element_count: val.len(),
                        }),
                    );
                }
            }

            for item in val {
                append_value_container(context, item)?;
            }
        }
        CoreValue::Map(val) => {
            // if map size < 256, use SHORT_MAP
            match val.size() {
                0..=255 => {
                    append_instruction_code_new(
                        context.cursor_mut(),
                        InstructionCode::SHORT_MAP,
                    );
                    append_u8(context.cursor_mut(), val.size() as u8);
                }
                _ => {
                    append_regular_instruction(
                        context.cursor_mut(),
                        RegularInstruction::Map(MapData {
                            element_count: val.size() as u32, // FIXME #633: casting from usize to u32 here
                        }),
                    );
                }
            }
            for (key, value) in val.into_iter() {
                append_key_value_pair(
                    context,
                    ValueContainer::from(key),
                    value,
                )?;
            }
        }
        CoreValue::Range(range) => {
            append_regular_instruction(
                context.cursor_mut(),
                RegularInstruction::Range,
            );
            append_value_container(context, *range.start)?;
            append_value_container(context, *range.end)?;
        }
        CoreValue::NominalTypeDefinition(_) => {
            todo!()
        }
    };
    Ok(())
}

pub fn append_core_type_cast(
    _context: &mut CoreCompilationContext,
    _core_lib_type_id: CoreLibTypeId,
) {
    // TODO: append type cast with only id (no need to access shared container)
    todo!()
}

pub fn append_type_cast(
    context: &mut CoreCompilationContext,
    ty: &TypeDefinition,
) -> Result<(), SharedValueCompilationError> {
    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::TypedValue,
    );

    // append type
    append_type(context, &Type::from(ty.clone()));

    Ok(())
}

pub fn append_text(cursor: &mut ByteCursor, string: &str) {
    let bytes = string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code_new(cursor, InstructionCode::SHORT_TEXT);
        append_u8(cursor, len as u8);
    } else {
        append_instruction_code_new(cursor, InstructionCode::TEXT);
        append_u32(cursor, len as u32);
    }

    cursor.write_all(&bytes[..len]).unwrap();
}

pub fn append_boolean(cursor: &mut ByteCursor, boolean: bool) {
    if boolean {
        append_regular_instruction(cursor, RegularInstruction::True)
    } else {
        append_regular_instruction(cursor, RegularInstruction::False)
    }
}

pub fn append_decimal(cursor: &mut ByteCursor, decimal: &Decimal) {
    append_instruction_code_new(cursor, InstructionCode::DECIMAL);
    append_big_decimal(cursor, decimal);
}

pub fn append_big_decimal(cursor: &mut ByteCursor, decimal: &Decimal) {
    decimal.write_le(cursor).unwrap();
}

pub fn append_endpoint(cursor: &mut ByteCursor, endpoint: &Endpoint) {
    append_instruction_code_new(cursor, InstructionCode::ENDPOINT);
    endpoint.write_le(cursor).unwrap();
}

/// Appends a typed integer with explicit type casts
pub fn append_typed_integer(
    context: &mut CoreCompilationContext,
    integer: &TypedInteger,
) {
    append_core_type_cast(context, CoreLibTypeId::from(integer));
    append_encoded_integer(context.cursor_mut(), integer);
}

/// Appends a default, unsized integer
pub fn append_integer(cursor: &mut ByteCursor, integer: &Integer) {
    append_regular_instruction(
        cursor,
        RegularInstruction::Integer(integer.clone()), // FIXME: no clone
    );
}

/// Appends an encoded integer without explicit type casts
pub fn append_encoded_integer(cursor: &mut ByteCursor, integer: &TypedInteger) {
    let instruction = match integer {
        TypedInteger::I8(val) => RegularInstruction::Int8(Int8Data(*val)),
        TypedInteger::I16(val) => RegularInstruction::Int16(Int16Data(*val)),
        TypedInteger::I32(val) => RegularInstruction::Int32(Int32Data(*val)),
        TypedInteger::I64(val) => RegularInstruction::Int64(Int64Data(*val)),
        TypedInteger::I128(val) => RegularInstruction::Int128(Int128Data(*val)),
        TypedInteger::U8(val) => RegularInstruction::UInt8(UInt8Data(*val)),
        TypedInteger::U16(val) => RegularInstruction::UInt16(UInt16Data(*val)),
        TypedInteger::U32(val) => RegularInstruction::UInt32(UInt32Data(*val)),
        TypedInteger::U64(val) => RegularInstruction::UInt64(UInt64Data(*val)),
        TypedInteger::U128(val) => {
            RegularInstruction::UInt128(UInt128Data(*val))
        }
        TypedInteger::IBig(val) => RegularInstruction::BigInteger(val.clone()), // FIXME: no clone
    };

    append_regular_instruction(cursor, instruction);
}

pub fn append_encoded_decimal(cursor: &mut ByteCursor, decimal: &TypedDecimal) {
    fn append_f32_or_f64(cursor: &mut ByteCursor, decimal: &TypedDecimal) {
        match decimal {
            TypedDecimal::F32(val) => {
                append_regular_instruction(
                    cursor,
                    RegularInstruction::DecimalF32(Float32Data(
                        val.into_inner(),
                    )),
                );
            }
            TypedDecimal::F64(val) => {
                append_regular_instruction(
                    cursor,
                    RegularInstruction::DecimalF64(Float64Data(
                        val.into_inner(),
                    )),
                );
            }
            TypedDecimal::Decimal(val) => {
                append_instruction_code_new(
                    cursor,
                    InstructionCode::DECIMAL_BIG,
                );
                append_big_decimal(cursor, val);
            }
        }
    }

    append_f32_or_f64(cursor, decimal)

    // TODO #635: maybe use this in the future, but type casts are necessary to decide which actual type is represented
    // match decimal.as_integer() {
    //     Some(int) => {
    //         let smallest = smallest_fitting_signed(int as i128);
    //         match smallest {
    //             TypedInteger::I8(val) => {
    //                 append_float_as_i16(buffer, val as i16);
    //             }
    //             TypedInteger::I16(val) => {
    //                 append_float_as_i16(buffer, val);
    //             }
    //             TypedInteger::I32(val) => {
    //                 append_float_as_i32(buffer, val);
    //             }
    //             _ => append_f32_or_f64(buffer, decimal),
    //         }
    //     }
    //     None => append_f32_or_f64(buffer, decimal),
    // }
}

pub fn append_big_integer(cursor: &mut ByteCursor, integer: &Integer) {
    integer
        .write_le(cursor)
        .expect("Failed to write big integer");
}

pub fn append_typed_decimal(
    context: &mut CoreCompilationContext,
    decimal: &TypedDecimal,
) {
    append_core_type_cast(context, CoreLibTypeId::from(decimal));
    append_encoded_decimal(context.cursor_mut(), decimal);
}

pub fn append_float_as_i16(cursor: &mut ByteCursor, int: i16) {
    append_instruction_code_new(cursor, InstructionCode::DECIMAL_AS_INT_16);
    append_i16(cursor, int);
}
pub fn append_float_as_i32(cursor: &mut ByteCursor, int: i32) {
    append_instruction_code_new(cursor, InstructionCode::DECIMAL_AS_INT_32);
    append_i32(cursor, int);
}

pub fn append_get_shared_ref(
    context: &mut CoreCompilationContext,
    address: &PointerAddress,
    mutability: &ReferenceMutability,
) {
    match address {
        PointerAddress::SelfOwned(local_address) => {
            append_instruction_code_new(
                context.cursor_mut(),
                InstructionCode::GET_LOCAL_SHARED_REF,
            );
            context
                .cursor_mut()
                .write_all(&local_address.address)
                .unwrap();
        }
        PointerAddress::Remote(address) => {
            append_instruction_code_new(
                context.cursor_mut(),
                match mutability {
                    ReferenceMutability::Immutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF
                    }
                    ReferenceMutability::Mutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT
                    }
                },
            );
            context.cursor_mut().write_all(&address.0).unwrap();
        }
    }
}

pub fn append_get_core_lib_value(cursor: &mut ByteCursor, id: CoreLibId) {
    append_regular_instruction(
        cursor,
        RegularInstruction::GetCoreLibValue(CoreLibIdIndex::from(id)),
    );
}

pub fn append_key_value_pair(
    context: &mut CoreCompilationContext,
    key: ValueContainer,
    value: ValueContainer,
) -> Result<(), SharedValueCompilationError> {
    // insert key
    match key {
        // if text, append_key_string, else dynamic
        ValueContainer::Local(Value {
            inner: CoreValue::Text(text),
            ..
        }) => {
            append_key_string(context.cursor_mut(), &text.0);
        }
        _ => {
            append_regular_instruction(
                context.cursor_mut(),
                RegularInstruction::KeyValueDynamic,
            );
            append_value_container(context, key)?;
        }
    }
    // insert value
    append_value_container(context, value)
}

pub fn append_key_string(cursor: &mut ByteCursor, key_string: &str) {
    let bytes = key_string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code_new(
            cursor,
            InstructionCode::KEY_VALUE_SHORT_TEXT,
        );
        append_u8(cursor, len as u8);
        cursor.write_all(&bytes[..len]).unwrap();
    } else {
        append_instruction_code_new(cursor, InstructionCode::KEY_VALUE_DYNAMIC);
        append_text(cursor, key_string);
    }
}

#[inline]
pub fn append_regular_instruction(
    cursor: &mut ByteCursor,
    instruction: RegularInstruction,
) {
    // add instruction code
    cursor
        .write_all(&[InstructionCode::from(&instruction) as u8])
        .unwrap();
    // add instruction
    instruction.write(cursor).unwrap();
}

pub fn append_instruction(cursor: &mut ByteCursor, instruction: Instruction) {
    match instruction {
        Instruction::Regular(instruction) => {
            append_regular_instruction(cursor, instruction)
        }
        Instruction::Type(instruction) => {
            append_type_instruction(cursor, instruction)
        }
    }
}

#[deprecated(note = "use append_regular_instruction instead")]
pub fn append_instruction_code(_buffer: &mut Vec<u8>, _code: InstructionCode) {
    unimplemented!("append_instruction_code instead")
}

pub fn append_instruction_code_new(
    cursor: &mut ByteCursor,
    code: InstructionCode,
) {
    cursor.write_all(&[code as u8]).unwrap();
}

pub fn append_statements_preamble(
    cursor: &mut ByteCursor,
    len: usize,
    is_terminated: bool,
) {
    match len {
        0..=255 => {
            append_instruction_code_new(
                cursor,
                InstructionCode::SHORT_STATEMENTS,
            );
            append_u8(cursor, len as u8);
        }
        _ => {
            append_instruction_code_new(cursor, InstructionCode::STATEMENTS);
            append_u32(
                cursor,
                len as u32, // FIXME #673: conversion from usize to u32
            );
        }
    }

    // append termination flag
    append_u8(cursor, if is_terminated { 1 } else { 0 });
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        assert_regular_instructions_equal,
        dif::pointer_address,
        global::protocol_structures::instruction_data::StackIndex,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            OwnedSharedContainer, SelfOwnedPointerAddress,
            SharedContainerMutability,
        },
        values::{core_values::list::List, value::Value},
    };
    use core::assert_matches;

    #[test]
    fn compile_tagged_empty_value() {
        let value = Value {
            inner: CoreValue::Null,
            custom_type: Some(TypeDefinition::TaggedType(
                TaggedTypeDefinition {
                    ty: Some(Box::new(Type::Alias(
                        TypeDefinition::CoreType(CoreLibTypeId::Base(
                            CoreLibBaseTypeId::Unit,
                        ))
                        .into(),
                    ))),
                    tag: "Example".to_string(),
                },
            )),
        };

        let compiled = compile_value(value).unwrap();
        assert_regular_instructions_equal!(
            &compiled,
            [RegularInstruction::TaggedValue(TaggedValue {
                tag: ShortTextData("Example".to_string()),
                is_empty: true
            })]
        );
    }

    #[test]
    fn compile_tagged_value() {
        let value = Value {
            inner: CoreValue::Null,
            custom_type: Some(TypeDefinition::TaggedType(
                TaggedTypeDefinition {
                    ty: None,
                    tag: "Example".to_string(),
                },
            )),
        };

        let compiled = compile_value(value).unwrap();
        assert_regular_instructions_equal!(
            &compiled,
            [
                RegularInstruction::TaggedValue(TaggedValue {
                    tag: ShortTextData("Example".to_string()),
                    is_empty: false,
                }),
                RegularInstruction::Null
            ]
        );
    }

    #[test]
    fn compile_core_type_value_integer() {
        let value = Value::from(CoreValue::Type(
            TypeDefinition::CoreType(CoreLibTypeId::Base(
                CoreLibBaseTypeId::Integer,
            ))
            .into(),
        ));

        let compiled = compile_value(value).unwrap();
        assert_regular_instructions_equal!(
            &compiled,
            [RegularInstruction::GetCoreLibValue(
                CoreLibBaseTypeId::Integer.into()
            )]
        );
    }

    #[test]
    fn compile_callable_value() {
        use crate::{
            types::type_definition::callable::{
                CallableKind, CallableTypeDefinition,
            },
            values::core_values::callable::{Callable, CallableBody},
        };

        let signature = CallableTypeDefinition {
            kind: CallableKind::Function,
            parameter_types: vec![],
            rest_parameter_type: None,
            return_type: None,
            yeet_type: None,
        };
        let value = Value {
            inner: CoreValue::Callable(Callable {
                name: Some("test".to_string()),
                signature,
                body: CallableBody::DatexBytecode(vec![
                    InstructionCode::UINT_8.into(),
                    42,
                ]),
            }),
            custom_type: None,
        };

        let compiled = compile_value(value).unwrap();
        assert_regular_instructions_equal!(
            &compiled,
            [RegularInstruction::Callable(CallableData {
                name: ShortTextData("test".to_string()),
                parameter_count: 0,
                body_length: 2,
                body: vec![InstructionCode::UINT_8.into(), 42],
            })]
    fn shared_value_compilation() {
        let mut provider = SelfOwnedPointerAddressProvider::default();
        let owned_shared =
            SharedContainer::new_owned_with_inferred_allowed_type(
                5,
                SharedContainerMutability::Immutable,
                &mut provider,
            );
        let pointer_address = owned_shared.pointer_address();
        let shared_container = ValueContainer::Shared(owned_shared);
        let mut context = CoreCompilationContext::new(Vec::new());

        append_value_container(&mut context, shared_container).unwrap();

        // The address should now be registered in the shared value tracking
        assert_matches!(
            context
                .shared_value_tracking
                .shared_values
                .remove(&pointer_address)
                .unwrap(),
            (SharedContainer::Owned(_), StackIndex(0))
        );

        assert_regular_instructions_equal!(
            &context.into_buffer(),
            [RegularInstruction::TakeStackValue(StackIndex(0))]
        );
    }

    #[test]
    fn shared_value_nested() {
        let mut provider = SelfOwnedPointerAddressProvider::default();
        let inner_shared =
            SharedContainer::new_owned_with_inferred_allowed_type(
                5,
                SharedContainerMutability::Mutable,
                &mut provider,
            );
        let outer_shared =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::Shared(SharedContainer::Referenced(
                    inner_shared.try_derive_mutable_reference().unwrap(),
                )),
                SharedContainerMutability::Immutable,
                &mut provider,
            );
        let outer_pointer_address = outer_shared.pointer_address();
        let shared_container = ValueContainer::Shared(outer_shared);
        let mut context = CoreCompilationContext::new(Vec::new());

        append_value_container(&mut context, shared_container).unwrap();

        assert_matches!(
            context
                .shared_value_tracking
                .shared_values
                .remove(&outer_pointer_address)
                .unwrap(),
            (SharedContainer::Owned(_), StackIndex(0))
        );

        assert_regular_instructions_equal!(
            &context.into_buffer(),
            [RegularInstruction::TakeStackValue(StackIndex(0)),]
        );
    }

    #[test]
    fn shared_ref() {
        let mut provider = SelfOwnedPointerAddressProvider::default();
        let reference = SharedContainer::new_owned_with_inferred_allowed_type(
            5,
            SharedContainerMutability::Immutable,
            &mut provider,
        )
        .derive_immutable_reference();
        let pointer_address = reference.pointer_address();
        let shared_container =
            ValueContainer::Shared(SharedContainer::Referenced(reference));
        let mut context = CoreCompilationContext::new(Vec::new());

        append_value_container(&mut context, shared_container).unwrap();

        assert_matches!(
            context
                .shared_value_tracking
                .shared_values
                .remove(&pointer_address)
                .unwrap(),
            (SharedContainer::Referenced(_), StackIndex(0))
        );

        assert_regular_instructions_equal!(
            &context.into_buffer(),
            [RegularInstruction::GetStackValueSharedRef(StackIndex(0))]
        );
    }

    #[test]
    fn local_nested() {
        let mut provider = SelfOwnedPointerAddressProvider::default();
        let a_shared = SharedContainer::new_owned_with_inferred_allowed_type(
            1,
            SharedContainerMutability::Immutable,
            &mut provider,
        );
        let a_pointer_address = a_shared.pointer_address();
        let b_shared = SharedContainer::new_owned_with_inferred_allowed_type(
            2,
            SharedContainerMutability::Immutable,
            &mut provider,
        );
        let b_pointer_address = b_shared.pointer_address();
        let local = ValueContainer::Local(
            List::new(vec![
                ValueContainer::Shared(a_shared),
                ValueContainer::Shared(b_shared),
            ])
            .into(),
        );
        let mut context = CoreCompilationContext::new(Vec::new());

        append_value_container(&mut context, local).unwrap();

        assert_matches!(
            context
                .shared_value_tracking
                .shared_values
                .remove(&a_pointer_address)
                .unwrap(),
            (SharedContainer::Owned(_), StackIndex(0))
        );
        assert_matches!(
            context
                .shared_value_tracking
                .shared_values
                .remove(&b_pointer_address)
                .unwrap(),
            (SharedContainer::Owned(_), StackIndex(1))
        );

        assert_regular_instructions_equal!(
            &context.into_buffer(),
            [
                RegularInstruction::ShortList(ListData { element_count: 2 }),
                RegularInstruction::TakeStackValue(StackIndex(0)),
                RegularInstruction::TakeStackValue(StackIndex(1)),
            ]
        );
    }
}
