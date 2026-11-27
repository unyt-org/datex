use crate::{
    ast::expressions::{
        Apply, BinaryOperation, CallableDeclaration, CallableSignature,
        CloneExpression, ComparisonOperation, Conditional, CreateMut,
        CreateShared, NormalExpression, NormalExpressionData, DeriveRef,
        DeriveSharedRef, EntityDeclarationExpression, EntityValueExpression,
        GenericInstantiation, InterfaceMethodCall, List, Map, PropertyAccess,
        PropertyAssignment, RangeDeclaration, RemoteExecution, StackAssignment,
        StackListAssignment, Statements, Tagged, TypeDeclarationExpression,
        UnaryOperation, Unbox, UnboxAssignment, UnboxSlotAssignment,
        VariableAssignment, VariableDeclaration, expression::Expression,
    },
    visitor::{
        VisitAction, expression::ExpressionVisitor,
        type_expression::visitable::VisitableTypeExpression,
    },
};

pub type ExpressionVisitResult<E> = Result<VisitAction<NormalExpression>, E>;

pub trait VisitableExpression<E> {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E>;
}

impl<E> VisitableExpression<E> for RangeDeclaration {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.start)?;
        visitor.visit_datex_expression(&mut self.end)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for BinaryOperation {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.left)?;
        visitor.visit_datex_expression(&mut self.right)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for Statements {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        for item in &mut self.statements {
            visitor.visit_datex_expression(item)?;
        }
        Ok(())
    }
}
impl<E> VisitableExpression<E> for List {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        for item in &mut self.items {
            visitor.visit_datex_expression(item)?;
        }
        Ok(())
    }
}
impl<E> VisitableExpression<E> for Map {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        for (_key, value) in &mut self.entries {
            visitor.visit_datex_expression(value)?;
        }
        Ok(())
    }
}
impl<E> VisitableExpression<E> for Conditional {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.condition)?;
        visitor.visit_datex_expression(&mut self.then_branch)?;
        if let Some(else_branch) = &mut self.else_branch {
            visitor.visit_datex_expression(else_branch)?;
        }
        Ok(())
    }
}
impl<E> VisitableExpression<E> for VariableDeclaration {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.init_expression)?;
        if let Some(type_annotation) = &mut self.type_annotation {
            visitor.visit_type_expression(type_annotation)?;
        }
        Ok(())
    }
}
impl<E> VisitableExpression<E> for VariableAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}
impl<E> VisitableExpression<E> for UnaryOperation {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}
impl<E> VisitableExpression<E> for TypeDeclarationExpression {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_type_expression(&mut self.definition)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for EntityDeclarationExpression {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_type_expression(&mut self.definition)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for ComparisonOperation {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.left)?;
        visitor.visit_datex_expression(&mut self.right)?;
        Ok(())
    }
}
impl<E> VisitableExpression<E> for UnboxAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.assigned_expression)?;
        visitor.visit_datex_expression(&mut self.unbox_expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for UnboxSlotAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.assigned_expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for Tagged {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        if let Some(expression) = &mut self.expression {
            visitor.visit_datex_expression(expression)?;
        }
        Ok(())
    }
}

impl<E> VisitableExpression<E> for EntityValueExpression {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.value)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for Apply {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.base)?;
        for arg in &mut self.arguments {
            visitor.visit_datex_expression(arg)?;
        }
        Ok(())
    }
}

impl<E> VisitableExpression<E> for InterfaceMethodCall {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.target)?;
        for arg in &mut self.arguments {
            visitor.visit_datex_expression(arg)?;
        }
        Ok(())
    }
}

impl<E> VisitableExpression<E> for PropertyAccess {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.base)?;
        visitor.visit_datex_expression(&mut self.property)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for GenericInstantiation {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.base)?;
        for arg in &mut self.generic_arguments {
            visitor.visit_type_expression(arg)?;
        }
        Ok(())
    }
}

impl<E> VisitableExpression<E> for RemoteExecution {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.left)?;
        visitor.visit_datex_expression(&mut self.right)?;
        Ok(())
    }
}
impl<E> VisitableExpression<E> for StackAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for StackListAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for CallableDeclaration {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        self.signature.walk_children(visitor)?;
        visitor.visit_datex_expression(&mut self.body)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for CallableSignature {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        if let Some(return_type) = &mut self.return_type {
            visitor.visit_type_expression(return_type)?;
        }
        if let Some(yeet_type) = &mut self.yeet_type {
            visitor.visit_type_expression(yeet_type)?;
        }
        for (_, param_type) in &mut self.parameters {
            visitor.visit_type_expression(param_type)?;
        }
        if let Some(rest_parameter) = &mut self.rest_parameter {
            visitor.visit_type_expression(&mut rest_parameter.1)?;
        }
        Ok(())
    }
}

