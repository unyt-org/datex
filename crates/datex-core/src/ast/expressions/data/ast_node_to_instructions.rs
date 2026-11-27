use crate::{
    ast::expressions::NormalExpressionData,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    core_compiler::to_instructions::{ToInstructions, yield_all, yield_all_ok},
    instruction::{BytecodeItem, regular_instruction::RegularInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl AstNodeToInstructions for NormalExpressionData {
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
            match self {
                NormalExpressionData::Integer(integer) => {
                    yield Ok(BytecodeItem::from(RegularInstruction::Integer(
                        integer.clone(),
                    )))
                }
                NormalExpressionData::TypedInteger(typed_integer) => {
                    yield_all_ok!(
                        typed_integer
                            .to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Instant(instant) => {
                    yield_all_ok!(
                        instant.to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Decimal(decimal) => {
                    yield_all_ok!(
                        decimal.to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::TypedDecimal(typed_decimal) => {
                    yield_all_ok!(
                        typed_decimal
                            .to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Text(text) => {
                    yield_all_ok!(
                        text.to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Boolean(boolean) => {
                    yield_all_ok!(
                        boolean.to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Endpoint(endpoint) => {
                    yield_all_ok!(
                        endpoint.to_instructions(compilation_context, cache)
                    );
                }

                NormalExpressionData::Null => {
                    yield Ok(BytecodeItem::from(RegularInstruction::null()));
                }

                NormalExpressionData::List(list) => {
                    yield_all!(list.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Map(map) => {
                    yield_all!(map.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // unary operations
                NormalExpressionData::UnaryOperation(unary_operation) => {
                    yield_all!(unary_operation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // binary operations
                NormalExpressionData::BinaryOperation(binary_operation) => {
                    yield_all!(binary_operation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // comparisons
                NormalExpressionData::ComparisonOperation(
                    comparison_operation,
                ) => {
                    yield_all!(comparison_operation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // apply
                NormalExpressionData::Apply(apply) => {
                    yield_all!(apply.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // interface methods
                NormalExpressionData::InterfaceMethodCall(
                    interface_method_call,
                ) => {
                    yield_all!(interface_method_call.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // property access
                NormalExpressionData::PropertyAccess(property_access) => {
                    yield_all!(property_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::GenericInstantiation(
                    generic_instantiation,
                ) => {
                    yield_all!(generic_instantiation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::PropertyAssignment(property_assignment) => {
                    yield_all!(property_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::RequestSharedRef(request_shared_ref) => {
                    yield_all!(request_shared_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::UnboxAssignment(unbox_assignment) => {
                    yield_all!(unbox_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // root property
                NormalExpressionData::RootPropertyAccess(
                    root_property_access,
                ) => {
                    yield_all!(root_property_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // refs
                NormalExpressionData::DeriveRef(derive_ref) => {
                    yield_all!(derive_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // shared refs
                NormalExpressionData::DeriveSharedRef(derive_shared_ref) => {
                    yield_all!(derive_shared_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // shared values
                NormalExpressionData::CreateShared(create_shared) => {
                    yield_all!(create_shared.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::TypeExpression(type_expression) => {
                    yield Ok(RegularInstruction::TypeExpression.into());
                    yield_all!(type_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Range(range_dec) => {
                    yield_all!(range_dec.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Unbox(unbox) => {
                    yield_all!(unbox.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Tag(tag_expression) => {
                    yield_all!(tag_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::ResolveCoreLibId(core_lib_id) => {
                    yield Ok(RegularInstruction::get_core_lib_value(
                        (*core_lib_id).into(),
                    )
                    .into());
                }

                NormalExpressionData::CallableDeclaration(
                    callable_declaration,
                ) => {
                    yield_all!(callable_declaration.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Conditional(conditional) => {
                    yield_all!(conditional.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::RemoteExecution(remote_execution) => {
                    yield_all!(remote_execution.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::VariableDeclaration(
                    _variable_declaration,
                ) => {
                    yield_all!(_variable_declaration.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::Statements(statements) => {
                    yield_all!(statements.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::VariableAssignment(variable_assignment) => {
                    yield_all!(variable_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::VariableAccess(variable_access) => {
                    yield_all!(variable_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                NormalExpressionData::Placeholder(value_access_type) => {
                    yield_all!(value_access_type.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::Noop => todo!(),
                NormalExpressionData::NativeImplementationIndicator => todo!(),
                NormalExpressionData::OmitRecursive => todo!(),
                NormalExpressionData::Identifier(_) => todo!(),
                NormalExpressionData::Sender => todo!(),
                NormalExpressionData::Receiver => todo!(),
                NormalExpressionData::MoveSharedValue(
                    _self_owned_pointer_address,
                ) => todo!(),
                NormalExpressionData::Compile(compile_expression) => {
                    yield_all!(compile_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::TypeDeclaration(
                    type_declaration_expression,
                ) => {
                    yield_all!(
                        type_declaration_expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata
                        )
                    );
                }
                NormalExpressionData::EntityDeclaration(
                    entity_declaration_expression,
                ) => {
                    yield_all!(
                        entity_declaration_expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata
                        )
                    );
                }
                NormalExpressionData::CreateMut(create_mut) => {
                    yield_all!(create_mut.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::Clone(clone_expression) => {
                    yield_all!(clone_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::StackIndex(_stack_index) => todo!(),
                NormalExpressionData::StackAssignment(_stack_assignment) => {
                    todo!()
                }
                NormalExpressionData::StackListAssignment(
                    stack_list_assignment,
                ) => {
                    yield_all!(stack_list_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::UnboxSlotAssignment(
                    unbox_slot_assignment,
                ) => {
                    yield_all!(unbox_slot_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::VariantAccess(variant_access) => {
                    yield_all!(variant_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                NormalExpressionData::EntityValue(entity_value_expression) => {
                    yield_all!(
                        entity_value_expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata
                        )
                    );
                }

                NormalExpressionData::Recover => unreachable!(),
            }
        })
    }
}
