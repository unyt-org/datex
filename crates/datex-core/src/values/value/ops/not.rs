use core::ops::Not;

use crate::values::{value::Value, value_container::error::ValueError};

impl Not for Value {
    type Output = Option<Value>;

    fn not(self) -> Self::Output {
        (!self.inner).map(Value::from)
    }
}
