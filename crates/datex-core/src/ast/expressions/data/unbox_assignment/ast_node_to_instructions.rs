use crate::{
    ast::expressions::UnboxAssignment,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    global::operators::ModificationOperator,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl AstNodeToInstructions for UnboxAssignment {
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

            match self.operator {
                Some(operator) => match operator {
                    ModificationOperator::AddAssign => {
                        yield Ok(RegularInstruction::increment().into())
                    }
                    ModificationOperator::SubtractAssign => {
                        yield Ok(RegularInstruction::decrement().into())
                    }
                    _ => todo!("Generate x = x * z instructions;"),
                },
                None => {
                    yield Ok(
                        RegularInstruction::set_shared_container_value().into()
                    )
                }
            };

            // compile assigned expression
            yield_all!(self.assigned_expression.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));

            // compile unbox expression
            yield_all!(self.unbox_expression.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
