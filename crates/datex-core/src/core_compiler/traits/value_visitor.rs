use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking,
    types::r#type::Type, values::value_container::ValueContainer,
};
use core::cell::RefCell;

pub trait ValueVisitor<'ctx> {
    fn visit_value_container(&mut self, value: &ValueContainer);
    fn visit_type(&mut self, ty: &Type);

    fn shared_value_tracking(
        &self,
    ) -> Option<&RefCell<SharedValueTracking<'ctx>>>;
}
