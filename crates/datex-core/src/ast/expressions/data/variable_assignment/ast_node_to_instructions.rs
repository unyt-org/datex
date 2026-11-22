use crate::{
    ast::{
        expressions::{VariableAssignment, VariableKind},
        to_instructions::yield_unwrap,
    },
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    global::operators::ModificationOperator,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};
use log::info;

impl AstNodeToInstructions for VariableAssignment {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        cache: &'a mut SharedReferencesCache,
        ast_metadata: &'a mut AstMetadata,
        ctx_metadata: CompileContextMetadata,
    ) -> Box<
        dyn Iterator<
                Item = Result<crate::instruction::BytecodeItem, CompilerError>,
            > + 'a,
    > {
        Box::new(gen move {
            let VariableAssignment {
                operator,
                name,
                expression,
                ..
            } = self;

            compilation_context.mark_has_non_static_value();
            // get variable slot address
            let (stack_index, kind) = yield_unwrap!(
                yield_unwrap!(
                    compilation_context
                        .scope
                        .resolve_variable_name(name, None)
                        .map_err(|_| {
                            CompilerError::AssignmentToExternalVariable(
                                name.clone(),
                            )
                        })
                )
                .ok_or_else(|| {
                    CompilerError::UndeclaredVariable(name.clone())
                })
            );

            // TODO #484: check not needed, is already handled in precompiler - can we guarantee this?
            // if const, return error
            if kind == VariableKind::Const {
                return yield Err(CompilerError::AssignmentToConst(
                    name.clone(),
                ));
            }

            match operator {
                None => {
                    // append binary code to load variable
                    info!(
                        "append variable - stack index: {stack_index:?}, name: {name}"
                    );
                    yield Ok(
                        RegularInstruction::set_stack_value(stack_index).into()
                    );
                    let inner = ctx_metadata.inner();
                    // compile expression
                    yield_all!(expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        inner,
                    ));
                }
                Some(operator) => {
                    // TODO #435: handle mut type
                    // // if immutable reference, return error
                    // if mut_type == Some(ReferenceMutability::Immutable) {
                    //     return Err(
                    //         CompilerError::AssignmentToImmutableReference(
                    //             name.clone(),
                    //         ),
                    //     );
                    // }
                    // // if immutable value, return error
                    // else if mut_type == None {
                    //     return Err(CompilerError::AssignmentToImmutableValue(
                    //         name.clone(),
                    //     ));
                    // }

                    let direct_assignment_operator = match operator {
                        ModificationOperator::AddAssign => {
                            Some(RegularInstruction::increment())
                        }
                        ModificationOperator::SubtractAssign => {
                            Some(RegularInstruction::decrement())
                        }
                        _operator => None,
                    };

                    if let Some(instruction) = direct_assignment_operator {
                        yield Ok(instruction.into());

                        // compile expression
                        yield_all!(expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata
                        ));

                        yield Ok(RegularInstruction::borrow_stack_value(
                            stack_index,
                        )
                        .into());
                    } else {
                        // Generate x = x * z instructions;
                        todo!()
                    }
                }
            }
        })
    }
}
