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
    values::{
        core_values::range::Range,
        value_container::{OwnedValueKey, ValueContainer},
    },
};
use core::cell::RefCell;

use crate::{prelude::*, runtime::memory::Memory};

pub fn set_property(
    target: &mut ValueContainer,
    key: OwnedValueKey,
    value: ValueContainer,
) -> Result<(), ExecutionError> {
    target
        .try_set_property(
            0, // TODO #644: set correct source id
            None, key, value,
        )
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
                reference.value_container()
            } else {
                return Err(ExecutionError::InvalidUnbox);
            }
        }
    })
}

// use crate::runtime::ExecutionContext::Local;
// use crate::values::value_container::ValueContainer::Shared;

pub fn handle_unary_logical_operation(
    operator: LogicalUnaryOperator,
    value_container: ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    match operator {
        LogicalUnaryOperator::Not => {
            let val = value_container
                .to_value()
                .borrow()
                .cast_to_bool()
                .unwrap(); // No unwrap

            Ok(ValueContainer::from(!val))
        }
    }
}

// pub fn handle_unary_logical_operation(
//     operator: LogicalUnaryOperator,
//     value: ValueContainer,
// ) -> Result<ValueContainer, ExecutionError> {
//     todo!("implement unary logical operation")
// }

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
            let v_type = rhs.actual_container_type(); // Type::try_from(value_container)?;
            let val = v_type.value_matches(lhs);
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
    // apply operation to active value
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
        ArithmeticOperator::Multiply => {
            Ok((lhs * rhs)?)
        }
        ArithmeticOperator::Divide => {
            Ok((lhs / rhs)?)
        }
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
    lhs: &ValueContainer,
    rhs: &ValueContainer,
) -> Result<ValueContainer, ExecutionError> {
    let lhs_bool = lhs
        .to_value()
        .borrow()
        .cast_to_bool()
        .unwrap();

    let rhs_bool = rhs
        .to_value()
        .borrow()
        .cast_to_bool()
        .unwrap();

    let result = match operator {
        LogicalOperator::And => lhs_bool.0 && rhs_bool.0,
        LogicalOperator::Or => lhs_bool.0 || rhs_bool.0,
    };
    
    Ok(ValueContainer::from(result))
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


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handle_logical_operation_and() {
        let lhs = ValueContainer::from(true);
        let rhs = ValueContainer::from(true);
        let result = handle_logical_operation(LogicalOperator::And, &lhs, &rhs).unwrap();
        assert_eq!(result, ValueContainer::from(true));
    }
    #[test]
    fn test_handle_unary_logical_operation_not() {
        let val = ValueContainer::from(true);
        let result = handle_unary_logical_operation(LogicalUnaryOperator::Not, val).unwrap();
        assert_eq!(result, ValueContainer::from(false));
    }
}