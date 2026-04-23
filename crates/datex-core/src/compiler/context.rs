use crate::{
    core_compiler::{
        core_compilation_context::CoreCompilationContext,
        value_compiler::append_instruction_code_new,
    },
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::instruction_data::StackIndex,
    },
    prelude::*,
    runtime::execution::context::ExecutionMode,
    utils::buffers::append_u32,
    values::value_container::ValueContainer,
};
use binrw::io::Cursor;

/// compilation context, created for each compiler call, even if compiling a script for the same scope
pub struct CompilationContext {
    pub core_context: CoreCompilationContext,
    pub inserted_value_index: usize,
    pub inserted_values: Vec<Option<ValueContainer>>,
    /// this flag is set to true if any non-static value is encountered
    pub has_non_static_value: bool,
    pub execution_mode: ExecutionMode,
}

impl CompilationContext {
    const MAX_INT_32: i64 = 2_147_483_647;
    const MIN_INT_32: i64 = -2_147_483_648;

    const MAX_INT_8: i64 = 127;
    const MIN_INT_8: i64 = -128;

    const MAX_INT_16: i64 = 32_767;
    const MIN_INT_16: i64 = -32_768;

    const MAX_UINT_16: i64 = 65_535;

    const INT_8_BYTES: u8 = 1;
    const INT_16_BYTES: u8 = 2;
    const INT_32_BYTES: u8 = 4;
    const INT_64_BYTES: u8 = 8;
    const INT_128_BYTES: u8 = 16;

    const FLOAT_32_BYTES: u8 = 4;
    const FLOAT_64_BYTES: u8 = 8;

    pub fn new(
        buffer: Vec<u8>,
        inserted_values: Vec<Option<ValueContainer>>,
        execution_mode: ExecutionMode,
    ) -> Self {
        CompilationContext {
            inserted_value_index: 0,
            core_context: CoreCompilationContext::new(buffer),
            inserted_values,
            has_non_static_value: false,
            execution_mode,
        }
    }

    pub fn buffer_index(&self) -> u64 {
        self.core_context.cursor().position()
    }

    pub fn cursor(&mut self) -> &mut Cursor<Vec<u8>> {
        self.core_context.cursor_mut()
    }

    pub fn into_buffer(self) -> Vec<u8> {
        self.core_context.into_buffer()
    }

    pub fn core_context(&mut self) -> &mut CoreCompilationContext {
        &mut self.core_context
    }

    pub fn insert_stack_index(&mut self, stack_index: StackIndex) {
        append_u32(self.cursor(), stack_index.0);
    }

    pub fn set_u32_at_index(&mut self, u32: u32, index: usize) {
        let buf = self.cursor().get_mut();
        buf[index..index + CompilationContext::INT_32_BYTES as usize]
            .copy_from_slice(&u32.to_le_bytes());
    }

    pub fn mark_has_non_static_value(&mut self) {
        self.has_non_static_value = true;
    }

    pub fn append_instruction_code(&mut self, code: InstructionCode) {
        append_instruction_code_new(self.cursor(), code);
    }
}
