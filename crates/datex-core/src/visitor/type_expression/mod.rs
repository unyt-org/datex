//! This module contains the implementation of the visitor pattern for traversing and transforming the AST of [TypeExpression]s.
use crate::{
    ast::type_expressions::RangeTypeExpr,
    prelude::*,
    values::core_values::{boolean::Boolean, text::Text},
};
use core::ops::Range;

use crate::{
    ast::{
        expressions::VariableAccess,
        type_expressions::{
            CallableTypeExpression, FixedSizeList, GenericAccess,
            IdentifierWithPointerAddress, Intersection, SliceList,
            StructuralList, StructuralMap, TypeExpression, TypeExpressionData,
            TypeVariantAccess, Union,
        },
    },
    libs::core::type_id::CoreLibTypeId,
    shared_values::PointerAddress,
    values::core_values::{
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        integer::{Integer, typed_integer::TypedInteger},
    },
    visitor::{
        VisitAction,
        type_expression::visitable::{
            TypeExpressionVisitResult, VisitableTypeExpression,
        },
    },
};

pub mod visitable;

pub trait TypeExpressionVisitor<E>: Sized {
    /// Handle type expression error
    /// Can either propagate the error or return a VisitAction to recover
    /// Per default, it just propagates the error
    fn handle_type_expression_error(
        &mut self,
        error: E,
        expression: &TypeExpression,
    ) -> Result<VisitAction<TypeExpression>, E> {
        let _ = expression;
        Err(error)
    }

    fn before_visit_type_expression(
        &mut self,
        expression: &mut TypeExpression,
    ) {
        let _ = expression;
    }
    fn after_visit_type_expression(&mut self, expression: &mut TypeExpression) {
        let _ = expression;
    }

