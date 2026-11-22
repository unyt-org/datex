use crate::{
    ast::expressions::EntityDeclarationExpression,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};
impl AstNodeToInstructions for EntityDeclarationExpression {
    fn ast_node_to_instructions<'a>(
        &'a self,
        _compilation_context: &'a mut CompilationContext,
        _cache: &'a mut SharedReferencesCache,
        _ast_metadata: &'a mut AstMetadata,
        _ctx_metadata: CompileContextMetadata,
    ) -> Box<
        dyn Iterator<
                Item = Result<crate::instruction::BytecodeItem, CompilerError>,
            > + 'a,
    > {
        todo!()
    }
}