impl<E> VisitableExpression<E> for Unbox {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for CloneExpression {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for DeriveRef {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for DeriveSharedRef {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for CreateShared {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for CreateMut {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for PropertyAssignment {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        visitor.visit_datex_expression(&mut self.base)?;
        visitor.visit_datex_expression(&mut self.property)?;
        visitor.visit_datex_expression(&mut self.assigned_expression)?;
        Ok(())
    }
}

impl<E> VisitableExpression<E> for NormalExpression {
    fn walk_children(
        &mut self,
        visitor: &mut impl ExpressionVisitor<E>,
    ) -> Result<(), E> {
        match self.data_mut() {
            NormalExpressionData::PropertyAssignment(property_assignment) => {
                property_assignment.walk_children(visitor)
            }
            NormalExpressionData::BinaryOperation(op) => {
                op.walk_children(visitor)
            }
            NormalExpressionData::Statements(statements) => {
                statements.walk_children(visitor)
            }
            NormalExpressionData::List(list) => list.walk_children(visitor),
            NormalExpressionData::Map(map) => map.walk_children(visitor),
            NormalExpressionData::Conditional(conditional) => {
                conditional.walk_children(visitor)
            }
            NormalExpressionData::VariableDeclaration(variable_declaration) => {
                variable_declaration.walk_children(visitor)
            }
            NormalExpressionData::VariableAssignment(variable_assignment) => {
                variable_assignment.walk_children(visitor)
            }
            NormalExpressionData::TypeDeclaration(type_declaration) => {
                type_declaration.walk_children(visitor)
            }
            NormalExpressionData::EntityDeclaration(entity_declaration) => {
                entity_declaration.walk_children(visitor)
            }
            NormalExpressionData::TypeExpression(type_expression) => {
                type_expression.walk_children(visitor)
            }
            NormalExpressionData::CallableDeclaration(function_declaration) => {
                function_declaration.walk_children(visitor)
            }
            NormalExpressionData::DeriveRef(create_ref) => {
                create_ref.walk_children(visitor)
            }
            NormalExpressionData::DeriveSharedRef(create_shared_ref) => {
                create_shared_ref.walk_children(visitor)
            }
            NormalExpressionData::CreateShared(create_ref) => {
                create_ref.walk_children(visitor)
            }
            NormalExpressionData::CreateMut(create_mut) => {
                create_mut.walk_children(visitor)
            }
            NormalExpressionData::Unbox(datex_expression) => {
                datex_expression.walk_children(visitor)
            }
            NormalExpressionData::Clone(datex_expression) => {
                datex_expression.walk_children(visitor)
            }
            NormalExpressionData::StackAssignment(stack_assignment) => {
                stack_assignment.walk_children(visitor)
            }
            NormalExpressionData::StackListAssignment(stack_list_assignment) => {
                stack_list_assignment.walk_children(visitor)
            }
            NormalExpressionData::ComparisonOperation(comparison_operation) => {
                comparison_operation.walk_children(visitor)
            }
            NormalExpressionData::UnboxAssignment(unbox_assignment) => {
                unbox_assignment.walk_children(visitor)
            }
            NormalExpressionData::UnboxSlotAssignment(unbox_slot_assignment) => {
                unbox_slot_assignment.walk_children(visitor)
            }
            NormalExpressionData::UnaryOperation(unary_operation) => {
                unary_operation.walk_children(visitor)
            }
            NormalExpressionData::Apply(apply) => apply.walk_children(visitor),
            NormalExpressionData::InterfaceMethodCall(call) => {
                call.walk_children(visitor)
            }
            NormalExpressionData::PropertyAccess(property_access) => {
                property_access.walk_children(visitor)
            }
            NormalExpressionData::GenericInstantiation(generic_instantiation) => {
                generic_instantiation.walk_children(visitor)
            }
            NormalExpressionData::RemoteExecution(remote_execution) => {
                remote_execution.walk_children(visitor)
            }
            NormalExpressionData::Compile(compile_expression) => {
                compile_expression.expression.walk_children(visitor)
            }

            NormalExpressionData::Range(range) => range.walk_children(visitor),

            NormalExpressionData::Tag(tag) => tag.walk_children(visitor),

            NormalExpressionData::EntityValue(entity_value) => {
                entity_value.walk_children(visitor)
            }

            NormalExpressionData::Noop
            | NormalExpressionData::OmitRecursive
            | NormalExpressionData::NativeImplementationIndicator
            | NormalExpressionData::VariantAccess(_)
            | NormalExpressionData::VariableAccess(_)
            | NormalExpressionData::RequestSharedRef(_)
            | NormalExpressionData::StackIndex(_)
            | NormalExpressionData::Placeholder(_)
            | NormalExpressionData::Recover
            | NormalExpressionData::Null
            | NormalExpressionData::Boolean(_)
            | NormalExpressionData::Text(_)
            | NormalExpressionData::Decimal(_)
            | NormalExpressionData::TypedDecimal(_)
            | NormalExpressionData::Integer(_)
            | NormalExpressionData::TypedInteger(_)
            | NormalExpressionData::Identifier(_)
            | NormalExpressionData::RootPropertyAccess(_)
            | NormalExpressionData::ResolveCoreLibId(_)
            | NormalExpressionData::Endpoint(_)
            | NormalExpressionData::MoveSharedValue(_)
            | NormalExpressionData::Sender
            | NormalExpressionData::Receiver
            | NormalExpressionData::Instant(_) => Ok(()),
        }
    }
}
