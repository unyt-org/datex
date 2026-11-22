use crate::{
    ast::expressions::RangeTypeExpr,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    instruction::{BytecodeItem, type_instruction::TypeInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};
use core::ops::Deref;

impl AstNodeToInstructions for RangeTypeExpr {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        cache: &'a mut SharedReferencesCache,
        ast_metadata: &'a mut AstMetadata,
        ctx_metadata: CompileContextMetadata,
    ) -> Box<dyn Iterator<Item = Result<BytecodeItem, CompilerError>> + 'a>
    {
        Box::new(gen move {
            yield Ok(TypeInstruction::Range.into());
            yield_all!(self.start.deref().ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
            yield_all!(self.end.deref().ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
