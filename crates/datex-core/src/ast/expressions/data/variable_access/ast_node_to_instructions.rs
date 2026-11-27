use crate::{
    ast::expressions::{ValueAccessType, VariableAccess},
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    global::protocol_structures::injected_values::{
        InjectedValueType, LocalInjectedValueType, SharedInjectedValueType,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    utils::yield_unwrap::yield_unwrap,
};

impl AstNodeToInstructions for VariableAccess {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        _cache: &'a mut SharedReferencesCache,
        _ast_metadata: &'a mut AstMetadata,
        _ctx_metadata: CompileContextMetadata,
    ) -> Box<
        dyn Iterator<
                Item = Result<crate::instruction::BytecodeItem, CompilerError>,
            > + 'a,
    > {
        Box::new(gen move {
            let VariableAccess {
                name, access_type, ..
            } = self;

            compilation_context.mark_has_non_static_value();

            let slot_type = match access_type {
                ValueAccessType::SharedRefMut => {
                    InjectedValueType::Shared(SharedInjectedValueType::RefMut)
                }
                ValueAccessType::SharedRef => {
                    InjectedValueType::Shared(SharedInjectedValueType::Ref)
                }
                // TODO: map to local slot types depending on type
                ValueAccessType::MoveOrCopy => {
                    InjectedValueType::Shared(SharedInjectedValueType::Move)
                }
                // TODO:
                ValueAccessType::Clone => {
                    InjectedValueType::Local(LocalInjectedValueType::Move)
                }
                ValueAccessType::Borrow => {
                    InjectedValueType::Shared(SharedInjectedValueType::Move)
                }
            };

            // get variable slot address
            let (stack_index, ..) = yield_unwrap!(
                compilation_context
                    .scope
                    .resolve_variable_name_with_slot_type(name, slot_type)
                    .ok_or_else(|| {
                        CompilerError::UndeclaredVariable(name.clone())
                    })
            );

            // append binary code to load variable
            let slot_access_instruction = match access_type {
                ValueAccessType::SharedRefMut => {
                    RegularInstruction::get_stack_value_shared_ref_mut(
                        stack_index,
                    )
                }
                ValueAccessType::SharedRef => {
                    RegularInstruction::get_stack_value_shared_ref(stack_index)
                }
                ValueAccessType::MoveOrCopy => {
                    RegularInstruction::take_stack_value(stack_index)
                }
                ValueAccessType::Clone => {
                    RegularInstruction::clone_stack_value(stack_index)
                }
                ValueAccessType::Borrow => {
                    RegularInstruction::borrow_stack_value(stack_index)
                }
            };

            yield Ok(slot_access_instruction.into());
        })
    }
}
