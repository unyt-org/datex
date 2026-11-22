use crate::{
    ast::expressions::DeriveSharedRef,
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
    shared_values::ReferenceMutability,
};

impl AstNodeToInstructions for DeriveSharedRef {
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
            match self.mutability {
                ReferenceMutability::Immutable => {
                    yield Ok(
                        RegularInstruction::derive_shared_reference().into()
                    );
                }
                ReferenceMutability::Mutable => {
                    yield Ok(RegularInstruction::derive_shared_reference_mut()
                        .into());
                }
            }

            yield_all!(self.expression.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
