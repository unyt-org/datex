use crate::{
    ast::expressions::Apply,
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
};

impl AstNodeToInstructions for Apply {
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
            compilation_context.mark_has_non_static_value();

            // more than u8 size args -> error
            let len = self.arguments.len();
            if len > u8::MAX as usize {
                return yield Err(CompilerError::TooManyApplyArguments);
            }

            yield Ok(RegularInstruction::apply(len as u8).into());
            // compile arguments
            for argument in &self.arguments {
                yield_all!(argument.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata.inner()
                ));
            }
            // compile function expression
            yield_all!(self.base.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
