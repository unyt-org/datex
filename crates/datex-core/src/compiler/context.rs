use crate::{
    ast::type_expressions::TypeExpression,
    compiler::scope::CompilationScope,
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::{
            CompileInput, CoreCompilationContext, DXBWithSharedValues,
        },
        shared_value_tracking::SharedValueTracking,
        to_instructions::{SharedValueTrackingProvider, ToInstructions},
        type_compiler::append_type_instruction,
        value_compiler::append_instruction_code,
    },
    global::stack_index::StackIndex,
    instruction::{
        instruction_codes::InstructionCode, type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::execution::context::ExecutionMode,
    utils::buffers::append_u32,
    values::value_container::ValueContainer,
};
use binrw::{BinWrite, io::Cursor, meta::WriteEndian};
use core::cell::RefCell;
/// compilation context, created for each compiler call, even if compiling a script for the same scope
pub struct CompilationContext<'a> {
    pub core_context: CoreCompilationContext<'a>,
    pub inserted_value_index: usize,
    pub inserted_values: Vec<Option<ValueContainer>>,
    /// this flag is set to true if any non-static value is encountered
    pub has_non_static_value: bool,
    pub execution_mode: ExecutionMode,
    pub scope: CompilationScope,
}

impl<'a> CompilationContext<'a> {
    const INT_32_BYTES: u8 = 4;

    pub fn new(
        buffer: Vec<u8>,
        inserted_values: Vec<Option<ValueContainer>>,
        execution_mode: ExecutionMode,
        input: CompileInput<'a>,
    ) -> Self {
        CompilationContext {
            inserted_value_index: 0,
            core_context: CoreCompilationContext::new(buffer, input),
            inserted_values,
            has_non_static_value: false,
            execution_mode,
            scope: CompilationScope::new(ExecutionMode::default()),
        }
    }

    pub fn buffer_index(&self) -> u64 {
        self.core_context.cursor().position()
    }

    pub fn cursor(&mut self) -> &mut Cursor<Vec<u8>> {
        self.core_context.cursor_mut()
    }

    pub fn into_dxb_with_shared_values(self) -> DXBWithSharedValues {
        self.core_context.into_dxb_with_shared_values()
    }

    pub fn core_context(&mut self) -> &mut CoreCompilationContext<'a> {
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

    pub fn write<T: BinWrite + WriteEndian>(&mut self, value: T)
    where
        for<'b> <T as binrw::BinWrite>::Args<'b>: core::default::Default,
    {
        self.core_context.write(value);
    }

    /// Converts a [TypeExpression] to [TypeInstruction]s and appends them to the current buffer.
    pub fn append_compiled_type_expression(
        &mut self,
        type_expression: &TypeExpression,
    ) {
        let instructions = type_expression
            .to_instructions(self)
            .collect::<Vec<TypeInstruction>>();
        for instruction in instructions {
            append_type_instruction(self.cursor(), instruction);
        }
    }

    #[deprecated(note = "use write() instead")]
    pub fn append_instruction_code(&mut self, code: InstructionCode) {
        append_instruction_code(self.cursor(), code);
    }
}

impl<'ctx> SharedValueTrackingProvider<'ctx> for CompilationContext<'ctx> {
    fn shared_value_tracking<'a>(
        &'a self,
    ) -> Option<&'a RefCell<SharedValueTracking<'ctx>>> {
        Some(&self.core_context.shared_value_tracking)
    }
}
