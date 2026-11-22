use crate::{
    ast::expressions::DeriveRef,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl AstNodeToInstructions for DeriveRef {
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
        // TODO #764: handle lifetimes, mutability, correctly (in precompiler)
        // TODO #765: handle move/clone
        Box::new(gen move {
            compilation_context.mark_has_non_static_value();

            yield_all!(self.expression.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
