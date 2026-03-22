use crate::{
    core_compiler::type_compiler::{
        append_type, append_type_definition, append_type_metadata,
        append_type_space_instruction_code,
    },
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::instructions::TypeMetadataBin,
        type_instruction_codes::TypeInstructionCode,
    },
    libs::core::{CoreLibPointerId, get_core_lib_type_definition},
    shared_values::shared_container::SharedContainerMutability,
    types::definition::TypeDefinition,
    utils::buffers::{
        append_f32, append_f64, append_i8, append_i16, append_i32, append_i64,
        append_i128, append_u8, append_u16, append_u32, append_u64,
        append_u128,
    },
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
use binrw::{BinWrite, io::Cursor};

use crate::{
    prelude::*,
    shared_values::{
        pointer::PointerReferenceMutability,
        pointer_address::{PointerAddress, ReferencedPointerAddress},
    },
    values::core_values::r#type::TypeMetadata,
};
use crate::global::protocol_structures::instructions::{RawPointerAddress};
use crate::shared_values::shared_container::SharedContainer;

/// Compiles a given value container to a DXB body
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
pub fn compile_value_container(value_container: &ValueContainer) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(256);
    append_value_container(&mut buffer, value_container);

    buffer
}

pub fn compile_value(value_container: &Value) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(256);
    append_value(&mut buffer, value_container);

    buffer
}

pub fn compile_shared_container(shared_container: &SharedContainer, insert_value: bool) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(256);
    append_shared_container(&mut buffer, shared_container, insert_value);

    buffer
}


/// Appends a value container.
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
fn append_value_container_inner(
    buffer: &mut Vec<u8>,
    value_container: &ValueContainer,
) {
    match value_container {
        ValueContainer::Local(value) => append_value(buffer, value),
        ValueContainer::Shared(reference) => {
            append_shared_container_as_ref(buffer, &reference, true);
        }
    }
}


/// Appends a value container.
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
pub fn append_value_container(
    buffer: &mut Vec<u8>,
    value_container: &ValueContainer,
) {
    match value_container {
        ValueContainer::Local(value) => append_value(buffer, &value),
        ValueContainer::Shared(reference) => {
            append_shared_container(buffer, &reference, true);
        }
    }
}

/// Appends a shared container to the buffer a reference
pub fn append_shared_container_as_ref(
    buffer: &mut Vec<u8>,
    shared_container: &SharedContainer,
    insert_value: bool,
) {
    append_shared_container(buffer, &shared_container.derive_with_max_mutability(), insert_value)
}

/// Appends a shared container to the buffer, with optional mutability information for the shared container
/// If shared_container_mutability is None, a move is performed
/// If force_reference is set to true, no move is performed, even if the shared_container is owned - instead
/// the container is transferred as a reference with maximum mutability
/// TODO: set insert_value only if for remote execution and not already on remote endpoint
pub fn append_shared_container(
    buffer: &mut Vec<u8>,
    shared_container: &SharedContainer,
    insert_value: bool,
) {
    match &shared_container.reference_mutability {
        Some(mutability) => {
            append_instruction_code(buffer, if insert_value {
                InstructionCode::SHARED_REF_WITH_VALUE
            } else {
                InstructionCode::SHARED_REF
            });

            // insert address
            match shared_container.pointer().address() {
                // only for SHARED_REF_WITH_VALUE
                PointerAddress::Owned(owned_address) if insert_value => {
                    append_local_pointer_address(buffer, owned_address.address);
                }
                address => {
                    let raw_address = RawPointerAddress::from(address);
                    append_raw_pointer_address(buffer, &raw_address);
                }
            }

            // flag indicating reference mutability
            append_u8(buffer, *mutability as u8);

            // insert value with container mutability
            if insert_value {
                // flag indicating container mutability
                append_u8(buffer, shared_container.mutability() as u8);
                append_value(buffer, &shared_container.collapse_to_value().borrow())
            }
        },
        None => {
            append_instruction_code(buffer, InstructionCode::TAKE_PROPERTY_INDEX);
            append_u32(buffer, 0); // list index 0 (only moving a single pointer)
            append_perform_moves(buffer, &[shared_container]).unwrap();
        },
    };
}

