//! This module contains the implementation of operations that can be performed on [ValueContainer]s
use crate::{
    global::operators::{
        ArithmeticUnaryOperator, BinaryOperator, ComparisonOperator,
        LogicalUnaryOperator, SharedValueUnaryOperator, UnaryOperator,
        binary::{
            ArithmeticOperator, BitwiseOperator, LogicalOperator, RangeOperator,
        },
    },
    runtime::execution::ExecutionError,
    shared_values::{ReferenceMutability, SharedContainer},
    traits::{
        identity::Identity, structural_eq::StructuralEq, value_eq::ValueEq,
    },
    values::{
        core_value::CoreValue, core_values::range::Range,
        value_container::ValueContainer,
    },
};
use core::cell::RefCell;

use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::traits::SharedContainerCommon,
    types::{traits::type_match::TypeSatisfiesValueContainer, r#type::Type},
};

/// Handles a binary operation between two [ValueContainer]s based on the specified [BinaryOperator].
fn handle_unary_shared_value_operation(
    operator: SharedValueUnaryOperator,
    value_container: ValueContainer,
    _memory: &RefCell<SharedReferencesCache>,
) -> Result<ValueContainer, ExecutionError> {
    Ok(match operator {
        SharedValueUnaryOperator::Unbox => {
            if let ValueContainer::Shared(reference) = value_container {
                reference.value_container().clone()
            } else {
                return Err(ExecutionError::InvalidUnbox);
            }
        }
    })
}

/// Handles a unary operation on a [ValueContainer] based on the specified [UnaryOperator].
fn handle_unary_logical_operation(
    operator: LogicalUnaryOperator,
    value_container: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        LogicalUnaryOperator::Not => {
            Ok(ValueContainer::from(!expect_boolean(&value_container)?))
        }
    }
}

/// Returns the boolean value of a [ValueContainer], or an error if it is not a boolean
/// Used for conditions (if, while) and logical operators, which do not perform implicit conversions
pub fn expect_boolean(
    value_container: &ValueContainer,
) -> Result<bool, ExecutionError> {
    match &value_container.collapsed_value().borrow().inner {
        CoreValue::Boolean(boolean) => Ok(boolean.as_bool()),
        _ => Err(ExecutionError::ExpectedBooleanValue),
    }
}

/// Handles an arithmetic unary operation on a [ValueContainer] based on the specified [ArithmeticUnaryOperator].
fn handle_unary_arithmetic_operation(
    operator: ArithmeticUnaryOperator,
    value_container: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        ArithmeticUnaryOperator::Minus => Ok((-value_container)?),
        ArithmeticUnaryOperator::Plus => Ok(value_container),
        _ => Err(ExecutionError::NotImplemented(format!(
            "Arithmetic unary operation {operator:?}"
        ))),
    }
}

/// Handles a unary operation on a [ValueContainer] based on the specified [UnaryOperator].
pub fn handle_unary_operation(
    operator: UnaryOperator,
    value_container: ValueContainer,
    memory: &RefCell<SharedReferencesCache>,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        UnaryOperator::Reference(reference) => {
            handle_unary_shared_value_operation(
                reference,
                value_container,
                memory,
            )
        }
        UnaryOperator::Logical(logical) => {
            handle_unary_logical_operation(logical, value_container)
        }
        UnaryOperator::Arithmetic(arithmetic) => {
            handle_unary_arithmetic_operation(arithmetic, value_container)
        }
        _ => Err(ExecutionError::NotImplemented(format!(
            "Unary operation {operator:?}"
        ))),
    }
}

/// Handles a comparison operation between two [ValueContainer]s based on the specified [ComparisonOperator].
pub fn handle_comparison_operation(
    operator: ComparisonOperator,
    lhs: &ValueContainer,
    rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // apply operation to active value
    match operator {
        ComparisonOperator::StructuralEqual => {
            let val = lhs.structural_eq(rhs);
            Ok(ValueContainer::from(val))
        }
        ComparisonOperator::Equal => {
            let val = lhs.value_eq(rhs);
            Ok(ValueContainer::from(val))
        }
        ComparisonOperator::NotStructuralEqual => {
            let val = !lhs.structural_eq(rhs);
            Ok(ValueContainer::from(val))
        }
        ComparisonOperator::NotEqual => {
            let val = !lhs.value_eq(rhs);
            Ok(ValueContainer::from(val))
        }
        ComparisonOperator::Is => {
            // TODO #103 we should throw a runtime error when one of lhs or rhs is a value
            // instead of a ref. Identity checks using the is operator shall be only allowed
            // for references.
            // @benstre: or keep as always false ? - maybe a compiler check would be better
            let val = lhs.identical(rhs);
            Ok(ValueContainer::from(val))
        }
        ComparisonOperator::Matches => {
            // TODO #407: Fix matches, rhs will always be a type, so actual_type() call is wrong
            let v_type = Type::try_from(rhs.clone())
                .map_err(|_| ExecutionError::ExpectedTypeValue)?;
            let val = v_type.satisfies_value_container(lhs);
            Ok(ValueContainer::from(val))
        }
        // TODO: ordering comparisons (<, <=, >, >=) need a value ordering
        _ => Err(ExecutionError::NotImplemented(format!(
            "Comparison operation {operator:?}"
        ))),
    }
}

