use crate::{
    ast::expressions::{TypeExpression, expression::Expression},
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    instruction::BytecodeItem,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};
impl AstNodeToInstructions for TypeExpression {
    fn ast_node_to_instructions<'a>(
        &'a self,
        compilation_context: &'a mut CompilationContext,
        cache: &'a mut SharedReferencesCache,
        ast_metadata: &'a mut AstMetadata,
        ctx_metadata: CompileContextMetadata,
    ) -> Box<dyn Iterator<Item = Result<BytecodeItem, CompilerError>> + 'a>
    {
        self.data().ast_node_to_instructions(
            compilation_context,
            cache,
            ast_metadata,
            ctx_metadata,
        )
    }
}