/// Appends multiple shared containers as moves to the buffer
/// TODO: Also handle moves of nested shared values!
pub fn append_perform_moves(
    buffer: &mut Vec<u8>,
    shared_containers: &[&SharedContainer],
) -> Result<(), ()> {
    append_instruction_code(buffer, InstructionCode::PERFORM_MOVE);
    append_u32(buffer, shared_containers.len() as u32); // number of moved values
    for shared_container in shared_containers {
        if let Some(local_address) = shared_container.try_get_owned_local_address() {
            append_u8(buffer, if shared_container.is_mutable() {1} else {0});
            append_local_pointer_address(buffer, local_address);
        }
        else {
            return Err(());
        }
    }
    Ok(())
}



pub fn append_raw_pointer_address(buffer: &mut Vec<u8>, raw_address: &RawPointerAddress) {
    buffer.extend_from_slice(&raw_address.to_bytes());
}

pub fn append_local_pointer_address(buffer: &mut Vec<u8>, local_address: [u8; 5]) {
    buffer.extend_from_slice(&local_address);
}


pub fn append_value(buffer: &mut Vec<u8>, value: &Value) {
    // append non-default type information
    if !value.has_default_type() {
        append_type_cast(buffer, &value.actual_type);
    }
    match &value.inner {
        CoreValue::Type(_ty) => {
            core::todo!("#439 Type value not supported in CompilationContext");
        }
        CoreValue::Callable(_callable) => {
            core::todo!(
                "#632 Callable value not supported in CompilationContext"
            );
        }
        CoreValue::Integer(integer) => {
            // NOTE: we might optimize this later, but using INT with big integer encoding
            // for all integers for now
            // let integer = integer.to_smallest_fitting();
            // append_encoded_integer(buffer, &integer);
            append_integer(buffer, integer);
        }
        CoreValue::TypedInteger(integer) => {
            append_encoded_integer(buffer, integer);
        }

        CoreValue::Endpoint(endpoint) => append_endpoint(buffer, endpoint),
        CoreValue::Decimal(decimal) => append_decimal(buffer, decimal),
        CoreValue::TypedDecimal(val) => append_encoded_decimal(buffer, val),
        CoreValue::Boolean(val) => append_boolean(buffer, val.0),
        CoreValue::Null => {
            append_instruction_code(buffer, InstructionCode::NULL)
        }
        CoreValue::Text(val) => {
            append_text(buffer, &val.0);
        }
        CoreValue::List(val) => {
            // if list size < 256, use SHORT_LIST
            match val.len() {
                0..=255 => {
                    append_instruction_code(
                        buffer,
                        InstructionCode::SHORT_LIST,
                    );
                    append_u8(buffer, val.len() as u8);
                }
                _ => {
                    append_instruction_code(buffer, InstructionCode::LIST);
                    append_u32(buffer, val.len());
                }
            }

            for item in val {
                append_value_container(buffer, item.into());
            }
        }
        CoreValue::Map(val) => {
            // if map size < 256, use SHORT_MAP
            match val.size() {
                0..=255 => {
                    append_instruction_code(buffer, InstructionCode::SHORT_MAP);
                    append_u8(buffer, val.size() as u8);
                }
                _ => {
                    append_instruction_code(buffer, InstructionCode::MAP);
                    append_u32(buffer, val.size() as u32); // FIXME #633: casting from usize to u32 here
                }
            }
            for (key, value) in val.iter() {
                append_key_value_pair(
                    buffer,
                    &ValueContainer::from(key),
                    value,
                );
            }
        }
        CoreValue::Range(range) => {
            append_instruction_code(buffer, InstructionCode::RANGE);
            append_value_container(buffer, (&*range.start).into());
            append_value_container(buffer, (&*range.end).into());
        }
    }
}

pub fn append_type_cast(buffer: &mut Vec<u8>, ty: &TypeDefinition) {
    append_instruction_code(buffer, InstructionCode::TYPED_VALUE);
    // append instruction code
    let instruction_code = TypeInstructionCode::from(ty);
    append_type_space_instruction_code(buffer, instruction_code);

    // append type information for non-core types
    let metadata = TypeMetadataBin::from(&TypeMetadata::default());
    append_type_metadata(buffer, metadata);

    // append type definition
    append_type_definition(buffer, ty);
}

pub fn append_text(buffer: &mut Vec<u8>, string: &str) {
    let bytes = string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code(buffer, InstructionCode::SHORT_TEXT);
        append_u8(buffer, len as u8);
    } else {
        append_instruction_code(buffer, InstructionCode::TEXT);
        append_u32(buffer, len as u32);
    }

    buffer.extend_from_slice(bytes);
}

