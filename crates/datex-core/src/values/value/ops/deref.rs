use core::ops::Deref;

use crate::values::{core_value::CoreValue, value::Value};

impl Deref for Value {
    type Target = CoreValue;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
