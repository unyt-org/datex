use core::ops::AddAssign;

use crate::values::value::Value;

// TODO #119: crate a TryAddAssign trait etc.
impl<T> AddAssign<T> for Value
where
    Value: From<T>,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs: Value = rhs.into();
        match (self, rhs) {
            (Value::Core(lhs), Value::Core(rhs)) => {
                *lhs += rhs;
            }
            (Value::Core(lhs), Value::Native(rhs)) => {
                *lhs += rhs;
            }
            (_, Value::Core(_)) => {
                todo!("Handle add assign for non-Core and Core values")
            }
            (_, _) => {
                todo!("Handle add assign for non-Core values")
            }
        }
    }
}
