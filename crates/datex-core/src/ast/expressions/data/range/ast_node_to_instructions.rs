use crate::{
    ast::expressions::RangeDeclaration,
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

impl AstNodeToInstructions for RangeDeclaration {
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
            yield Ok(RegularInstruction::range().into());
            yield_all!(self.start.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
            yield_all!(self.end.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
