use crate::{
    core_compiler::{
        core_compilation_context::CoreCompilationContext,
        value_compiler::{
            append_instruction_code_new, append_regular_instruction,
        },
    },
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::{
            instruction_data::{JumpData, StackIndex},
            regular_instructions::RegularInstruction,
        },
    },
    prelude::*,
    runtime::execution::context::ExecutionMode,
    utils::buffers::append_u32,
    values::value_container::ValueContainer,
};
use binrw::io::Cursor;

#[derive(Clone, Debug)]
pub struct PendingJump {
    pub label_id: u32,
    pub patch_position: usize,
    pub extra_read_ahead: u32,
}

/// compilation context, created for each compiler call, even if compiling a script for the same scope
pub struct CompilationContext {
    pub core_context: CoreCompilationContext,
    pub inserted_value_index: usize,
    pub inserted_values: Vec<Option<ValueContainer>>,
    /// this flag is set to true if any non-static value is encountered
    pub has_non_static_value: bool,
    pub execution_mode: ExecutionMode,
    pub next_label_id: u32,
    pub labels: HashMap<u32, usize>,
    pub pending_jumps: Vec<PendingJump>,
    /// Label to jump to for `return` statements (inside a function body)
    pub return_target_label: Option<u32>,
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
            next_label_id: 0,
            labels: HashMap::new(),
            pending_jumps: Vec::new(),
            return_target_label: None,
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

    pub fn new_label(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    pub fn bind_label(&mut self, label_id: u32) {
        let pos = self.cursor().position() as usize;
        self.labels.insert(label_id, pos);
    }

    pub fn emit_jump_to_label(&mut self, target_label: u32) {
        let patch_position = self.cursor().position() as usize + 1;
        append_regular_instruction(
            self.cursor(),
            RegularInstruction::Jump(JumpData { offset: 0 }),
        );
        self.pending_jumps.push(PendingJump {
            label_id: target_label,
            patch_position,
            extra_read_ahead: 0,
        });
    }

    pub fn emit_jump_if_false_to_label(
        &mut self,
        target_label: u32,
        condition_bytes_len: u32,
    ) {
        let patch_position = self.cursor().position() as usize + 1;
        append_regular_instruction(
            self.cursor(),
            RegularInstruction::JumpIfFalse(JumpData { offset: 0 }),
        );
        self.pending_jumps.push(PendingJump {
            label_id: target_label,
            patch_position,
            extra_read_ahead: condition_bytes_len,
        });
    }

    pub fn resolve_pending_jumps(&mut self) {
        let jumps: Vec<PendingJump> = self.pending_jumps.drain(..).collect();
        for jump in &jumps {
            let label_pos = self.labels[&jump.label_id];
            let offset = label_pos as i32
                - (jump.patch_position as i32
                    + 4
                    + jump.extra_read_ahead as i32);
            let buf = self.cursor().get_mut();
            buf[jump.patch_position..jump.patch_position + 4]
                .copy_from_slice(&(offset as u32).to_le_bytes());
        }
    }

    pub fn mark_has_non_static_value(&mut self) {
        self.has_non_static_value = true;
    }

    pub fn append_instruction_code(&mut self, code: InstructionCode) {
        append_instruction_code_new(self.cursor(), code);
    }
}
