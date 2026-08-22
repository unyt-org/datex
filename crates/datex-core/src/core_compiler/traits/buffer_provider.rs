use crate::core_compiler::core_compilation_context::ByteCursor;
// TBD move out, if needed by more than just compiler
pub trait BufferProvider {
    fn cursor_mut(&mut self) -> &mut ByteCursor;
}
