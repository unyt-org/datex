//! This module contains the implementation of the visitor pattern for traversing and transforming the AST of [DatexExpression]s.
pub mod visitable;
use crate::{
    ast::expressions::{
        Apply, BinaryOperation, CallableDeclaration, CloneExpression,
        ComparisonOperation, CompileExpression, Conditional, CreateMut,
        CreateShared, DatexExpression, DatexExpressionData, DeriveRef,
        DeriveSharedRef, EntityDeclarationExpression, EntityValueExpression,
        GenericInstantiation, InterfaceMethodCall, List, Map, PropertyAccess,
        PropertyAssignment, RemoteExecution, RequestSharedRef,
        RootPropertyAccess, StackAssignment, StackListAssignment, Statements,
        TagExpression, TypeDeclarationExpression, UnaryOperation, Unbox,
        UnboxAssignment, UnboxSlotAssignment, ValueAccessType, VariableAccess,
        VariableAssignment, VariableDeclaration, VariantAccess,
    },
    global::protocol_structures::instruction_data::StackIndex,
    libs::core::core_lib_id::CoreLibId,
    prelude::*,
    values::core_values::{
        Instant,
        boolean::Boolean,
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        integer::{Integer, typed_integer::TypedInteger},
        text::Text,
    },
    visitor::{
        VisitAction,
        expression::visitable::{ExpressionVisitResult, VisitableExpression},
        type_expression::TypeExpressionVisitor,
    },
};
use core::ops::Range;

pub trait ExpressionVisitor<E>: TypeExpressionVisitor<E> {
    /// Handle expression error
    /// Can either propagate the error or return a VisitAction to recover
    /// Per default, it just propagates the error
    fn handle_expression_error(
        &mut self,
        error: E,
        expression: &DatexExpression,
    ) -> Result<VisitAction<DatexExpression>, E> {
        let _ = expression;
        Err(error)
    }

    fn before_visit_datex_expression(
        &mut self,
        expression: &mut DatexExpression,
    ) {
        let _ = expression;
    }
    fn after_visit_datex_expression(
        &mut self,
        expression: &mut DatexExpression,
    ) {
        let _ = expression;
    }

