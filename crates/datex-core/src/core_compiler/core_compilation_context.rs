use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking, prelude::*,
    shared_values::shared_containers::OwnedSharedContainer,
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

    pub fn into_buffer_and_moved_values(
        self,
    ) -> (Vec<u8>, Vec<OwnedSharedContainer>) {
        (
            self.cursor.into_inner(),
            self.shared_value_tracking.into_moved_shared_values(),
        )
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.cursor
    }
}
