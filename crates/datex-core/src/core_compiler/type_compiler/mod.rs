use crate::{
    core_compiler::{
        core_compilation_context::{ByteCursor, CoreCompilationContext},
        value_compiler::append_get_shared_ref,
    },
    global::protocol_structures::{
        instruction_data::TypeMetadataBin, type_instructions::TypeInstruction,
    },
    libs::core::core_lib_id::CoreLibIdIndex,
    prelude::*,
    shared_values::ReferenceMutability,
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, impl_type::ImplTypeDefinition},
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
    utils::buffers::append_u8,
};
use binrw::{BinWrite, io::Write};
pub mod type_to_instructions;
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

// pub fn append_structural_type_definition(
//     context: &mut CoreCompilationContext,
//     type_definition: &TypeDefinition,
// ) {
//     match type_definition {
//         TypeDefinition::ImplType(ImplTypeDefinition {
//             inner_type,
//             impl_markers,
//         }) => {
//             // Append the number of impls
//             let impl_count = impl_markers.len() as u8;
//             append_u8(context.cursor_mut(), impl_count);

//             // Append each impl address
//             for impl_type in impl_markers {
//                 append_get_shared_ref(
//                     context,
//                     impl_type,
//                     &ReferenceMutability::Immutable,
//                 )
//             }

//             // Append the base type
//             append_type(context, inner_type);
//         }
//         TypeDefinition::CoreType(core_lib_id) => {
//             append_type_instruction(
//                 context.cursor_mut(),
//                 TypeInstruction::TypeDefinitionCoreType(*core_lib_id),
//             );
//         }
//         TypeDefinition::Shared(type_ref) => {
//             // TODO #636: ensure pointer_address exists here
//             let pointer_address = type_ref.pointer_address();
//             append_get_shared_ref(
//                 context,
//                 &pointer_address,
//                 &ReferenceMutability::Immutable,
//             )
//         }
//         _ => todo!("#637 Type definition compilation not implemented yet"),
//     };
// }

#[cfg(test)]
mod tests {
    use crate::{
        assert_instructions_equal,
        core_compiler::{
            type_compiler::compile_type, value_compiler::compile_value,
        },
        global::protocol_structures::{
            instructions::Instruction,
            regular_instructions::RegularInstruction,
            type_instructions::TypeInstruction,
        },
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        types::{r#type::Type, type_definition::TypeDefinition},
        values::{core_value::CoreValue, value::Value},
    };

    fn assert_type_instructions(
        ty: Type,
        expected_instruction: Vec<TypeInstruction>,
    ) {
        let vec =
            vec![Instruction::Regular(RegularInstruction::TypeExpression)]
                .into_iter()
                .chain(expected_instruction.into_iter().map(Instruction::Type))
                .collect::<Vec<_>>();

        let compiled = compile_value(Value {
            custom_type: None,
            inner: CoreValue::Type(ty),
        })
        .expect("Failed to compile type");

        println!("Compiled instructions: {:#?}", compiled);

        assert_instructions_equal!(&compiled, vec)
    }

    #[test]
    fn type_definition_core() {
        let ty = Type::Alias(
            TypeDefinition::CoreType(CoreLibTypeId::Base(
                CoreLibBaseTypeId::Boolean,
            ))
            .into(),
        );
        assert_type_instructions(
            ty,
            vec![TypeInstruction::TypeDefinitionCoreType(
                CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean),
            )],
        );
    }
}
