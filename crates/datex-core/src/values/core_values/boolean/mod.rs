use crate::{
    prelude::*,
    traits::structural_eq::StructuralEq,
    values::value_container::{ValueContainer, error::ValueError},
};

use core::{fmt::Display, ops::Not, result::Result};
use serde::{Deserialize, Serialize};
pub mod equality;
pub mod ops;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Boolean(pub bool);

impl Boolean {
    pub fn as_bool(&self) -> bool {
        self.0
    }
}
impl Boolean {
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
    pub fn is_true(&self) -> bool {
        self.0
    }
    pub fn is_false(&self) -> bool {
        !self.0
    }
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
    pub fn as_str(&self) -> &str {
        if self.0 { "true" } else { "false" }
    }
}

impl Display for Boolean {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "{}", self.0)
    }
}

impl From<bool> for Boolean {
    fn from(v: bool) -> Self {
        Boolean(v)
    }
}

// new into
impl<T: Into<ValueContainer>> TryFrom<Option<T>> for Boolean {
    type Error = ValueError;
    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => {
                let boolean: ValueContainer = v.into();
                boolean.try_as().ok_or(ValueError::TypeConversionError)
            }
            None => Err(ValueError::IsVoid),
        }
    }
}
