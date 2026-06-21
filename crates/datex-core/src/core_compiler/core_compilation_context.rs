use crate::{
    core_compiler::shared_value_tracking::SharedValueTracking, prelude::*,
    shared_values::OwnedSharedContainer,
};
use binrw::io::Cursor;
use crate::core_compiler::preamble::append_injected_values_preamble;
use crate::shared_values::SharedContainer;

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

        let top_level_values = append_injected_values_preamble(
            &mut cursor,
            tracked_values,
        );

        (
            cursor.into_inner(),
            top_level_values,
        )
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.cursor
    }

}
