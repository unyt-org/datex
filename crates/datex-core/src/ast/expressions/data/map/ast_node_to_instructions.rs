use crate::{
    ast::expressions::{NormalExpressionData, Map},
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

impl AstNodeToInstructions for Map {
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
            let has_only_text_keys = self.entries.iter().all(|(key, _)| {
                matches!(&*key.data, NormalExpressionData::Text(_))
            });

            yield Ok(RegularInstruction::map(
                self.entries.len() as u32,
                has_only_text_keys,
            )
            .into());
            for (key, value) in &self.entries {
                match &*key.data {
                    // text -> insert key string
                    NormalExpressionData::Text(text) => {
                        if text.len() < 256 {
                            yield Ok(RegularInstruction::key_value_short_text(
                                text.as_string(),
                            )
                            .into());
                        } else {
                            yield Ok(
                                RegularInstruction::key_value_dynamic().into()
                            );
                            yield Ok(RegularInstruction::text(
                                text.as_string(),
                            )
                            .into());
                        }
                    }
                    // other -> insert key as dynamic
                    _ => {
                        yield Ok(RegularInstruction::key_value_dynamic().into());
                        yield_all!(key.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner()
                        ));
                    }
                };

                // value
                yield_all!(value.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata.inner()
                ));
            }
        })
    }
}
