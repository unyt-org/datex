use core::ops::Neg;

use crate::values::{value::Value, value_container::error::ValueError};

impl Neg for Value {
    type Output = Result<Value, ValueError>;

    fn neg(self) -> Self::Output {
        (-self.inner).map(Value::from)
    }
}
