use crate::core_compiler::shared_value_tracking::SharedValueTracking;
use crate::global::protocol_structures::instruction_data::StackIndex;
use binrw::io::Cursor;
use crate::prelude::*;

pub type ByteCursor = Cursor<Vec<u8>>;

pub struct CoreCompilationContext {
    cursor: ByteCursor,
    shared_value_tracking: SharedValueTracking
}

impl CoreCompilationContext {
    /// Create a new core compilation context with an initial byte input buffer and starting slot address for shared value tracking
    pub fn new(
        buffer: Vec<u8>,
        start_address: StackIndex
    ) -> CoreCompilationContext {
        CoreCompilationContext {
            cursor: Cursor::new(buffer),
            shared_value_tracking: SharedValueTracking::new(start_address),
        }
    }

    pub fn cursor(&self) -> &Cursor<Vec<u8>> {
        &self.cursor
    }

    pub fn into_buffer(self) -> Vec<u8> {
        self.cursor.into_inner()
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.cursor
    }
}