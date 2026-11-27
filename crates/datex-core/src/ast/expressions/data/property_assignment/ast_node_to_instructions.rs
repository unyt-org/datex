use crate::{
    ast::expressions::{
        NormalExpressionData, PropertyAssignment, expression::Expression,
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

impl AstNodeToInstructions for PropertyAssignment {
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

            let PropertyAssignment {
                base,
                property,
                assigned_expression,
                ..
            } = self;
            // depending on the key, handle different property assignments

            match &property.data() {
                NormalExpressionData::Text(key)
                    if key.len() <= u8::MAX as usize =>
                {
                    yield Ok(RegularInstruction::set_entry_text(
                        key.as_string(),
                    )
                    .into());
                }

                NormalExpressionData::Integer(index)
                    if let Some(index) = index.as_u32() =>
                {
                    yield Ok(RegularInstruction::set_entry_index(index).into());
                }

                _ => {
                    yield Ok(RegularInstruction::set_entry_dynamic().into());
                    yield_all!(property.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ));
                }
            }
            // compile assigned expression
            yield_all!(assigned_expression.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));

            // compile base expression
            yield_all!(base.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
