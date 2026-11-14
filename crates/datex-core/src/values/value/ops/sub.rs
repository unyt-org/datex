use core::ops::Sub;

use crate::values::{value::Value, value_container::error::ValueError};

impl Sub for &Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Core(lhs), Value::Core(rhs)) => {
                lhs.sub(rhs).map(Value::Core)
            }
            (_, _) => {
                todo!()
            }
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, rhs: Value) -> Self::Output {
        &self - &rhs
    }
}