    /// Visit datex expression
    fn visit_datex_expression(
        &mut self,
        expr: &mut DatexExpression,
    ) -> Result<(), E> {
        self.before_visit_datex_expression(expr);
        let visit_result = match &mut *expr.data {
            DatexExpressionData::PropertyAssignment(property_assignment) => {
                self.visit_property_assignment(property_assignment, &expr.span)
            }
            DatexExpressionData::VariantAccess(variant_access) => {
                self.visit_variant_access(variant_access, &expr.span)
            }
            DatexExpressionData::UnaryOperation(op) => {
                self.visit_unary_operation(op, &expr.span)
            }
            DatexExpressionData::Statements(stmts) => {
                self.visit_statements(stmts, &expr.span)
            }
            DatexExpressionData::VariableDeclaration(var_decl) => {
                self.visit_variable_declaration(var_decl, &expr.span)
            }
            DatexExpressionData::VariableAssignment(var_assign) => {
                self.visit_variable_assignment(var_assign, &expr.span)
            }
            DatexExpressionData::VariableAccess(var_access) => {
                self.visit_variable_access(var_access, &expr.span)
            }
            DatexExpressionData::Integer(i) => {
                self.visit_integer(i, &expr.span)
            }
            DatexExpressionData::DateTime(dt) => {
                self.visit_datetime(dt, &expr.span)
            }
            DatexExpressionData::Range(range) => {
                self.visit_range(range, &expr.span)
            }
            DatexExpressionData::TypedInteger(ti) => {
                self.visit_typed_integer(ti, &expr.span)
            }
            DatexExpressionData::Decimal(d) => {
                self.visit_decimal(d, &expr.span)
            }
            DatexExpressionData::TypedDecimal(td) => {
                self.visit_typed_decimal(td, &expr.span)
            }
            DatexExpressionData::Text(s) => self.visit_text(s, &expr.span),
            DatexExpressionData::Boolean(b) => {
                self.visit_boolean(b, &expr.span)
            }
            DatexExpressionData::Endpoint(e) => {
                self.visit_endpoint(e, &expr.span)
            }
            DatexExpressionData::Null => self.visit_null(&expr.span),
            DatexExpressionData::List(list) => {
                self.visit_list(list, &expr.span)
            }
            DatexExpressionData::Map(map) => self.visit_map(map, &expr.span),
            DatexExpressionData::RequestSharedRef(request_shared_ref) => self
                .visit_request_shared_reference(request_shared_ref, &expr.span),
            DatexExpressionData::Conditional(conditional) => {
                self.visit_conditional(conditional, &expr.span)
            }
            DatexExpressionData::TypeDeclaration(type_declaration) => {
                self.visit_type_declaration(type_declaration, &expr.span)
            }
            DatexExpressionData::EntityDeclaration(entity_declaration) => {
                self.visit_entity_declaration(entity_declaration, &expr.span)
            }
            DatexExpressionData::TypeExpression(type_expression) => self
                .visit_type_expression(type_expression)
                .map(|_| VisitAction::AbortRecursion),
            DatexExpressionData::CallableDeclaration(callable_declaration) => {
                self.visit_callable_declaration(
                    callable_declaration,
                    &expr.span,
                )
            }
            DatexExpressionData::DeriveRef(get_ref) => {
                self.visit_get_ref(get_ref, &expr.span)
            }
            DatexExpressionData::DeriveSharedRef(get_shared_ref) => {
                self.visit_get_shared_ref(get_shared_ref, &expr.span)
            }
            DatexExpressionData::CreateShared(create_shared) => {
                self.visit_create_shared(create_shared, &expr.span)
            }
            DatexExpressionData::CreateMut(create_mut) => {
                self.visit_create_mut(create_mut, &expr.span)
            }
            DatexExpressionData::Unbox(unbox) => {
                self.visit_unbox(unbox, &expr.span)
            }
            DatexExpressionData::Clone(clone) => {
                self.visit_clone(clone, &expr.span)
            }
            DatexExpressionData::StackIndex(slot) => {
                self.visit_stack_index(slot, &expr.span)
            }
            DatexExpressionData::EntityValue(entity_value) => {
                self.visit_entity_value(entity_value, &expr.span)
            }
            DatexExpressionData::StackAssignment(stack_assignment) => {
                self.visit_stack_assignment(stack_assignment, &expr.span)
            }
            DatexExpressionData::StackListAssignment(stack_list_assignment) => {
                self.visit_stack_list_assignment(
                    stack_list_assignment,
                    &expr.span,
                )
            }
            DatexExpressionData::BinaryOperation(binary_operation) => {
                self.visit_binary_operation(binary_operation, &expr.span)
            }
            DatexExpressionData::ComparisonOperation(comparison_operation) => {
                self.visit_comparison_operation(
                    comparison_operation,
                    &expr.span,
                )
            }
            DatexExpressionData::UnboxAssignment(unbox_assignment) => {
                self.visit_unbox_assignment(unbox_assignment, &expr.span)
            }
            DatexExpressionData::UnboxSlotAssignment(unbox_slot_assignment) => {
                self.visit_unbox_slot_assignment(
                    unbox_slot_assignment,
                    &expr.span,
                )
            }
            DatexExpressionData::Apply(apply) => {
                self.visit_apply(apply, &expr.span)
            }
            DatexExpressionData::InterfaceMethodCall(call) => {
                self.visit_interface_method_call(call, &expr.span)
            }
            DatexExpressionData::PropertyAccess(property_access) => {
                self.visit_property_access(property_access, &expr.span)
            }
            DatexExpressionData::GenericInstantiation(
                generic_instantiation,
            ) => self
                .visit_generic_instantiation(generic_instantiation, &expr.span),
            DatexExpressionData::RemoteExecution(remote_execution) => {
                self.visit_remote_execution(remote_execution, &expr.span)
            }
            DatexExpressionData::Identifier(identifier) => {
                self.visit_identifier(identifier, &expr.span)
            }
            DatexExpressionData::Placeholder(placeholder_type) => {
                self.visit_placeholder(placeholder_type, &expr.span)
            }
            DatexExpressionData::Recover => {
                unreachable!(
                    "Placeholder and Recover expressions should not be visited"
                )
            }
            DatexExpressionData::Noop => Ok(VisitAction::AbortRecursion),
            DatexExpressionData::NativeImplementationIndicator => {
                Ok(VisitAction::AbortRecursion)
            }
            DatexExpressionData::Compile(compile_expression) => {
                self.visit_compile_expression(compile_expression, &expr.span)
            }
            DatexExpressionData::Tag(tag) => {
                self.visit_tag_expression(tag, &expr.span)
            }
            DatexExpressionData::RootPropertyAccess(root_property_access) => {
                self.visit_root_property_access(
                    root_property_access,
                    &expr.span,
                )
            }
            DatexExpressionData::ResolveCoreLibId(core_lib_id) => {
                self.visit_get_core_lib_id(core_lib_id, &expr.span)
            }
            DatexExpressionData::OmitRecursive => {
                unreachable!("Omit expressions should not be visited")
            }
            DatexExpressionData::MoveSharedValue(_) => {
                todo!("Move shared value visit?");
            }
        };

        let action = match visit_result {
            Ok(act) => act,
            Err(error) => self.handle_expression_error(error, expr)?,
        };
        let result = match action {
            VisitAction::SetTypeRecurseChildNodes(type_annotation) => {
                expr.ty = Some(type_annotation);
                expr.walk_children(self)?;
                Ok(())
            }
            VisitAction::SetTypeSkipChildren(type_annotation) => {
                expr.ty = Some(type_annotation);
                Ok(())
            }
            VisitAction::AbortRecursion => Ok(()),
            VisitAction::ToNoop => {
                *expr.data = DatexExpressionData::Noop;
                Ok(())
            }
            VisitAction::ContinueRecursion => {
                expr.walk_children(self)?;
                Ok(())
            }
            VisitAction::Replace(new_expr) => {
                *expr = new_expr.to_owned();
                Ok(())
            }
            VisitAction::ReplaceRecurseChildNodes(new_expr) => {
                expr.walk_children(self)?;
                *expr = new_expr.to_owned();
                Ok(())
            }
            VisitAction::ReplaceRecurse(new_expr) => {
                *expr = new_expr.to_owned();
                self.visit_datex_expression(expr)?;
                Ok(())
            }
        };
        self.after_visit_datex_expression(expr);
        result
    }

