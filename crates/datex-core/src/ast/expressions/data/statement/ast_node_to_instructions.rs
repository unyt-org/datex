use crate::{
    ast::expressions::{Statements, UnboundedStatement},
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    utils::yield_unwrap::yield_unwrap,
};

impl AstNodeToInstructions for Statements {
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
            let Statements {
                statements,
                is_terminated,
                unbounded,
            } = self;

            compilation_context.mark_has_non_static_value();
            // if single statement and not terminated, just compile the expression
            // (not for unbounded execution mode)
            if unbounded.is_none() && statements.len() == 1 && !is_terminated {
                yield_all!(statements[0].ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata
                ));
            } else {
                let is_outer_context = ctx_metadata.is_outer_context();

                // Enter a child scope for nested contexts.
                if !is_outer_context {
                    let parent_scope =
                        core::mem::take(&mut compilation_context.scope);
                    compilation_context.scope = parent_scope.push();
                }

                if let Some(UnboundedStatement { is_first, .. }) = unbounded {
                    // if this is the first section of an unbounded statements block, mark as unbounded
                    if *is_first {
                        yield Ok(
                            RegularInstruction::unbounded_statements().into()
                        );
                    }
                    // if not first, don't insert any instruction code
                }
                // otherwise, statements with fixed length
                else {
                    yield Ok(RegularInstruction::statements(
                        statements.len() as u32,
                        *is_terminated,
                    )
                    .into());
                }

                for statement in statements {
                    yield_all!(statement.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ));
                }

                // Restore the parent scope after compiling the child context.
                if !is_outer_context {
                    let child_scope =
                        core::mem::take(&mut compilation_context.scope);

                    let last_scope =
                        child_scope.pop().ok_or(CompilerError::ScopePopError);

                    compilation_context.scope = yield_unwrap!(last_scope);
                }

                // if this is the last section of an unbounded statements block, add closing instruction
                if let Some(UnboundedStatement { is_last: true, .. }) =
                    unbounded
                {
                    yield Ok(RegularInstruction::unbounded_statements_end(
                        *is_terminated,
                    )
                    .into());
                }
            }
        })
    }
}
