use crate::{
    ast::expressions::CallableDeclaration,
    compiler::{
        Variable, ast_node_to_instructions::AstNodeToInstructions,
        compile_child_realm_instructions, context::CompilationContext,
        error::CompilerError, metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    global::stack_index::StackIndex,
    instruction::{
        instruction_data::{
            CallableDeclarationData, CallableSignatureData, ShortTextData,
        },
        regular_instruction::RegularInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    utils::yield_unwrap::yield_unwrap,
};

impl AstNodeToInstructions for CallableDeclaration {
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
            let CallableDeclaration {
                signature,
                body,
                injected_variable_count,
            } = self;

            compilation_context.mark_has_non_static_value();

            // generate variables for parameters
            let mut variables = vec![];
            let mut index = StackIndex(0);
            for (name, _) in &signature.parameters {
                variables.push(
                    Variable::new_const(name.clone(), index), // TODO: allow var
                );
                index += 1;
            }
            if let Some((name, _)) = &signature.rest_parameter {
                variables.push(
                    Variable::new_const(name.clone(), index), // TODO: allow var
                );
            }

            let res = compile_child_realm_instructions(
                compilation_context,
                cache,
                body,
                injected_variable_count.unwrap(), // must be set by precompiler
                ast_metadata,
                variables,
            );

            let (instruction_block_data, new_scope) = yield_unwrap!(res);

            compilation_context.scope = new_scope;

            yield Ok(RegularInstruction::CallableDeclaration(
                CallableDeclarationData {
                    signature: CallableSignatureData {
                        name: ShortTextData(
                            signature.name.clone().unwrap_or_default(),
                        ),
                        kind: signature.kind,
                        requires_async: signature.requires_async,
                        parameter_count: signature.parameters.len() as u8,
                        has_rest_parameter: signature.rest_parameter.is_some(),
                        has_return_type: signature.return_type.is_some(),
                        has_yeet_type: signature.yeet_type.is_some(),
                        parameter_names: signature
                            .parameters
                            .iter()
                            .map(|(name, _)| ShortTextData(name.clone()))
                            .collect(),
                        rest_parameter_name: signature
                            .rest_parameter
                            .clone()
                            .map(|(name, _)| ShortTextData(name)),
                    },
                    body: instruction_block_data,
                },
            )
            .into());

            // add parameter types
            for (_, param) in &signature.parameters {
                yield_all!(param.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata.inner()
                ));
            }
            // add rest parameter type
            if let Some((_, param)) = &signature.rest_parameter {
                yield_all!(param.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata.inner()
                ));
            }
            // add return type
            if let Some(ty) = &signature.return_type {
                yield_all!(ty.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata.inner()
                ));
            }
            // add yield type
            if let Some(ty) = &signature.yeet_type {
                yield_all!(ty.ast_node_to_instructions(
                    compilation_context,
                    cache,
                    ast_metadata,
                    ctx_metadata
                ));
            }
        })
    }
}
