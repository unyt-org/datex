use crate::{
    core_compiler::type_compiler::{
        append_type_definition, append_type_metadata,
    },
    global::{
        instruction_codes::InstructionCode,
        type_instruction_codes::TypeInstructionCode,
    },
    libs::core::{CoreLibPointerId, get_core_lib_type_definition},
    types::definition::TypeDefinition,
    utils::buffers::{
        append_i16, append_i32, append_u8, append_u32
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
use binrw::{BinWrite, io::Cursor, BinResult};
use binrw::io::Write;

use crate::{
    prelude::*,
    shared_values::{
        pointer::PointerReferenceMutability,
        pointer_address::{PointerAddress, ReferencedPointerAddress},
    },
    values::core_values::r#type::TypeMetadata,
};
use crate::core_compiler::core_compilation_context::{ByteCursor, CoreCompilationContext};
use crate::core_compiler::type_compiler::{append_type_instruction, append_type_space_instruction_code_new};
use crate::global::protocol_structures::instruction_data::{Float32Data, Float64Data, Int128Data, Int16Data, Int32Data, Int64Data, Int8Data, IntegerData, ListData, MapData, RawLocalPointerAddress, RawPointerAddress, SharedRef, SharedRefWithValue, TypeMetadataBin, UInt128Data, UInt16Data, UInt32Data, UInt64Data, UInt8Data};
use crate::global::protocol_structures::instructions::Instruction;
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::shared_values::shared_container::SharedContainer;

/// Compiles a given value container to a DXB body
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
pub fn compile_value_container(value_container: &ValueContainer) -> BinResult<Vec<u8>> {
    let mut context = Cursor::new(Vec::with_capacity(256));
    append_value_container(&mut context, value_container)?;

    Ok(context.into_inner())
}

pub fn compile_value(value_container: &Value) -> BinResult<Vec<u8>> {
    let mut context = Cursor::new(Vec::with_capacity(256));
    append_value(&mut context, value_container)?;

    Ok(context.into_inner())
}

pub fn compile_shared_container(shared_container: &SharedContainer, insert_value: bool) -> BinResult<Vec<u8>>  {
    let mut context = Cursor::new(Vec::with_capacity(256));
    append_shared_container(&mut context, shared_container, insert_value)?;
    Ok(context.into_inner())
}


/// Appends a value container.
/// For local values, the value is just serialized
/// For shared values, a reference with maximum mutability is serialized (no move)
pub fn append_value_container(
    context: &mut CoreCompilationContext,
    value_container: &ValueContainer,
) -> BinResult<()> {
    match value_container {
        ValueContainer::Local(value) => append_value(context, &value)?,
        ValueContainer::Shared(reference) => {
            append_shared_container(context, &reference, true)?;
        }
    }
    Ok(())
}

/// Appends a shared container to the buffer a reference
pub fn append_shared_container_as_ref(
    context: &mut CoreCompilationContext,
    shared_container: &SharedContainer,
    insert_value: bool,
) -> BinResult<()> {
    append_shared_container(context, &shared_container.derive_with_max_mutability(), insert_value)
}

/// Appends a shared container to the buffer, with optional mutability information for the shared container
/// If shared_container_mutability is None, a move is performed
/// If force_reference is set to true, no move is performed, even if the shared_container is owned - instead
/// the container is transferred as a reference with maximum mutability
/// TODO: set insert_value only if for remote execution and not already on remote endpoint
pub fn append_shared_container(
    context: &mut CoreCompilationContext,
    shared_container: &SharedContainer,
    insert_value: bool,
) -> BinResult<()> {
    match &shared_container.reference_mutability {
        // ref
        Some(mutability) => {
            match shared_container.pointer().address() {
                PointerAddress::Owned(owned_address) => {
                    // owned ref + value
                    if insert_value {
                        append_regular_instruction(
                            context.cursor_mut(),
                            RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                                address: RawLocalPointerAddress { bytes: owned_address.address},
                                container_mutability: shared_container.mutability(),
                                ref_mutability: *mutability,
                            })
                        )?;

                        // insert value with container mutability
                        if insert_value {
                            append_value(context, &shared_container.collapse_to_value().borrow())?;
                        }
                    }
                    // owned ref without value
                    else {
                        append_regular_instruction(
                            context.cursor_mut(),
                            RegularInstruction::SharedRef(SharedRef {
                                address: RawPointerAddress::Local(RawLocalPointerAddress { bytes: owned_address.address}),
                                ref_mutability: *mutability,
                            })
                        )?;
                    }
                }
                address => {
                    if insert_value {
                        return Err(binrw::Error::AssertFail {
                            pos: context.cursor_mut().position(),
                            message: "Cannot insert value for non-owned shared container".to_string(),
                        }); // not allowed for non-owned pointer to share ref with value
                    }
                    append_regular_instruction(
                        context.cursor_mut(),
                        RegularInstruction::SharedRef(SharedRef {
                            address: RawPointerAddress::from(address),
                            ref_mutability: *mutability,
                        })
                    )?;
                }
            };

            Ok(())
        },
        None => {
            // FIXME
            append_instruction_code_new(context.cursor_mut(), InstructionCode::TAKE_PROPERTY_INDEX);
            append_u32(context.cursor_mut(), 0); // list index 0 (only moving a single pointer)
            append_perform_moves(context, &[shared_container]).unwrap();

            Ok(())
        },
    }
}

