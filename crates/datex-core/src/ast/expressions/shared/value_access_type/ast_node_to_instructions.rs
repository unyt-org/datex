use crate::{
    ast::expressions::ValueAccessType,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::{ToInstructions, yield_all_ok_collected},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    utils::yield_unwrap::yield_unwrap,
    values::value_container::ValueContainer,
};

/// implementation used for [NormalExpressionData::Placeholder]
impl AstNodeToInstructions for ValueAccessType {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        cache: &'a mut SharedReferencesCache,
        _ast_metadata: &'a mut AstMetadata,
        _ctx_metadata: CompileContextMetadata,
    ) -> Box<
        dyn Iterator<
                Item = Result<crate::instruction::BytecodeItem, CompilerError>,
            > + 'a,
    > {
        Box::new(gen move {
            // FIXME #720
            let placeholder = compilation_context
                .inserted_values
                .get_mut(compilation_context.inserted_value_index)
                .expect("Placeholder index out of bounds");
            if let Some(value_container) = placeholder.take() {
                // TODO: validate in precompiler that the value container is actually a shared value

                match value_container {
                    ValueContainer::Local(value) => {
                        match self {
                            ValueAccessType::SharedRef
                            | ValueAccessType::SharedRefMut => {
                                return yield Err(
                                    CompilerError::SharedRefToNonSharedValue,
                                );
                            }
                            ValueAccessType::MoveOrCopy => {
                                yield_all_ok_collected!(value.to_instructions(
                                    compilation_context,
                                    cache
                                ));
                            }
                            ValueAccessType::Clone => {
                                yield_all_ok_collected!(value.to_instructions(
                                    compilation_context,
                                    cache
                                ));
                            }
                            ValueAccessType::Borrow => {
                                yield_all_ok_collected!(value.to_instructions(
                                    compilation_context,
                                    cache
                                ));
                            }
                        }
                    }
                    ValueContainer::Shared(shared_container) => {
                        match self {
                            ValueAccessType::SharedRefMut => {
                                let shared_container_mut_ref = yield_unwrap!(
                                    shared_container
                                    .try_derive_mutable_reference()
                                    .map_err(|_| CompilerError::SharedMutRefToImmutableValue)
                                );
                                yield_all_ok_collected!(shared_container_mut_ref.to_instructions(compilation_context, cache));
                            }
                            ValueAccessType::SharedRef => {
                                let shared_container_immut_ref = shared_container.derive_immutable_reference();
                                yield_all_ok_collected!(shared_container_immut_ref.to_instructions(compilation_context, cache));
                            },
                            ValueAccessType::MoveOrCopy => {
                                match shared_container {
                                    SharedContainer::Owned(shared_container) => {
                                        yield_all_ok_collected!(shared_container.to_instructions(compilation_context, cache));
                                    }
                                    _ => return yield Err(CompilerError::InvalidConversionFromRefToOwnedValue),
                                }
                            },
                            ValueAccessType::Clone => {
                                let cloned = shared_container.inner().base_shared_container().value_container().clone();
                                match cloned {
                                    ValueContainer::Local(value) => {
                                        yield_all_ok_collected!(value.to_instructions(compilation_context, cache));
                                    }
                                    ValueContainer::Shared(shared_container) => {
                                        yield_all_ok_collected!(shared_container.to_instructions(compilation_context, cache));
                                    }
                                }
                            },
                            ValueAccessType::Borrow => {
                                let reference = shared_container.derive_reference_with_max_mutability();
                                yield_all_ok_collected!(reference.to_instructions(compilation_context, cache));
                            }
                        };
                    }
                }
            } else {
                // TODO
                // compilation_context
                //     .append_instruction_code(InstructionCode::CLONE_STACK_VALUE);
                // compilation_context.insert_virtual_slot_address(
                //     InjectedParentVariable::local(
                //         compilation_context.inserted_value_index as u32,
                //     ),
                // );
            }
            compilation_context.inserted_value_index += 1;
        })
    }
}
