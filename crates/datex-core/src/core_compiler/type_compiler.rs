use binrw::io::{Cursor, Write};
use binrw::{BinResult, BinWrite};
use crate::{
    core_compiler::value_compiler::append_get_shared_ref,
    global::{
        type_instruction_codes::TypeInstructionCode,
    },
    prelude::*,
    shared_values::pointer::PointerReferenceMutability,
    types::definition::TypeDefinition,
    utils::buffers::append_u8,
    values::core_values::r#type::Type,
};
use crate::core_compiler::core_compilation_context::{ByteCursor, CoreCompilationContext};
use crate::global::protocol_structures::instruction_data::{SlotAddress, TypeMetadataBin};
use crate::global::protocol_structures::type_instructions::TypeInstruction;

/// Compiles a given type container to a DXB body
pub fn compile_type(ty: &Type) -> Vec<u8> {
    let mut context = CoreCompilationContext::new(Vec::new(), SlotAddress(0));
    append_type(&mut context, ty);

    context.into_buffer()
}

pub fn append_type(context: &mut CoreCompilationContext, ty: &Type) {
    // append instruction code
    let instruction_code = TypeInstructionCode::from(&ty.type_definition);
    append_type_space_instruction_code_new(context.cursor_mut(), instruction_code);

    // append metadata
    let metadata = TypeMetadataBin::from(&ty.metadata);
    append_type_metadata(context.cursor_mut(), metadata);

    // append type definition
    append_type_definition(context, &ty.type_definition);
}

pub fn append_type_definition(
    context: &mut CoreCompilationContext,
    type_definition: &TypeDefinition,
) {
    match type_definition {
        TypeDefinition::ImplType(ty, impls) => {
            // Append the number of impls
            let impl_count = impls.len() as u8;
            append_u8(context.cursor_mut(), impl_count);

            // Append each impl address
            for impl_type in impls {
                append_get_shared_ref(
                    context,
                    impl_type,
                    &PointerReferenceMutability::Immutable,
                )
            }

            // Append the base type
            append_type(context, ty);
        }
        TypeDefinition::SharedReference(type_ref) => {
            // TODO #636: ensure pointer_address exists here
            let type_ref = type_ref.borrow();
            let pointer_address = type_ref.pointer().address();
            append_get_shared_ref(
                context,
                &pointer_address,
                &PointerReferenceMutability::Immutable,
            )
        }
        _ => todo!("#637 Type definition compilation not implemented yet"),
    };
}

#[deprecated(note = "use `append_type_instruction` instead")]
pub fn append_type_space_instruction_code(
    buffer: &mut Vec<u8>,
    code: TypeInstructionCode,
) {
    unimplemented!("use append_type_instruction instead");
}

pub fn append_type_space_instruction_code_new(
    cursor: &mut ByteCursor,
    code: TypeInstructionCode,
) {
    cursor.write_all(&[code as u8]).unwrap();
}


pub fn append_type_instruction(cursor: &mut ByteCursor, instruction: TypeInstruction) -> BinResult<()> {
    // add instruction code
    cursor.write_all(&[TypeInstructionCode::from(&instruction) as u8])?;
    // add instruction
    instruction.write(cursor)?;
    Ok(())
}


pub fn append_type_metadata(cursor: &mut ByteCursor, code: TypeMetadataBin) {
    append_u8(cursor, code.into_bytes()[0]);
}