/// Handles an arithmetic operation between two [ValueContainer]s based on the specified [ArithmeticOperator].
fn handle_arithmetic_operation(
    operator: ArithmeticOperator,
    lhs: &ValueContainer,
    rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // apply operation to active value
    match operator {
        ArithmeticOperator::Add => Ok((lhs + rhs)?),
        ArithmeticOperator::Subtract => Ok((lhs - rhs)?),
        // ArithmeticOperator::Multiply => {
        //     Ok((active_value_container * &value_container)?)
        // }
        // ArithmeticOperator::Divide => {
        //     Ok((active_value_container / &value_container)?)
        // }
        _ => Err(ExecutionError::NotImplemented(format!(
            "#408 Arithmetic operation {operator:?}"
        ))),
    }
}

/// Handles a bitwise operation between two [ValueContainer]s based on the specified [BitwiseOperator].
fn handle_bitwise_operation(
    operator: BitwiseOperator,
    _lhs: &ValueContainer,
    _rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    Err(ExecutionError::NotImplemented(format!(
        "#409 Bitwise operation {operator:?}"
    )))
}

/// Handles a logical operation between two [ValueContainer]s based on the specified [LogicalOperator].
fn handle_logical_operation(
    operator: LogicalOperator,
    _lhs: &ValueContainer,
    _rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // and/or are compiled to short circuiting control flow instructions and never reach this point
    Err(ExecutionError::NotImplemented(format!(
        "#410 Logical operation {operator:?}"
    )))
}

/// Handles a range operation between two [ValueContainer]s based on the specified [RangeOperator].
fn handle_range_operation(
    operator: RangeOperator,
    lhs: &ValueContainer,
    rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // match operator and return range
    match operator {
        RangeOperator::Inclusive => Ok(ValueContainer::from(Range {
            start: Box::new(lhs.clone()),
            end: Box::new(rhs.clone()),
        })),
        _ => Err(ExecutionError::NotImplemented(format!(
            "#742 Range operation {operator:?}"
        ))),
    }
}

/// Handles a binary operation between two [ValueContainer]s based on the specified [BinaryOperator].
pub fn handle_binary_operation(
    operator: BinaryOperator,
    lhs: &ValueContainer,
    rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        BinaryOperator::Arithmetic(arith_op) => {
            handle_arithmetic_operation(arith_op, lhs, rhs)
        }
        BinaryOperator::Bitwise(bitwise_op) => {
            handle_bitwise_operation(bitwise_op, lhs, rhs)
        }
        BinaryOperator::Logical(logical_op) => {
            handle_logical_operation(logical_op, lhs, rhs)
        }
        BinaryOperator::Range(range_op) => {
            handle_range_operation(range_op, lhs, rhs)
        }
    }
}

/// Derives a shared reference from the given target [ValueContainer].
/// If the target is not a shared container, an [ExecutionError::ExpectedSharedValue] is returned.
/// The derived reference will have the specified [ReferenceMutability].
/// If a mutable reference is requested for a non-mutable shared container, an [ExecutionError::MutableReferenceToNonMutableValue] is returned.
pub fn derive_shared_reference(
    target: &ValueContainer,
    mutability: ReferenceMutability,
) -> Result<ValueContainer, ExecutionError> {
    // value_container must be a shared value, otherwise we cannot create a reference to it
    if let ValueContainer::Shared(shared) = target {
        Ok(ValueContainer::Shared(SharedContainer::Referenced(
            match mutability {
                ReferenceMutability::Immutable => {
                    Ok(shared.derive_immutable_reference())
                }
                ReferenceMutability::Mutable => {
                    shared.try_derive_mutable_reference().map_err(|_| {
                        ExecutionError::MutableReferenceToNonMutableValue
                    })
                }
            }?,
        )))
    } else {
        Err(ExecutionError::ExpectedSharedValue)
    }
}
