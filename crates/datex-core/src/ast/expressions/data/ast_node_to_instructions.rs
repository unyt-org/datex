use crate::{
    ast::expressions::DataExpressionData,
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

impl AstNodeToInstructions for DataExpressionData {
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
                DataExpressionData::Integer(integer) => {
                    yield Ok(BytecodeItem::from(RegularInstruction::Integer(
                        integer.clone(),
                    )))
                }
                DataExpressionData::TypedInteger(typed_integer) => {
                    yield_all_ok!(
                        typed_integer
                            .to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Instant(instant) => {
                    yield_all_ok!(
                        instant.to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Decimal(decimal) => {
                    yield_all_ok!(
                        decimal.to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::TypedDecimal(typed_decimal) => {
                    yield_all_ok!(
                        typed_decimal
                            .to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Text(text) => {
                    yield_all_ok!(
                        text.to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Boolean(boolean) => {
                    yield_all_ok!(
                        boolean.to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Endpoint(endpoint) => {
                    yield_all_ok!(
                        endpoint.to_instructions(compilation_context, cache)
                    );
                }

                DataExpressionData::Null => {
                    yield Ok(BytecodeItem::from(RegularInstruction::null()));
                }

                DataExpressionData::List(list) => {
                    yield_all!(list.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Map(map) => {
                    yield_all!(map.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // unary operations
                DataExpressionData::UnaryOperation(unary_operation) => {
                    yield_all!(unary_operation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // binary operations
                DataExpressionData::BinaryOperation(binary_operation) => {
                    yield_all!(binary_operation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // comparisons
                DataExpressionData::ComparisonOperation(
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
                DataExpressionData::Apply(apply) => {
                    yield_all!(apply.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // interface methods
                DataExpressionData::InterfaceMethodCall(
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
                DataExpressionData::PropertyAccess(property_access) => {
                    yield_all!(property_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::GenericInstantiation(
                    generic_instantiation,
                ) => {
                    yield_all!(generic_instantiation.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::PropertyAssignment(property_assignment) => {
                    yield_all!(property_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::RequestSharedRef(request_shared_ref) => {
                    yield_all!(request_shared_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::UnboxAssignment(unbox_assignment) => {
                    yield_all!(unbox_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // root property
                DataExpressionData::RootPropertyAccess(
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
                DataExpressionData::DeriveRef(derive_ref) => {
                    yield_all!(derive_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // shared refs
                DataExpressionData::DeriveSharedRef(derive_shared_ref) => {
                    yield_all!(derive_shared_ref.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                // shared values
                DataExpressionData::CreateShared(create_shared) => {
                    yield_all!(create_shared.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::TypeExpression(type_expression) => {
                    yield Ok(RegularInstruction::TypeExpression.into());
                    yield_all!(type_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Range(range_dec) => {
                    yield_all!(range_dec.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Unbox(unbox) => {
                    yield_all!(unbox.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Tag(tag_expression) => {
                    yield_all!(tag_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::ResolveCoreLibId(core_lib_id) => {
                    yield Ok(RegularInstruction::get_core_lib_value(
                        (*core_lib_id).into(),
                    )
                    .into());
                }

                DataExpressionData::CallableDeclaration(
                    callable_declaration,
                ) => {
                    yield_all!(callable_declaration.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Conditional(conditional) => {
                    yield_all!(conditional.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::RemoteExecution(remote_execution) => {
                    yield_all!(remote_execution.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::VariableDeclaration(
                    _variable_declaration,
                ) => {
                    yield_all!(_variable_declaration.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::Statements(statements) => {
                    yield_all!(statements.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::VariableAssignment(variable_assignment) => {
                    yield_all!(variable_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::VariableAccess(variable_access) => {
                    yield_all!(variable_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }

                DataExpressionData::Placeholder(value_access_type) => {
                    yield_all!(value_access_type.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::Noop => todo!(),
                DataExpressionData::NativeImplementationIndicator => todo!(),
                DataExpressionData::OmitRecursive => todo!(),
                DataExpressionData::Identifier(_) => todo!(),
                DataExpressionData::Sender => todo!(),
                DataExpressionData::Receiver => todo!(),
                DataExpressionData::MoveSharedValue(
                    _self_owned_pointer_address,
                ) => todo!(),
                DataExpressionData::Compile(compile_expression) => {
                    yield_all!(compile_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::TypeDeclaration(
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
                DataExpressionData::EntityDeclaration(
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
                DataExpressionData::CreateMut(create_mut) => {
                    yield_all!(create_mut.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::Clone(clone_expression) => {
                    yield_all!(clone_expression.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::StackIndex(_stack_index) => todo!(),
                DataExpressionData::StackAssignment(_stack_assignment) => {
                    todo!()
                }
                DataExpressionData::StackListAssignment(
                    stack_list_assignment,
                ) => {
                    yield_all!(stack_list_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::UnboxSlotAssignment(
                    unbox_slot_assignment,
                ) => {
                    yield_all!(unbox_slot_assignment.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::VariantAccess(variant_access) => {
                    yield_all!(variant_access.ast_node_to_instructions(
                        compilation_context,
                        cache,
                        ast_metadata,
                        ctx_metadata
                    ));
                }
                DataExpressionData::EntityValue(entity_value_expression) => {
                    yield_all!(
                        entity_value_expression.ast_node_to_instructions(
                            compilation_context,
                            cache,
                            ast_metadata,
                            ctx_metadata
                        )
                    );
                }

                DataExpressionData::Recover => unreachable!(),
            }
        })
    }
}
