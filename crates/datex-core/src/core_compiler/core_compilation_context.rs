use crate::{
    core_compiler::{
        buffer_provider::BufferProvider,
        preamble::append_injected_values_preamble,
        shared_value_tracking::SharedValueTracking,
        value_compiler::{append_inline_shared_container, append_value},
        value_visitor::ValueVisitor,
    },
    prelude::*,
    shared_values::SharedContainer,
    types::r#type::Type,
    values::value_container::ValueContainer,
};
use binrw::io::Cursor;

pub type ByteCursor = Cursor<Vec<u8>>;

pub struct CoreCompilationContext {
    pub cursor: ByteCursor,
    pub shared_value_tracking: SharedValueTracking,
}

impl CoreCompilationContext {
    /// Create a new core compilation context with an initial byte input buffer and starting slot address for shared value tracking
    pub fn new(buffer: Vec<u8>) -> CoreCompilationContext {
        CoreCompilationContext {
            cursor: Cursor::new(buffer),
            shared_value_tracking: SharedValueTracking::new(),
        }
    }

    pub fn cursor(&self) -> &Cursor<Vec<u8>> {
        &self.cursor
    }

    pub fn into_buffer(self) -> Vec<u8> {
        self.cursor.into_inner()
    }

    /// Finalizes the compilation context by appending a preamble with the injected shared values,
    /// and returns the final byte buffer and the list of shared values that were moved or referenced during compilation
    pub fn into_buffer_and_shared_values(
        self,
    ) -> (Vec<u8>, Vec<SharedContainer>) {
        let mut cursor = self.cursor;
        let tracked_values = self.shared_value_tracking.into_tracked_values();

        let top_level_values =
            append_injected_values_preamble(&mut cursor, tracked_values);

        (cursor.into_inner(), top_level_values)
    }
}

impl BufferProvider for CoreCompilationContext {
    fn cursor_mut(&mut self) -> &mut ByteCursor {
        &mut self.cursor
    }
}

impl ValueVisitor for CoreCompilationContext {
    /// Appends a value container.
    /// For local values, the value is just serialized
    /// For shared values, the container is registered in the context shared value tracking
    fn visit_value_container(&mut self, value_container: ValueContainer) {
        match value_container {
            ValueContainer::Local(value) => append_value(self, value),
            ValueContainer::Shared(reference) => {
                append_inline_shared_container(self, reference);
            }
        }
    }

    fn visit_type(&mut self, _ty: Type) {
        todo!()
    }
}