pub fn append_boolean(buffer: &mut Vec<u8>, boolean: bool) {
    if boolean {
        append_instruction_code(buffer, InstructionCode::TRUE);
    } else {
        append_instruction_code(buffer, InstructionCode::FALSE);
    }
}

pub fn append_decimal(buffer: &mut Vec<u8>, decimal: &Decimal) {
    append_instruction_code(buffer, InstructionCode::DECIMAL);
    append_big_decimal(buffer, decimal);
}

pub fn append_big_decimal(buffer: &mut Vec<u8>, decimal: &Decimal) {
    // big_decimal binrw write into buffer
    let original_length = buffer.len();
    let mut buffer_writer = Cursor::new(&mut *buffer);
    // set writer position to end
    buffer_writer.set_position(original_length as u64);
    decimal
        .write_le(&mut buffer_writer)
        .expect("Failed to write big decimal");
}

pub fn append_endpoint(buffer: &mut Vec<u8>, endpoint: &Endpoint) {
    append_instruction_code(buffer, InstructionCode::ENDPOINT);
    buffer.extend_from_slice(&endpoint.to_slice());
}

/// Appends a typed integer with explicit type casts
pub fn append_typed_integer(buffer: &mut Vec<u8>, integer: &TypedInteger) {
    append_type_cast(
        buffer,
        &get_core_lib_type_definition(CoreLibPointerId::from(integer)),
    );
    append_encoded_integer(buffer, integer);
}

/// Appends a default, unsized integer
pub fn append_integer(buffer: &mut Vec<u8>, integer: &Integer) {
    append_instruction_code(buffer, InstructionCode::INT);
    append_big_integer(buffer, integer);
}

/// Appends an encoded integer without explicit type casts
pub fn append_encoded_integer(buffer: &mut Vec<u8>, integer: &TypedInteger) {
    match integer {
        TypedInteger::I8(val) => {
            append_instruction_code(buffer, InstructionCode::INT_8);
            append_i8(buffer, *val);
        }
        TypedInteger::I16(val) => {
            append_instruction_code(buffer, InstructionCode::INT_16);
            append_i16(buffer, *val);
        }
        TypedInteger::I32(val) => {
            append_instruction_code(buffer, InstructionCode::INT_32);
            append_i32(buffer, *val);
        }
        TypedInteger::I64(val) => {
            append_instruction_code(buffer, InstructionCode::INT_64);
            append_i64(buffer, *val);
        }
        TypedInteger::I128(val) => {
            append_instruction_code(buffer, InstructionCode::INT_128);
            append_i128(buffer, *val);
        }
        TypedInteger::U8(val) => {
            append_instruction_code(buffer, InstructionCode::UINT_8);
            append_u8(buffer, *val);
        }
        TypedInteger::U16(val) => {
            append_instruction_code(buffer, InstructionCode::UINT_16);
            append_u16(buffer, *val);
        }
        TypedInteger::U32(val) => {
            append_instruction_code(buffer, InstructionCode::UINT_32);
            append_u32(buffer, *val);
        }
        TypedInteger::U64(val) => {
            append_instruction_code(buffer, InstructionCode::UINT_64);
            append_u64(buffer, *val);
        }
        TypedInteger::U128(val) => {
            append_instruction_code(buffer, InstructionCode::UINT_128);
            append_u128(buffer, *val);
        }
        TypedInteger::IBig(val) => {
            append_instruction_code(buffer, InstructionCode::INT_BIG);
            append_big_integer(buffer, val);
        }
    }
}

