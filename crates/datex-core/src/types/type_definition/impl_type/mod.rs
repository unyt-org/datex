use crate::{
    global::operators::ModificationOperator,
    prelude::*,
    shared_values::PointerAddress,
    types::{traits::operator_handler::OperatorHandler, r#type::Type},
    value_updates::update_data::UpdateModificationOperator,
};
use core::fmt::Display;
pub mod serde_dif;
mod type_match;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplTypeDefinition {
    pub inner_type: Box<Type>,
    pub impl_markers: Vec<PointerAddress>,
}

impl ImplTypeDefinition {
    pub fn new(inner_type: Type, impl_markers: Vec<PointerAddress>) -> Self {
        Self {
            inner_type: Box::new(inner_type),
            impl_markers,
        }
    }
}

impl OperatorHandler for ImplTypeDefinition {
    fn get_update_type_for_modification(
        &self,
        operator: ModificationOperator,
    ) -> Result<UpdateModificationOperator, ()> {
        self.inner_type.get_update_type_for_modification(operator)
    }
}

impl Display for ImplTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner_type)?;
        for marker in &self.impl_markers {
            write!(f, " + {}", marker)?;
        }
        Ok(())
    }
}
