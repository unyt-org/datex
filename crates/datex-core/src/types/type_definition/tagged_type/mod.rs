use crate::{
    global::operators::ModificationOperator,
    prelude::*,
    types::{traits::operator_handler::OperatorHandler, r#type::Type},
    value_updates::update_data::UpdateModificationOperator,
};
use core::fmt::Display;

pub mod serde_dif;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaggedTypeDefinition {
    pub tag: String,
    pub ty: Option<Box<Type>>,
}
impl Display for TaggedTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "#{} {}", self.tag, ty)
        } else {
            write!(f, "#{}", self.tag)
        }
    }
}

impl OperatorHandler for TaggedTypeDefinition {
    fn get_update_type_for_modification(
        &self,
        operator: ModificationOperator,
    ) -> Result<UpdateModificationOperator, ()> {
        if let Some(ty) = &self.ty {
            ty.get_update_type_for_modification(operator)
        } else {
            Err(())
        }
    }
}
