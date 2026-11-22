use crate::{
    ast::expressions::{
        DataExpressionData, PropertyAccess, expression::Expression,
    },
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

impl AstNodeToInstructions for PropertyAccess {
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

            // depending on the key, handle different property accesses
            match self.property.data() {
                // simple text key if length fits in u8
                DataExpressionData::Text(key) if key.len() <= 255 => {
                    yield Ok(RegularInstruction::get_entry_text(
                        key.as_string(),
                    )
                    .into());
                }
                // index access if integer fits in u32
                DataExpressionData::Integer(index)
                    if let Some(index) = index.as_u32() =>
                {
                    yield Ok(RegularInstruction::get_entry_index(index).into());
                }
                _ => {
                    yield Ok(RegularInstruction::get_entry_dynamic().into());
                    yield_all!(self.property.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ));
                }
            }

            // compile base expression
            yield_all!(self.base.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
