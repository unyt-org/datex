use crate::{
    core_compiler::core_compilation_context::ByteCursor,
    global::protocol_structures::type_instructions::TypeInstruction,
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

#[cfg(test)]
#[cfg(feature = "disassembler")]
mod tests {
    use binrw::BinRead;

    use crate::{
        assert_instructions_equal,
        core_compiler::{
            core_compilation_context::{
                ByteCursor, default_compile_input,
                default_core_compilation_context,
            },
            shared_value_tracking::SharedValueTracking,
            to_instructions::ToInstructions,
            value_compiler::compile_value,
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
        let compile_input = unsafe { default_compile_input() };
        let vec =
            vec![Instruction::Regular(RegularInstruction::TypeExpression)]
                .into_iter()
                .chain(expected_instruction.into_iter().map(Instruction::Type))
                .collect::<Vec<_>>();

        let compiled = compile_value(
            Value {
                custom_type: None,
                inner: CoreValue::Type(ty),
            },
            compile_input,
        );
        assert_eq!(compiled.shared_values.len(), 0);
        assert_instructions_equal!(&compiled.dxb, vec)
    }

    fn assert_regular_instructions_equal(
        val: Value,
        expected_instructions: Vec<RegularInstruction>,
    ) {
        let compile_input = unsafe { default_compile_input() };
        let compiled = compile_value(val, compile_input);
        let mut cursor = ByteCursor::new(compiled.dxb.to_vec());
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