pub fn append_encoded_decimal(buffer: &mut Vec<u8>, decimal: &TypedDecimal) {
    fn append_f32_or_f64(buffer: &mut Vec<u8>, decimal: &TypedDecimal) {
        match decimal {
            TypedDecimal::F32(val) => {
                append_float32(buffer, val.into_inner());
            }
            TypedDecimal::F64(val) => {
                append_float64(buffer, val.into_inner());
            }
            TypedDecimal::Decimal(val) => {
                append_instruction_code(buffer, InstructionCode::DECIMAL_BIG);
                append_big_decimal(buffer, val);
            }
        }
    }

    append_f32_or_f64(buffer, decimal);

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

pub fn append_float32(buffer: &mut Vec<u8>, float32: f32) {
    append_instruction_code(buffer, InstructionCode::DECIMAL_F32);
    append_f32(buffer, float32);
}
pub fn append_float64(buffer: &mut Vec<u8>, float64: f64) {
    append_instruction_code(buffer, InstructionCode::DECIMAL_F64);
    append_f64(buffer, float64);
}

pub fn append_big_integer(buffer: &mut Vec<u8>, integer: &Integer) {
    // use BinWrite to write the integer to the buffer
    // big_integer binrw write into buffer
    let original_length = buffer.len();
    let mut buffer_writer = Cursor::new(&mut *buffer);
    // set writer position to end
    buffer_writer.set_position(original_length as u64);
    integer
        .write_le(&mut buffer_writer)
        .expect("Failed to write big integer");
}

pub fn append_typed_decimal(buffer: &mut Vec<u8>, decimal: &TypedDecimal) {
    append_type_cast(
        buffer,
        &get_core_lib_type_definition(CoreLibPointerId::from(decimal)),
    );
    append_encoded_decimal(buffer, decimal);
}

pub fn append_float_as_i16(buffer: &mut Vec<u8>, int: i16) {
    append_instruction_code(buffer, InstructionCode::DECIMAL_AS_INT_16);
    append_i16(buffer, int);
}
pub fn append_float_as_i32(buffer: &mut Vec<u8>, int: i32) {
    append_instruction_code(buffer, InstructionCode::DECIMAL_AS_INT_32);
    append_i32(buffer, int);
}

pub fn append_get_shared_ref(
    buffer: &mut Vec<u8>,
    address: &PointerAddress,
    mutability: &PointerReferenceMutability,
) {
    match address {
        PointerAddress::Referenced(ReferencedPointerAddress::Internal(id)) => {
            append_get_internal_ref(buffer, id);
        }
        PointerAddress::Owned(local_address) => {
            append_instruction_code(
                buffer,
                InstructionCode::GET_LOCAL_SHARED_REF,
            );
            buffer.extend_from_slice(&local_address.address);
        }
        PointerAddress::Referenced(ReferencedPointerAddress::Remote(id)) => {
            append_instruction_code(
                buffer,
                match mutability {
                    PointerReferenceMutability::Immutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF
                    }
                    PointerReferenceMutability::Mutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT
                    }
                },
            );
            buffer.extend_from_slice(id);
        }
    }
}

pub fn append_get_internal_ref(buffer: &mut Vec<u8>, id: &[u8; 3]) {
    append_instruction_code(buffer, InstructionCode::GET_INTERNAL_SHARED_REF);
    buffer.extend_from_slice(id);
}

pub fn append_key_value_pair(
    buffer: &mut Vec<u8>,
    key: &ValueContainer,
    value: &ValueContainer,
) {
    // insert key
    match key {
        // if text, append_key_string, else dynamic
        ValueContainer::Local(Value {
            inner: CoreValue::Text(text),
            ..
        }) => {
            append_key_string(buffer, &text.0);
        }
        _ => {
            append_instruction_code(buffer, InstructionCode::KEY_VALUE_DYNAMIC);
            append_value_container(buffer, key);
        }
    }
    // insert value
    append_value_container(buffer, value);
}

pub fn append_key_string(buffer: &mut Vec<u8>, key_string: &str) {
    let bytes = key_string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code(buffer, InstructionCode::KEY_VALUE_SHORT_TEXT);
        append_u8(buffer, len as u8);
        buffer.extend_from_slice(bytes);
    } else {
        append_instruction_code(buffer, InstructionCode::KEY_VALUE_DYNAMIC);
        append_text(buffer, key_string);
    }
}

pub fn append_instruction_code(buffer: &mut Vec<u8>, code: InstructionCode) {
    append_u8(buffer, code as u8);
}


pub fn append_statements_preamble(buffer: &mut Vec<u8>, len: usize, is_terminated: bool) {
    match len {
        0..=255 => {
            append_instruction_code(
                buffer,
                InstructionCode::SHORT_STATEMENTS,
            );
            append_u8(
                buffer,
                len as u8,
            );
        }
        _ => {
            append_instruction_code(
                buffer,
                InstructionCode::STATEMENTS,
            );
            append_u32(
                buffer,
                len as u32, // FIXME #673: conversion from usize to u32
            );
        }
    }

    // append termination flag
    append_u8(
        buffer,
        if is_terminated { 1 } else { 0 },
    );
}