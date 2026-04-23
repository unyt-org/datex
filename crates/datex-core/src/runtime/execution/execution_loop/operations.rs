use crate::{
    global::operators::{
        ArithmeticUnaryOperator, AssignmentOperator, BinaryOperator,
        ComparisonOperator, LogicalUnaryOperator, SharedValueUnaryOperator,
        UnaryOperator,
        binary::{
            ArithmeticOperator, BitwiseOperator, LogicalOperator, RangeOperator,
        },
    },
    runtime::execution::ExecutionError,
    traits::{
        identity::Identity, structural_eq::StructuralEq, value_eq::ValueEq,
    },
    values::{core_values::range::Range, value_container::ValueContainer},
};
use core::cell::RefCell;

use crate::{
    prelude::*,
    runtime::memory::Memory,
    shared_values::observers::TransceiverId,
    types::{r#type::Type, type_match::TypeMatch},
    value_updates::{
        update_data::SetEntryUpdateData, update_handler::UpdateHandler,
    },
    values::value_container::ValueKey,
};

pub fn set_property(
    target: &mut ValueContainer,
    key: ValueKey,
    value: ValueContainer,
) -> Result<(), ExecutionError> {
    target
        .try_set_entry(SetEntryUpdateData { key, value }, TransceiverId(0)) // TODO #644: set correct source id
        .map_err(ExecutionError::from)
}

pub fn handle_unary_shared_value_operation(
    operator: SharedValueUnaryOperator,
    value_container: ValueContainer,
    _memory: &RefCell<Memory>,
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
pub fn handle_unary_logical_operation(
    operator: LogicalUnaryOperator,
    _value_container: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    unimplemented!(
        "Logical unary operations are not implemented yet: {operator:?}"
    )
}
pub fn handle_unary_arithmetic_operation(
    operator: ArithmeticUnaryOperator,
    value_container: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        ArithmeticUnaryOperator::Minus => Ok((-value_container)?),
        ArithmeticUnaryOperator::Plus => Ok(value_container),
        _ => unimplemented!(
            "Arithmetic unary operations are not implemented yet: {operator:?}"
        ),
    }
}

pub fn handle_unary_operation(
    operator: UnaryOperator,
    value_container: ValueContainer,
    memory: &RefCell<Memory>,
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
        _ => {
            core::todo!("#102 Unary instruction not implemented: {operator:?}")
        }
    }
}

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
            let val = v_type.matched_by_value(lhs);
            Ok(ValueContainer::from(val))
        }
        _ => {
            unreachable!("Instruction {:?} is not a valid operation", operator);
        }
    }
}

pub fn handle_assignment_operation(
    operator: AssignmentOperator,
    lhs: &ValueContainer,
    rhs: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        AssignmentOperator::AddAssign => Ok((lhs + &rhs)?),
        AssignmentOperator::SubtractAssign => Ok((lhs - &rhs)?),
        _ => {
            unreachable!("Instruction {:?} is not a valid operation", operator);
        }
    }
}

pub fn handle_arithmetic_operation(
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
        _ => {
            core::todo!(
                "#408 Implement arithmetic operation for {:?}",
                operator
            );
        }
    }
}

pub fn handle_bitwise_operation(
    operator: BitwiseOperator,
    _lhs: &ValueContainer,
    _rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // apply operation to active value
    {
        core::todo!("#409 Implement bitwise operation for {:?}", operator);
    }
}

pub fn handle_logical_operation(
    operator: LogicalOperator,
    _lhs: &ValueContainer,
    _rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    // apply operation to active value
    {
        core::todo!("#410 Implement logical operation for {:?}", operator);
    }
}

pub fn handle_range_operation(
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
        _ => {
            core::todo!("#742 Implement range operation for {:?}", operator);
        }
    }
}

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