/// Appends multiple shared containers as moves to the buffer
/// TODO: Also handle moves of nested shared values!
pub fn append_perform_moves(
    context: &mut CoreCompilationContext,
    shared_containers: &[&SharedContainer],
) -> Result<(), ()> {
    append_instruction_code_new(context.cursor_mut(), InstructionCode::PERFORM_MOVE);
    append_u32(context.cursor_mut(), shared_containers.len() as u32); // number of moved values
    for shared_container in shared_containers {
        if let Some(local_address) = shared_container.try_get_owned_local_address() {
            append_u8(context.cursor_mut(), if shared_container.is_mutable() {1} else {0});
            append_local_pointer_address(context.cursor_mut(), local_address);
        }
        else {
            return Err(());
        }
    }
    Ok(())
}



pub fn append_raw_pointer_address(cursor: &mut ByteCursor, raw_address: &RawPointerAddress) {
    cursor.write_all(&raw_address.to_bytes()).unwrap();
}

pub fn append_local_pointer_address(cursor: &mut ByteCursor, local_address: [u8; 5]) {
    cursor.write_all(&local_address).unwrap();
}


pub fn append_value(context: &mut CoreCompilationContext, value: &Value) -> BinResult<()> {
    // append non-default type information
    if !value.has_default_type() {
        append_type_cast(context, &value.actual_type)?;
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
            append_integer(context.cursor_mut(), integer)
        }
        CoreValue::TypedInteger(integer) => {
            append_encoded_integer(context.cursor_mut(), integer)
        }

        CoreValue::Endpoint(endpoint) => append_endpoint(context.cursor_mut(), endpoint),
        CoreValue::Decimal(decimal) => append_decimal(context.cursor_mut(), decimal),
        CoreValue::TypedDecimal(val) => append_encoded_decimal(context.cursor_mut(), val),
        CoreValue::Boolean(val) => append_boolean(context.cursor_mut(), val.0),
        CoreValue::Null => append_regular_instruction(context.cursor_mut(), RegularInstruction::Null),
        CoreValue::Text(val) => {
            append_text(context.cursor_mut(), &val.0)
        }
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
                        })
                    )?;
                }
            }

            for item in val {
                append_value_container(context, item.into())?;
            }

            Ok(())
        }
        CoreValue::Map(val) => {
            // if map size < 256, use SHORT_MAP
            match val.size() {
                0..=255 => {
                    append_instruction_code_new(context.cursor_mut(), InstructionCode::SHORT_MAP);
                    append_u8(context.cursor_mut(), val.size() as u8);
                }
                _ => {
                    append_regular_instruction(
                        context.cursor_mut(),
                        RegularInstruction::Map(MapData {
                            element_count: val.size() as u32, // FIXME #633: casting from usize to u32 here
                        })
                    )?;
                }
            }
            for (key, value) in val.iter() {
                append_key_value_pair(
                    context,
                    &ValueContainer::from(key),
                    value,
                )?;
            }

            Ok(())
        }
        CoreValue::Range(range) => {
            append_regular_instruction(context.cursor_mut(), RegularInstruction::Range)?;
            append_value_container(context, (&*range.start).into())?;
            append_value_container(context, (&*range.end).into())?;
            Ok(())
        }
    }
}

