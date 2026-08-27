use crate::{
    core_compiler::buffer_provider::BufferProvider, types::r#type::Type,
    values::value_container::ValueContainer,
};

pub trait ValueVisitor: BufferProvider {
    fn visit_value_container(&mut self, value: &ValueContainer);
    fn visit_type(&mut self, ty: &Type);
}