    /// Visit statements
    fn visit_statements(
        &mut self,
        statements: &mut Statements,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = statements;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit unary operation
    fn visit_unary_operation(
        &mut self,
        unary_operation: &mut UnaryOperation,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = unary_operation;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit variant access
    fn visit_variant_access(
        &mut self,
        variant_access: &mut VariantAccess,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = variant_access;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit property assignment
    fn visit_property_assignment(
        &mut self,
        property_assignment: &mut PropertyAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = property_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit conditional expression
    fn visit_conditional(
        &mut self,
        conditional: &mut Conditional,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = conditional;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit type declaration
    fn visit_type_declaration(
        &mut self,
        type_declaration: &mut TypeDeclarationExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = type_declaration;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit entity declaration
    fn visit_entity_declaration(
        &mut self,
        entity_declaration: &mut EntityDeclarationExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = entity_declaration;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit binary operation
    fn visit_binary_operation(
        &mut self,
        binary_operation: &mut BinaryOperation,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = binary_operation;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit comparison operation
    fn visit_comparison_operation(
        &mut self,
        comparison_operation: &mut ComparisonOperation,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = comparison_operation;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit unbox assignment
    fn visit_unbox_assignment(
        &mut self,
        unbox_assignment: &mut UnboxAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = unbox_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit unbox slot assignment
    fn visit_unbox_slot_assignment(
        &mut self,
        unbox_slot_assignment: &mut UnboxSlotAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = unbox_slot_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit apply
    fn visit_apply(
        &mut self,
        apply: &mut Apply,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = apply;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit interface method call
    fn visit_interface_method_call(
        &mut self,
        call: &mut InterfaceMethodCall,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = call;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit property access
    fn visit_property_access(
        &mut self,
        property_access: &mut PropertyAccess,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = property_access;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit generic instantiation
    fn visit_generic_instantiation(
        &mut self,
        generic_instantiation: &mut GenericInstantiation,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = generic_instantiation;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit remote execution
    fn visit_remote_execution(
        &mut self,
        remote_execution: &mut RemoteExecution,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = remote_execution;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit callable declaration
    fn visit_callable_declaration(
        &mut self,
        callable_declaration: &mut CallableDeclaration,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = callable_declaration;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_compile_expression(
        &mut self,
        compile_expression: &mut CompileExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = compile_expression;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_tag_expression(
        &mut self,
        tag: &mut TagExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = tag;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_root_property_access(
        &mut self,
        root_property_access: &mut RootPropertyAccess,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = root_property_access;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit stack assignment
    fn visit_stack_assignment(
        &mut self,
        stack_assignment: &mut StackAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = stack_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit stack list assignment
    fn visit_stack_list_assignment(
        &mut self,
        stack_list_assignment: &mut StackListAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = stack_list_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit variable declaration
    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &mut VariableDeclaration,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = variable_declaration;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit variable assignment
    fn visit_variable_assignment(
        &mut self,
        variable_assignment: &mut VariableAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = variable_assignment;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit variable access
    fn visit_variable_access(
        &mut self,
        var_access: &mut VariableAccess,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = var_access;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit create reference expression
    fn visit_get_ref(
        &mut self,
        create_ref: &mut DeriveRef,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = create_ref;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit create shared reference expression
    fn visit_get_shared_ref(
        &mut self,
        get_shared_ref: &mut DeriveSharedRef,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = get_shared_ref;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit create shared value expression
    fn visit_create_shared(
        &mut self,
        create_shared: &mut CreateShared,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = create_shared;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit create mut value expression
    fn visit_create_mut(
        &mut self,
        create_mut: &mut CreateMut,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = create_mut;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit unbox expression
    fn visit_unbox(
        &mut self,
        unbox: &mut Unbox,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = unbox;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit clone expression
    fn visit_clone(
        &mut self,
        clone: &mut CloneExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = clone;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit list expression
    fn visit_list(
        &mut self,
        list: &mut List,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = list;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit map expression
    fn visit_map(
        &mut self,
        map: &mut Map,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = map;
        let _ = span;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit integer literal
    fn visit_integer(
        &mut self,
        integer: &mut Integer,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = integer;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit datetime literal
    fn visit_datetime(
        &mut self,
        _instant: &mut Instant,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit typed integer literal
    fn visit_typed_integer(
        &mut self,
        typed_integer: &mut TypedInteger,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = typed_integer;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit decimal literal
    fn visit_decimal(
        &mut self,
        decimal: &mut Decimal,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = decimal;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit typed decimal literal
    fn visit_typed_decimal(
        &mut self,
        typed_decimal: &mut TypedDecimal,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = typed_decimal;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit identifier
    fn visit_identifier(
        &mut self,
        identifier: &mut String,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = identifier;
        Ok(VisitAction::AbortRecursion)
    }

    fn visit_placeholder(
        &mut self,
        placeholder_type: &mut ValueAccessType,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = placeholder_type;
        Ok(VisitAction::AbortRecursion)
    }

    fn visit_get_core_lib_id(
        &mut self,
        core_lib_id: &mut CoreLibId,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = core_lib_id;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit text literal
    fn visit_text(
        &mut self,
        text: &mut Text,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = text;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit request reference expression
    fn visit_request_shared_reference(
        &mut self,
        get_shared_ref: &mut RequestSharedRef,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = get_shared_ref;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit boolean literal
    fn visit_boolean(
        &mut self,
        boolean: &mut Boolean,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = boolean;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit endpoint expression
    fn visit_endpoint(
        &mut self,
        endpoint: &mut Endpoint,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = endpoint;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit null literal
    fn visit_null(&mut self, span: &Range<usize>) -> ExpressionVisitResult<E> {
        let _ = span;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit stack index expression
    fn visit_stack_index(
        &mut self,
        stack_index: &StackIndex,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = stack_index;
        Ok(VisitAction::AbortRecursion)
    }

    fn visit_range(
        &mut self,
        range: &mut crate::ast::expressions::RangeDeclaration,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = range;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_entity_value(
        &mut self,
        entity_value: &mut EntityValueExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<E> {
        let _ = span;
        let _ = entity_value;
        Ok(VisitAction::ContinueRecursion)
    }
}
