use crate::{
    ast::expressions::Conditional,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    instruction::{
        BytecodeItem, instruction_data::UnboundedStatementsData,
        regular_instruction::RegularInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl AstNodeToInstructions for Conditional {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        cache: &'a mut SharedReferencesCache,
        ast_metadata: &'a mut AstMetadata,
        ctx_metadata: CompileContextMetadata,
    ) -> Box<dyn Iterator<Item = Result<BytecodeItem, CompilerError>> + 'a>
    {
        Box::new(gen move {
            let Conditional {
                condition,
                then_branch,
                else_branch,
            } = self;

            compilation_context.mark_has_non_static_value();
            let input = compilation_context.core_context.input.clone();

            let condition_instructions = {
                let mut inner_ctx = CompilationContext::new(
                    compilation_context.inserted_values.clone(),
                    compilation_context.execution_mode,
                    input.clone(),
                    compilation_context.scope.clone(),
                );

                condition
                    .ast_node_to_instructions(
                        &mut inner_ctx,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner(),
                    )
                    .collect::<Vec<_>>()
            };

            let then_instructions = {
                let mut inner_ctx = CompilationContext::new(
                    compilation_context.inserted_values.clone(),
                    compilation_context.execution_mode,
                    input.clone(),
                    compilation_context.scope.clone(),
                );

                then_branch
                    .ast_node_to_instructions(
                        &mut inner_ctx,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner(),
                    )
                    .collect::<Vec<_>>()
            };

            let else_instructions = match else_branch {
                Some(else_expr) => {
                    let mut inner_ctx = CompilationContext::new(
                        compilation_context.inserted_values.clone(),
                        compilation_context.execution_mode,
                        input,
                        compilation_context.scope.clone(),
                    );

                    else_expr
                        .ast_node_to_instructions(
                            &mut inner_ctx,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner(),
                        )
                        .collect::<Vec<_>>()
                }
                None => Vec::new(),
            };

            let else_label = ctx_metadata.next_label();
            let condition_end = ctx_metadata.next_label();
            let end_label = ctx_metadata.next_label();

            yield Ok(RegularInstruction::UnboundedStatements.into());

            yield Ok(BytecodeItem::JumpIfFalse {
                target: else_label,
                condition_end,
            });

            yield_all!(condition_instructions);
            yield Ok(BytecodeItem::Label(condition_end));

            yield_all!(then_instructions);

            if !else_instructions.is_empty() {
                yield Ok(BytecodeItem::Jump { target: end_label });
                yield Ok(BytecodeItem::Label(else_label));
                yield_all!(else_instructions);

                yield Ok(BytecodeItem::Label(end_label));
            } else {
                yield Ok(BytecodeItem::Label(else_label));
            }

            yield Ok(RegularInstruction::UnboundedStatementsEnd(
                UnboundedStatementsData { terminated: false },
            )
            .into());
        })
    }
}