pub fn append_type_cast(context: &mut CoreCompilationContext, ty: &TypeDefinition) -> BinResult<()> {
    append_regular_instruction(context.cursor_mut(), RegularInstruction::TypedValue)?;

    // append type instruction
    let instruction_code = TypeInstructionCode::from(ty);
    append_type_space_instruction_code_new(context.cursor_mut(), instruction_code);

    // append type information for non-core types
    let metadata = TypeMetadataBin::from(&TypeMetadata::default());
    append_type_metadata(context.cursor_mut(), metadata);

    // append type definition
    append_type_definition(context, ty);

    Ok(())
}

pub fn append_text(cursor: &mut ByteCursor, string: &str) -> BinResult<()> {
    let bytes = string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code_new(cursor, InstructionCode::SHORT_TEXT);
        append_u8(cursor, len as u8);
    } else {
        append_instruction_code_new(cursor, InstructionCode::TEXT);
        append_u32(cursor, len as u32);
    }

    cursor.write_all(&bytes[..len])?;

    Ok(())
}

pub fn append_boolean(cursor: &mut ByteCursor, boolean: bool) -> BinResult<()> {
    if boolean {
        append_regular_instruction(cursor, RegularInstruction::True)
    } else {
        append_regular_instruction(cursor, RegularInstruction::False)
    }
}

pub fn append_decimal(cursor: &mut ByteCursor, decimal: &Decimal) -> BinResult<()> {
    append_instruction_code_new(cursor, InstructionCode::DECIMAL);
    append_big_decimal(cursor, decimal)
}

pub fn append_big_decimal(cursor: &mut ByteCursor, decimal: &Decimal) -> BinResult<()> {
    decimal.write_le(cursor)
}

pub fn append_endpoint(cursor: &mut ByteCursor, endpoint: &Endpoint) -> BinResult<()> {
    append_instruction_code_new(cursor, InstructionCode::ENDPOINT);
    endpoint.write_le(cursor)
}

/// Appends a typed integer with explicit type casts
pub fn append_typed_integer(context: &mut CoreCompilationContext, integer: &TypedInteger) -> BinResult<()> {
    append_type_cast(
        context,
        &get_core_lib_type_definition(CoreLibPointerId::from(integer)),
    )?;
    append_encoded_integer(context.cursor_mut(), integer)
}

/// Appends a default, unsized integer
pub fn append_integer(cursor: &mut ByteCursor, integer: &Integer) -> BinResult<()> {
    append_regular_instruction(
        cursor,
        RegularInstruction::Integer(IntegerData(integer.clone())) // FIXME: no clone
    )
}

/// Appends an encoded integer without explicit type casts
pub fn append_encoded_integer(cursor: &mut ByteCursor, integer: &TypedInteger) -> BinResult<()> {
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
        TypedInteger::U128(val) => RegularInstruction::UInt128(UInt128Data(*val)),
        TypedInteger::IBig(val) => RegularInstruction::BigInteger(IntegerData(val.clone())), // FIXME: no clone
    };

    append_regular_instruction(
        cursor,
        instruction,
    )
}

