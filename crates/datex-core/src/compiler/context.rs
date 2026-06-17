use crate::{
    collections::HashMap,
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
use core::mem;

struct PendingJump {
    placeholder_index: usize,
    target_label: u32,
}

/// compilation context, created for each compiler call, even if compiling a script for the same scope
pub struct CompilationContext {
    pub core_context: CoreCompilationContext,
    pub inserted_value_index: usize,
    pub inserted_values: Vec<Option<ValueContainer>>,
    /// this flag is set to true if any non-static value is encountered
    pub has_non_static_value: bool,
    pub execution_mode: ExecutionMode,
    pub return_target_label: Option<u32>,
    next_label_id: u32,
    labels: HashMap<u32, u64>,
    pending_jumps: Vec<PendingJump>,
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
            return_target_label: None,
            next_label_id: 0,
            labels: HashMap::new(),
            pending_jumps: Vec::new(),
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

    pub fn set_i32_at_index(&mut self, value: i32, index: usize) {
        let buf = self.cursor().get_mut();
        buf[index..index + CompilationContext::INT_32_BYTES as usize]
            .copy_from_slice(&value.to_le_bytes());
    }

    pub fn append_i32(&mut self, value: i32) {
        let cursor = self.cursor();
        cursor.get_mut().extend_from_slice(&value.to_le_bytes());
        cursor.set_position(cursor.position() + mem::size_of::<i32>() as u64);
    }

    pub fn append_relative_jump_placeholder(&mut self) -> usize {
        let index = self.buffer_index() as usize;
        self.append_i32(0);
        index
    }

    pub fn patch_relative_jump(
        &mut self,
        placeholder_index: usize,
        target_index: usize,
    ) {
        let offset = target_index as i64
            - (placeholder_index as i64
                + CompilationContext::INT_32_BYTES as i64);
        self.set_i32_at_index(offset as i32, placeholder_index);
    }

    pub fn mark_has_non_static_value(&mut self) {
        self.has_non_static_value = true;
    }

    pub fn append_instruction_code(&mut self, code: InstructionCode) {
        append_instruction_code_new(self.cursor(), code);
    }

    pub fn new_label(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    pub fn bind_label(&mut self, label: u32) {
        self.labels.insert(label, self.buffer_index());
        self.resolve_pending_jumps_for_label(label);
    }

    pub fn emit_jump_to_label(&mut self, label: u32) {
        self.append_instruction_code(InstructionCode::JUMP);
        let placeholder = self.append_relative_jump_placeholder();
        if let Some(&target_index) = self.labels.get(&label) {
            let offset = target_index as i64
                - (placeholder as i64 + CompilationContext::INT_32_BYTES as i64);
            self.set_i32_at_index(offset as i32, placeholder);
        } else {
            self.pending_jumps.push(PendingJump {
                placeholder_index: placeholder,
                target_label: label,
            });
        }
    }

    pub fn emit_jump_if_false_to_label(&mut self, label: u32) {
        self.append_instruction_code(InstructionCode::JUMP_IF_FALSE);
        let placeholder = self.append_relative_jump_placeholder();
        if let Some(&target_index) = self.labels.get(&label) {
            let offset = target_index as i64
                - (placeholder as i64 + CompilationContext::INT_32_BYTES as i64);
            self.set_i32_at_index(offset as i32, placeholder);
        } else {
            self.pending_jumps.push(PendingJump {
                placeholder_index: placeholder,
                target_label: label,
            });
        }
    }

    fn resolve_pending_jumps_for_label(&mut self, label: u32) {
        if let Some(&target_index) = self.labels.get(&label) {
            let mut i = 0;
            while i < self.pending_jumps.len() {
                if self.pending_jumps[i].target_label == label {
                    let jump = self.pending_jumps.swap_remove(i);
                    let offset = target_index as i64
                        - (jump.placeholder_index as i64
                            + CompilationContext::INT_32_BYTES as i64);
                    self.set_i32_at_index(offset as i32, jump.placeholder_index);
                } else {
                    i += 1;
                }
            }
        }
    }
}
