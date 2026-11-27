use crate::{
    ast::expressions::InterfaceMethodCall,
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

impl AstNodeToInstructions for InterfaceMethodCall {
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

            // TODO: replace with trait impls
            match self.method_name.as_str() {
                "append" => {
                    // must be exactly one element
                    if self.arguments.len() != 1 {
                        return yield Err(
                            CompilerError::InvalidInterfaceMethodCall(
                                self.method_name.to_string(),
                            ),
                        );
                    }

                    yield Ok(RegularInstruction::append_entry().into());

                    for argument in &self.arguments {
                        yield_all!(argument.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner()
                        ));
                    }
                }

                "clear" => {
                    // no arguments allowed
                    if !self.arguments.is_empty() {
                        return yield Err(
                            CompilerError::InvalidInterfaceMethodCall(
                                self.method_name.to_string(),
                            ),
                        );
                    }

                    yield Ok(RegularInstruction::clear().into());
                }

                "splice" => {
                    // must be exactly three elements
                    if self.arguments.len() != 3 {
                        return yield Err(
                            CompilerError::InvalidInterfaceMethodCall(
                                self.method_name.to_string(),
                            ),
                        );
                    }

                    yield Ok(RegularInstruction::splice_dynamic().into());

                    for argument in &self.arguments {
                        yield_all!(argument.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner()
                        ));
                    }
                }

                _ => {
                    // TODO: check if method actually exists

                    yield Ok(RegularInstruction::call_method(
                        self.method_name.clone(),
                        self.arguments.len() as u8,
                    )
                    .into());

                    for argument in &self.arguments {
                        yield_all!(argument.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner()
                        ));
                    }
                }
            }

            // compile target expression
            yield_all!(self.target.ast_node_to_instructions(
                compilation_context,
                cache,
                ast_metadata,
                ctx_metadata.inner()
            ));
        })
    }
}