    /// Visit type expression
    fn visit_type_expression(
        &mut self,
        expr: &mut TypeExpression,
    ) -> Result<(), E> {
        self.before_visit_type_expression(expr);

        let span = expr.span.clone();

        let visit_result = match expr.data_mut() {
            TypeExpressionData::VariantAccess(variant_access) => {
                self.visit_variant_access_type(variant_access, &span)
            }
            TypeExpressionData::GetReference(pointer_address) => {
                self.visit_get_reference_type(pointer_address, &span)
            }
            TypeExpressionData::VariableAccess(variable_access) => {
                self.visit_variable_access_type(variable_access, &span)
            }
            TypeExpressionData::Integer(integer) => {
                self.visit_integer_type(integer, &span)
            }
            TypeExpressionData::TypedInteger(typed_integer) => {
                self.visit_typed_integer_type(typed_integer, &span)
            }
            TypeExpressionData::Decimal(decimal) => {
                self.visit_decimal_type(decimal, &span)
            }
            TypeExpressionData::TypedDecimal(typed_decimal) => {
                self.visit_typed_decimal_type(typed_decimal, &span)
            }
            TypeExpressionData::Boolean(boolean) => {
                self.visit_boolean_type(boolean, &span)
            }
            TypeExpressionData::Text(text) => self.visit_text_type(text, &span),
            TypeExpressionData::Endpoint(endpoint) => {
                self.visit_endpoint_type(endpoint, &span)
            }
            TypeExpressionData::StructuralList(structual_list) => {
                self.visit_structural_list_type(structual_list, &span)
            }
            TypeExpressionData::FixedSizeList(fixed_size_list) => {
                self.visit_fixed_size_list_type(fixed_size_list, &span)
            }
            TypeExpressionData::SliceList(slice_list) => {
                self.visit_slice_list_type(slice_list, &span)
            }
            TypeExpressionData::Intersection(intersection) => {
                self.visit_intersection_type(intersection, &span)
            }
            TypeExpressionData::Union(union) => {
                self.visit_union_type(union, &span)
            }
            TypeExpressionData::GenericAccess(generic_access) => {
                self.visit_generic_access_type(generic_access, &span)
            }
            TypeExpressionData::Callable(callable_type_expression) => {
                self.visit_callable_type(callable_type_expression, &span)
            }
            TypeExpressionData::StructuralMap(structural_map) => {
                self.visit_structural_map_type(structural_map, &span)
            }
            TypeExpressionData::Ref(type_ref) => {
                self.visit_ref_type(type_ref, &span)
            }
            TypeExpressionData::RefMut(type_ref_mut) => {
                self.visit_ref_mut_type(type_ref_mut, &span)
            }
            TypeExpressionData::Shared(type_shared) => {
                self.visit_shared_type(type_shared, &span)
            }
            TypeExpressionData::Mut(type_mut) => {
                self.visit_mut_type(type_mut, &span)
            }
            TypeExpressionData::Identifier(identifier) => {
                self.visit_type_identifier(identifier, &span)
            }
            TypeExpressionData::IdentifierWithPointerAddress(identifier) => {
                self.visit_type_identifier_with_pointer_address(
                    identifier, &span,
                )
            }
            TypeExpressionData::Range(range) => {
                self.visit_range_type(range, &span)
            }
            TypeExpressionData::Recover => {
                unreachable!("Recover expression should not be visited")
            }
            TypeExpressionData::GetCoreLibType(core_lib_type_id) => {
                self.visit_get_core_lib_type(core_lib_type_id, &span)
            }
        };
        let action = match visit_result {
            Ok(action) => action,
            Err(e) => self.handle_type_expression_error(e, expr)?,
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
                expr.data = Box::new(TypeExpressionData::null());
                Ok(())
            }
            VisitAction::ContinueRecursion => expr.walk_children(self),
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
                self.visit_type_expression(expr)?;
                Ok(())
            }
        };
        self.after_visit_type_expression(expr);
        result
    }

    /// Visit identifier expression
    fn visit_type_identifier(
        &mut self,
        literal: &mut String,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = literal;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit identifier with pointer address expression
    fn visit_type_identifier_with_pointer_address(
        &mut self,
        identifier_with_pointer_address: &mut IdentifierWithPointerAddress,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = identifier_with_pointer_address;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit structural list type expression
    fn visit_structural_list_type(
        &mut self,
        structural_list: &mut StructuralList,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = structural_list;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit fixed size list type expression
    fn visit_fixed_size_list_type(
        &mut self,
        fixed_size_list: &mut FixedSizeList,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = fixed_size_list;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit slice list type expression
    fn visit_slice_list_type(
        &mut self,
        slice_list: &mut SliceList,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = slice_list;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit intersection type expression
    fn visit_intersection_type(
        &mut self,
        intersection: &mut Intersection,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = intersection;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit union type expression
    fn visit_union_type(
        &mut self,
        union: &mut Union,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = union;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit generic access type expression
    fn visit_generic_access_type(
        &mut self,
        generic_access: &mut GenericAccess,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = generic_access;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit function type expression
    fn visit_callable_type(
        &mut self,
        callable_type_expression: &mut CallableTypeExpression,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = callable_type_expression;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit structural map type expression
    fn visit_structural_map_type(
        &mut self,
        structural_map: &mut StructuralMap,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = structural_map;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit type reference expression
    fn visit_ref_type(
        &mut self,
        type_ref: &mut TypeExpression,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = type_ref;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit mutable type reference expression
    fn visit_ref_mut_type(
        &mut self,
        type_ref_mut: &mut TypeExpression,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = type_ref_mut;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_shared_type(
        &mut self,
        type_shared: &mut TypeExpression,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = type_shared;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_mut_type(
        &mut self,
        type_mut: &mut TypeExpression,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = type_mut;
        Ok(VisitAction::ContinueRecursion)
    }

    /// Visit integer literal
    fn visit_integer_type(
        &mut self,
        integer: &mut Integer,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = integer;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit typed integer literal
    fn visit_typed_integer_type(
        &mut self,
        typed_integer: &mut TypedInteger,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = typed_integer;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit decimal literal
    fn visit_decimal_type(
        &mut self,
        decimal: &mut Decimal,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = decimal;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit typed decimal literal
    fn visit_typed_decimal_type(
        &mut self,
        typed_decimal: &mut TypedDecimal,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = typed_decimal;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit text literal
    fn visit_text_type(
        &mut self,
        text: &mut Text,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = text;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit get reference expression
    fn visit_get_reference_type(
        &mut self,
        pointer_address: &mut PointerAddress,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = pointer_address;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit variant access expression
    fn visit_variant_access_type(
        &mut self,
        variant_access: &mut TypeVariantAccess,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = variant_access;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit boolean literal
    fn visit_boolean_type(
        &mut self,
        boolean: &mut Boolean,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = boolean;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit endpoint expression
    fn visit_endpoint_type(
        &mut self,
        endpoint: &mut Endpoint,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = endpoint;
        Ok(VisitAction::AbortRecursion)
    }

    /// Visit variable access
    fn visit_variable_access_type(
        &mut self,
        var_access: &mut VariableAccess,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = var_access;
        Ok(VisitAction::AbortRecursion)
    }

    fn visit_range_type(
        &mut self,
        range: &mut RangeTypeExpr,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = range;
        Ok(VisitAction::ContinueRecursion)
    }

    fn visit_get_core_lib_type(
        &mut self,
        core_lib_type_id: &mut CoreLibTypeId,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<E> {
        let _ = span;
        let _ = core_lib_type_id;
        Ok(VisitAction::AbortRecursion)
    }
}