pub fn append_encoded_decimal(cursor: &mut ByteCursor, decimal: &TypedDecimal) -> BinResult<()> {
    fn append_f32_or_f64(cursor: &mut ByteCursor, decimal: &TypedDecimal) -> BinResult<()> {
        match decimal {
            TypedDecimal::F32(val) => {
                append_regular_instruction(
                    cursor,
                    RegularInstruction::DecimalF32(Float32Data(val.into_inner())),
                )
            }
            TypedDecimal::F64(val) => {
                append_regular_instruction(
                    cursor,
                    RegularInstruction::DecimalF64(Float64Data(val.into_inner())),
                )
            }
            TypedDecimal::Decimal(val) => {
                append_instruction_code_new(cursor, InstructionCode::DECIMAL_BIG);
                append_big_decimal(cursor, val)
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

pub fn append_typed_decimal(context: &mut CoreCompilationContext, decimal: &TypedDecimal) -> BinResult<()> {
    append_type_cast(
        context,
        &get_core_lib_type_definition(CoreLibPointerId::from(decimal)),
    )?;
    append_encoded_decimal(context.cursor_mut(), decimal)
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
    mutability: &PointerReferenceMutability,
) {
    match address {
        PointerAddress::Referenced(ReferencedPointerAddress::Internal(id)) => {
            append_get_internal_ref(context.cursor_mut(), id);
        }
        PointerAddress::Owned(local_address) => {
            append_instruction_code_new(
                context.cursor_mut(),
                InstructionCode::GET_LOCAL_SHARED_REF,
            );
            context.cursor_mut().write_all(&local_address.address).unwrap();
        }
        PointerAddress::Referenced(ReferencedPointerAddress::Remote(id)) => {
            append_instruction_code_new(
                context.cursor_mut(),
                match mutability {
                    PointerReferenceMutability::Immutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF
                    }
                    PointerReferenceMutability::Mutable => {
                        InstructionCode::REQUEST_REMOTE_SHARED_REF_MUT
                    }
                },
            );
            context.cursor_mut().write_all(id).unwrap();
        }
    }
}

pub fn append_get_internal_ref(cursor: &mut ByteCursor, id: &[u8; 3]) {
    append_instruction_code_new(cursor, InstructionCode::GET_INTERNAL_SHARED_REF);
    cursor.write_all(id).unwrap();
}

pub fn append_key_value_pair(
    context: &mut CoreCompilationContext,
    key: &ValueContainer,
    value: &ValueContainer,
) -> BinResult<()> {
    // insert key
    match key {
        // if text, append_key_string, else dynamic
        ValueContainer::Local(Value {
            inner: CoreValue::Text(text),
            ..
        }) => {
            append_key_string(context.cursor_mut(), &text.0)?;
        }
        _ => {
            append_regular_instruction(context.cursor_mut(), RegularInstruction::KeyValueDynamic)?;
            append_value_container(context, key)?;
        }
    }
    // insert value
    append_value_container(context, value)
}

pub fn append_key_string(cursor: &mut ByteCursor, key_string: &str) -> BinResult<()> {
    let bytes = key_string.as_bytes();
    let len = bytes.len();

    if len < 256 {
        append_instruction_code_new(cursor, InstructionCode::KEY_VALUE_SHORT_TEXT);
        append_u8(cursor, len as u8);
        cursor.write_all(&bytes[..len])?;
        Ok(())
    } else {
        append_instruction_code_new(cursor, InstructionCode::KEY_VALUE_DYNAMIC);
        append_text(cursor, key_string)
    }
}

pub fn append_regular_instruction(cursor: &mut ByteCursor, instruction: RegularInstruction) -> BinResult<()> {
    // add instruction code
    cursor.write_all(&[InstructionCode::from(&instruction) as u8])?;
    // add instruction
    instruction.write(cursor)?;
    Ok(())
}

pub fn append_instruction(cursor: &mut ByteCursor, instruction: Instruction) -> BinResult<()> {
    match instruction {
        Instruction::Regular(instruction) => append_regular_instruction(cursor, instruction),
        Instruction::Type(instruction) => append_type_instruction(cursor, instruction),
    }
}

#[deprecated(note = "use append_regular_instruction instead")]
pub fn append_instruction_code(buffer: &mut Vec<u8>, code: InstructionCode) {
    unimplemented!("append_instruction_code instead")
}

pub fn append_instruction_code_new(cursor: &mut ByteCursor, code: InstructionCode) {
    cursor.write_all(&[code as u8]).unwrap();
}



pub fn append_statements_preamble(cursor: &mut ByteCursor, len: usize, is_terminated: bool) {
    match len {
        0..=255 => {
            append_instruction_code_new(
                cursor,
                InstructionCode::SHORT_STATEMENTS,
            );
            append_u8(
                cursor,
                len as u8,
            );
        }
        _ => {
            append_instruction_code_new(
                cursor,
                InstructionCode::STATEMENTS,
            );
            append_u32(
                cursor,
                len as u32, // FIXME #673: conversion from usize to u32
            );
        }
    }

    // append termination flag
    append_u8(
        cursor,
        if is_terminated { 1 } else { 0 },
    );
}