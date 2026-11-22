use crate::{
    ast::expressions::VariableDeclaration,
    compiler::{
        Variable, VariableModel,
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

impl AstNodeToInstructions for VariableDeclaration {
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
            let VariableDeclaration {
                id,
                name,
                kind,
                type_annotation: _,
                init_expression: value,
            } = self;

            compilation_context.mark_has_non_static_value();

            // push to stack
            yield Ok(RegularInstruction::push_to_stack().into());

            // compile expression
            yield_all!(value.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));

            let stack_index = compilation_context.scope.get_next_stack_index();

            let variable_model =
                VariableModel::infer_from_ast_metadata_and_type(
                    ast_metadata,
                    id,
                    kind,
                    compilation_context.execution_mode,
                );

            // create new variable depending on the model
            let variable = match variable_model {
                VariableModel::Constant => {
                    Variable::new_const(name.clone(), stack_index)
                }
                VariableModel::VariableSlot => Variable::new_variable_slot(
                    name.clone(),
                    *kind,
                    stack_index,
                ),
            };

            compilation_context.scope.register_variable_slot(variable);
        })
    }
}
