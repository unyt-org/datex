use crate::{
    core_compiler::{
        core_compilation_context::{ByteCursor, CoreCompilationContext},
        value_compiler::append_get_shared_ref,
    },
    global::{
        protocol_structures::{
            instruction_data::TypeMetadataBin,
            type_instructions::TypeInstruction,
        },
        type_instruction_codes::TypeInstructionCode,
    },
    prelude::*,
    shared_values::ReferenceMutability,
    types::{
        r#type::Type, type_definition::TypeDefinition,
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
    utils::buffers::append_u8,
};
use binrw::{BinWrite, io::Write};

/// Compiles a given type container to a DXB body
pub fn compile_type(ty: &Type) -> Vec<u8> {
    let mut context = CoreCompilationContext::new(Vec::new());
    append_type(&mut context, ty);

    context.into_buffer()
}

pub fn append_type(context: &mut CoreCompilationContext, ty: &Type) {
    // TODO: handle nominal type additional data via separate instruction
    // append type definition
    ty.with_collapsed_definition_with_metadata(|ty| {
        append_type_definition(context, ty);
    })
}

pub fn append_type_definition(
    context: &mut CoreCompilationContext,
    ty: &TypeDefinitionWithMetadata,
) {
    // append instruction code
    let instruction_code = TypeInstructionCode::from(&ty.definition);
    append_type_space_instruction_code_new(
        context.cursor_mut(),
        instruction_code,
    );

    // append metadata
    let metadata = TypeMetadataBin::from(&ty.metadata);
    append_type_metadata(context.cursor_mut(), metadata);

    // append structural type definition
    append_structural_type_definition(context, &ty.definition);
}

pub fn append_structural_type_definition(
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
                    &ReferenceMutability::Immutable,
                )
            }

            // Append the base type
            append_type(context, ty);
        }
        TypeDefinition::Shared(type_ref) => {
            // TODO #636: ensure pointer_address exists here
            let pointer_address = type_ref.pointer_address();
            append_get_shared_ref(
                context,
                &pointer_address,
                &ReferenceMutability::Immutable,
            )
        }
        _ => todo!("#637 Type definition compilation not implemented yet"),
    };
}

#[deprecated(note = "use `append_type_instruction` instead")]
pub fn append_type_space_instruction_code(
    _buffer: &mut Vec<u8>,
    _code: TypeInstructionCode,
) {
    unimplemented!("use append_type_instruction instead");
}

pub fn append_type_space_instruction_code_new(
    cursor: &mut ByteCursor,
    code: TypeInstructionCode,
) {
    cursor.write_all(&[code as u8]).unwrap();
}

pub fn append_type_instruction(
    cursor: &mut ByteCursor,
    instruction: TypeInstruction,
) {
    // add instruction code
    cursor
        .write_all(&[TypeInstructionCode::from(&instruction) as u8])
        .unwrap();
    // add instruction
    instruction.write(cursor).unwrap();
}

pub fn append_type_metadata(cursor: &mut ByteCursor, code: TypeMetadataBin) {
    append_u8(cursor, code.into_bytes()[0]);
}
