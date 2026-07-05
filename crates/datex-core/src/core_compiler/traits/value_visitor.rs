use core::fmt::Display;

use crate::{
    core_compiler::buffer_provider::BufferProvider,
    shared_values::SharedContainer,
    types::r#type::Type,
    values::value_container::{ValueContainer, value_key::ValueKey},
};

#[derive(Debug, Clone)]
pub enum ParentAccessor {
    ValueKey(ValueKey),
    KeyValue,
    DirectAssignment,
}
impl Display for ParentAccessor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParentAccessor::ValueKey(key) => write!(f, "ValueKey({})", key),
            ParentAccessor::KeyValue => write!(f, "KeyValue"),
            ParentAccessor::DirectAssignment => write!(f, "DirectAssignment"),
        }
    }
}

impl From<ValueKey> for ParentAccessor {
    fn from(value_key: ValueKey) -> Self {
        ParentAccessor::ValueKey(value_key)
    }
}

#[derive(Debug, Clone)]
pub struct ParentContext {
    pub(crate) parent: SharedContainer,
    pub(crate) accessors: Vec<ParentAccessor>, // TODO: also support direct ref ("newtype" struct) assignments
}

impl ParentContext {
    pub fn new(parent: SharedContainer) -> Self {
        Self {
            parent,
            accessors: vec![],
        }
    }

    pub fn with_accessor(self, index: impl Into<ParentAccessor>) -> Self {
        ParentContext {
            parent: self.parent,
            accessors: {
                let mut new_path = self.accessors;
                new_path.push(index.into());
                new_path
            },
        }
    }
}

pub trait ValueVisitor: BufferProvider {
    fn visit_value_container(
        &mut self,
        value: ValueContainer,
        parent_context: Option<ParentContext>,
    );
    fn visit_type(&mut self, ty: Type);
}
