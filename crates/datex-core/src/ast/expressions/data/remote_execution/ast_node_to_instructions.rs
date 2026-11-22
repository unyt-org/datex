use crate::{
    ast::{expressions::RemoteExecution, to_instructions::yield_unwrap},
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        compile_child_realm_instructions, context::CompilationContext,
        error::CompilerError, metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl AstNodeToInstructions for RemoteExecution {
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
            let RemoteExecution {
                left: caller,
                right: ast,
                injected_variable_count,
            } = self;

            compilation_context.mark_has_non_static_value();

            let (instruction_block_data, new_scope) =
                yield_unwrap!(compile_child_realm_instructions(
                    compilation_context,
                    cache,
                    ast,
                    injected_variable_count.unwrap(), // must be set by precompiler
                    ast_metadata,
                    vec![],
                ));

            compilation_context.scope = new_scope;

            // insert remote execution instruction
            yield Ok(RegularInstruction::remote_execution(
                instruction_block_data,
            )
            .into());

            // insert compiled caller expression
            yield_all!(caller.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner().inner(),
            ));
        })
    }
}
