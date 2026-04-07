//! This is just a default `bool` type, same as in other languages
//! 
//! You can create `bool` in `if` condition with any variable type
//! 
//! # Example of usage in DATEX
//! ```datex
//! var test_bool = true;
//! !test_bool # will output false
//! 
//! var test_var = "test";
//! var is_true = test_var == "test"; # is_true become: true
//! 
//! test_bool or is_true # will output true, same as in Rust var1 || var2
//! ```

use crate::{
    prelude::*,
    traits::structural_eq::StructuralEq,
    values::value_container::{ValueContainer, ValueError},
};

use core::{fmt::Display, ops::Not, result::Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Boolean(pub bool);

impl Boolean {
    /// Return `bool` value from [`Boolean`]
    pub fn as_bool(&self) -> bool {
        self.0
    }
}
impl Boolean {
    
    /// Swap `bool` value
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    /// Return `true` if `value==true` else `false`
    pub fn is_true(&self) -> bool {
        self.0
    }

    /// Return `true` if `value==false` else `true`
    pub fn is_false(&self) -> bool {
        !self.0
    }

    /// Return value as `String`, e.g "true" or "false"
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }

    /// Return value as `&str`, e.g "true" or "false"
    pub fn as_str(&self) -> &str {
        if self.0 { "true" } else { "false" }
    }
}

impl Display for Boolean {
    /// Allows printing Boolean
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "{}", self.0)
    }
}

impl StructuralEq for Boolean {
    /// Allows compare Boolean
    fn structural_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<bool> for Boolean {
    /// Create [`Boolean`] from `bool`
    fn from(v: bool) -> Self {
        Boolean(v)
    }
}

impl Not for Boolean {
    type Output = Boolean;
    /// Swap Boolean value
    fn not(self) -> Self::Output {
        Boolean(!self.0)
    }
}

impl<T: Into<ValueContainer>> TryFrom<Option<T>> for Boolean {
    type Error = ValueError;
    /// Tries to convert ValueContainer into `bool`, need for compiler conditions check
    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        match value {
            Some(v) => {
                let boolean: ValueContainer = v.into();
                boolean
                    .to_value()
                    .borrow()
                    .cast_to_bool()
                    .ok_or(ValueError::TypeConversionError)
            }
            None => Err(ValueError::IsVoid),
        }
    }
}
