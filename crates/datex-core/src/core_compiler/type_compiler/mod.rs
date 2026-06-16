use crate::{
    core_compiler::{
        core_compilation_context::{ByteCursor, CoreCompilationContext},
        to_instructions::ToInstructions,
    },
    global::protocol_structures::type_instructions::TypeInstruction,
    libs::core::core_lib_id::CoreLibIdIndex,
    prelude::*,
    shared_values::ReferenceMutability,
    types::r#type::Type,
    utils::buffers::append_u8,
};
use binrw::{BinWrite, io::Write};
pub mod type_to_instructions;

pub mod type_definition_to_instructions;
pub fn append_type_instruction(
    cursor: &mut ByteCursor,
    instruction: TypeInstruction,
) {
    // add instruction
    instruction.write(cursor).unwrap();
}

pub fn append_type(context: &mut CoreCompilationContext, ty: &Type) {
    let instructions = ty
        .to_instructions(&mut context.shared_value_tracking)
        .into_iter()
        .collect::<Vec<_>>();
    for instruction in instructions {
        append_type_instruction(&mut context.cursor, instruction);
    }
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
    use binrw::BinRead;

    use crate::{
        assert_instructions_equal,
        core_compiler::{
            core_compilation_context::ByteCursor,
            shared_value_tracking::SharedValueTracking,
            to_instructions::ToInstructions, value_compiler::compile_value,
        },
        global::protocol_structures::{
            instructions::Instruction,
            regular_instructions::RegularInstruction,
            type_instructions::TypeInstruction,
        },
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        prelude::*,
        types::{
            r#type::Type,
            type_definition::TypeDefinition,
            type_definition_with_metadata::{
                LocalMutability, LocalOwnership, TypeDefinitionWithMetadata,
                TypeMetadata,
            },
        },
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

        assert_instructions_equal!(&compiled, vec)
    }

    fn assert_regular_instructions_equal(
        val: Value,
        expected_instructions: Vec<RegularInstruction>,
    ) {
        let compiled = compile_value(val).expect("Failed to compile value");
        let mut cursor = ByteCursor::new(compiled.to_vec());
        for expected in expected_instructions {
            let instruction = RegularInstruction::read(&mut cursor).unwrap();
            assert_eq!(instruction, expected);
        }
    }

    #[test]
    fn type_definition_with_metadata() {
        let ty = Type::Alias(TypeDefinitionWithMetadata::new(
            TypeDefinition::CoreType(CoreLibTypeId::Base(
                CoreLibBaseTypeId::Boolean,
            )),
            TypeMetadata::Local {
                mutability: LocalMutability::Mutable,
                ownership: LocalOwnership::Owned,
            },
        ));
        assert_type_instructions(
            ty,
            vec![
                TypeInstruction::TypeDefinitionWithMetadata(
                    TypeMetadata::Local {
                        mutability: LocalMutability::Mutable,
                        ownership: LocalOwnership::Owned,
                    },
                ),
                TypeInstruction::TypeDefinitionCoreType(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Boolean,
                )),
            ],
        );
    }

    #[test]
    fn core_type() {
        // We shortcut the type compilation for aliased core types, that don't have any metadata or a custom type
        // to just directly return the core lib value instruction, since the execution will treat them as the same type anyway
        let ty = Type::Alias(
            TypeDefinition::CoreType(CoreLibTypeId::Base(
                CoreLibBaseTypeId::Boolean,
            ))
            .into(),
        );
        assert_regular_instructions_equal(
            Value {
                custom_type: None,
                inner: CoreValue::Type(ty),
            },
            vec![RegularInstruction::GetCoreLibValue(
                CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean).into(),
            )],
        );
    }
}
