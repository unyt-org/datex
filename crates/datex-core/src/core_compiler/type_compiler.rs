use crate::{
    core_compiler::value_compiler::append_get_ref,
    global::type_instruction_codes::{TypeInstructionCode, TypeReferenceMutabilityCode},
    types::definition::TypeDefinition,
    utils::buffers::append_u8,
    values::core_values::r#type::Type,
};
use crate::global::protocol_structures::instructions::TypeMetadataBin;
use crate::prelude::*;
/// Compiles a given type container to a DXB body
pub fn compile_type(ty: &Type) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(256);
    append_type(&mut buffer, ty);

    buffer
}

pub fn append_type(buffer: &mut Vec<u8>, ty: &Type) {
    // append instruction code
    let instruction_code = TypeInstructionCode::from(&ty.type_definition);
    append_type_space_instruction_code(buffer, instruction_code);

    // append metadata
    let metadata = TypeMetadataBin::from(&ty.metadata);
    append_type_metadata(buffer, metadata);

    // append type definition
    append_type_definition(buffer, &ty.type_definition);
}

fn append_type_definition(
    buffer: &mut Vec<u8>,
    type_definition: &TypeDefinition,
) {
    match type_definition {
        TypeDefinition::ImplType(ty, impls) => {
            // Append the number of impls
            let impl_count = impls.len() as u8;
            append_u8(buffer, impl_count);

            // Append each impl address
            for impl_type in impls {
                append_get_ref(buffer, impl_type);
            }

            // Append the base type
            append_type(buffer, ty);
        }
        TypeDefinition::SharedReference(type_ref) => {
            // TODO #636: ensure pointer_address exists here
            let type_ref = type_ref.borrow();
            let pointer_address = type_ref
                .pointer
                .address();
            append_get_ref(buffer, pointer_address.as_ref());
        }
        _ => todo!("#637 Type definition compilation not implemented yet"),
    };
}

pub fn append_type_space_instruction_code(
    buffer: &mut Vec<u8>,
    code: TypeInstructionCode,
) {
    append_u8(buffer, code as u8);
}

pub fn append_type_metadata(
    buffer: &mut Vec<u8>,
    code: TypeMetadataBin,
) {
    append_u8(buffer, code.into_bytes()[0]);
}
