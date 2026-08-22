use crate::{
    ast::expressions::{DatexExpression, DatexExpressionData},
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
};
impl ToInstructions for DatexExpression {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            match self.data() {
                DatexExpressionData::Integer(integer) => {
                    yield RegularInstruction::Integer(integer.clone())
                }
                DatexExpressionData::TypedInteger(typed_integer) => {
                    for instruction in typed_integer.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Instant(instant) => {
                    for instruction in instant.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Decimal(decimal) => {
                    for instruction in decimal.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::TypedDecimal(typed_decimal) => {
                    for instruction in typed_decimal.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Text(text) => {
                    for instruction in text.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Boolean(boolean) => {
                    for instruction in boolean.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Endpoint(endpoint) => {
                    for instruction in endpoint.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Null => {
                    yield RegularInstruction::null();
                }

                DatexExpressionData::List(list) => {
                    for instruction in list.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Map(map) => {
                    for instruction in map.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // unary operations
                DatexExpressionData::UnaryOperation(unary_operation) => {
                    for instruction in unary_operation.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // binary operations
                DatexExpressionData::BinaryOperation(binary_operation) => {
                    for instruction in binary_operation.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // comparisons
                DatexExpressionData::ComparisonOperation(
                    comparison_operation,
                ) => {
                    for instruction in comparison_operation.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }

                // apply
                DatexExpressionData::Apply(apply) => {
                    for instruction in apply.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // interface methods
                DatexExpressionData::InterfaceMethodCall(
                    interface_method_call,
                ) => {
                    for instruction in
                        interface_method_call.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }

                // property access
                DatexExpressionData::PropertyAccess(property_access) => {
                    for instruction in property_access.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::GenericInstantiation(
                    generic_instantiation,
                ) => {
                    for instruction in
                        generic_instantiation.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }

                DatexExpressionData::PropertyAssignment(
                    property_assignment,
                ) => {
                    for instruction in property_assignment.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }

                DatexExpressionData::RequestSharedRef(request_shared_ref) => {
                    for instruction in request_shared_ref.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::UnboxAssignment(unbox_assignment) => {
                    for instruction in unbox_assignment.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // root property
                DatexExpressionData::RootPropertyAccess(
                    root_property_access,
                ) => {
                    for instruction in root_property_access.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }

                // refs
                DatexExpressionData::DeriveRef(derive_ref) => {
                    for instruction in derive_ref.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // shared refs
                DatexExpressionData::DeriveSharedRef(derive_shared_ref) => {
                    for instruction in derive_shared_ref.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                // shared values
                DatexExpressionData::CreateShared(create_shared) => {
                    for instruction in create_shared.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::TypeExpression(type_expression) => {
                    yield RegularInstruction::TypeExpression;
                    for instruction in type_expression.to_instructions(ctx) {
                        todo!(
                            "Transparent yield of typeinstructions over egular instructions"
                        ); //yield instruction;
                    }
                }

                DatexExpressionData::Range(range_dec) => {
                    for instruction in range_dec.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Unbox(unbox) => {
                    for instruction in unbox.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::Tag(tag_expression) => {
                    for instruction in tag_expression.to_instructions(ctx) {
                        yield instruction;
                    }
                }

                DatexExpressionData::ResolveCoreLibId(core_lib_id) => {
                    yield RegularInstruction::get_core_lib_value(
                        (*core_lib_id).into(),
                    );
                }

                DatexExpressionData::CallableDeclaration(
                    callable_declaration,
                ) => {
                    todo!("Need context and scope data and stuff");
                }

                DatexExpressionData::Conditional(conditional) => {
                    todo!(
                        "Need actual byte positions and not instruction counts for jumps"
                    );
                }
                DatexExpressionData::RemoteExecution(remote_execution) => {
                    todo!("Need context and scope data and stuff");
                }

                DatexExpressionData::VariableDeclaration(
                    variable_declaration,
                ) => {
                    todo!("Need context and scope data and stuff");
                }
                DatexExpressionData::Statements(statements) => {
                    todo!("Need context and scope data and stuff");
                }

                DatexExpressionData::VariableAssignment(
                    variable_assignment,
                ) => {
                    todo!("Need context and scope data and stuff");
                }

                DatexExpressionData::VariableAccess(variable_access) => {
                    todo!("Need context and scope data and stuff");
                }

                e => panic!(
                    "Expression to instruction not implemented for {:?}",
                    e
                ),
            }
        })
    }
}
