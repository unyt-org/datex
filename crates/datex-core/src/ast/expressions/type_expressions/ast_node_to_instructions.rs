use crate::{
    ast::expressions::{
        TypeExpression, TypeExpressionData, expression::Expression,
    },
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::yield_all,
    instruction::{BytecodeItem, type_instruction::TypeInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::literal_type_definition::LiteralTypeDefinition,
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
        Box::new(gen move {
            match self.data() {
                TypeExpressionData::TypedInteger(typed_integer) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::TypedInteger(
                            typed_integer.clone(),
                        ),
                    )
                    .into())
                }
                TypeExpressionData::Decimal(decimal) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::Decimal(decimal.clone()),
                    )
                    .into())
                }
                TypeExpressionData::TypedDecimal(typed_decimal) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::TypedDecimal(
                            typed_decimal.clone(),
                        ),
                    )
                    .into())
                }
                TypeExpressionData::Endpoint(endpoint) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::Endpoint(endpoint.clone()),
                    )
                    .into())
                }
                TypeExpressionData::Integer(integer) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::Integer(integer.clone()),
                    )
                    .into())
                }
                TypeExpressionData::Text(text) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::Text(text.clone()),
                    )
                    .into())
                }
                TypeExpressionData::Boolean(boolean) => {
                    yield Ok(TypeInstruction::Literal(
                        LiteralTypeDefinition::Boolean(boolean.clone()),
                    )
                    .into())
                }
                TypeExpressionData::GetCoreLibType(core_lib_id) => {
                    yield Ok(TypeInstruction::CoreType(*core_lib_id).into())
                }
                TypeExpressionData::Range(range) => {
                    yield_all!(range.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ));
                }
                TypeExpressionData::IdentifierWithPointerAddress(
                    identifier_with_pointer_address,
                ) => yield_all!(
                    identifier_with_pointer_address.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    )
                ),
                TypeExpressionData::VariableAccess(variable_access) => {
                    yield_all!(variable_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }

                TypeExpressionData::StructuralList(structural_list) => {
                    yield_all!(structural_list.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::FixedSizeList(fixed_size_list) => {
                    yield_all!(fixed_size_list.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::SliceList(slice_list) => {
                    yield_all!(slice_list.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Intersection(intersection) => {
                    yield_all!(intersection.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Union(union) => {
                    yield_all!(union.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::GenericAccess(generic_access) => {
                    yield_all!(generic_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Callable(callable_type_expression) => {
                    yield_all!(
                        callable_type_expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata.inner()
                        )
                    )
                }
                TypeExpressionData::StructuralMap(structural_map) => {
                    yield_all!(structural_map.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Ref(type_expression) => {
                    yield_all!((**type_expression).ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::RefMut(type_expression) => {
                    yield_all!((**type_expression).ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Shared(type_expression) => {
                    yield_all!((**type_expression).ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::Mut(type_expression) => {
                    yield_all!((**type_expression).ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::VariantAccess(type_variant_access) => {
                    yield_all!(type_variant_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata.inner()
                    ))
                }
                TypeExpressionData::GetReference(_pointer_address) => todo!(),

                TypeExpressionData::Identifier(_)
                | TypeExpressionData::Recover => unreachable!(),
            }
        })
    }
}
