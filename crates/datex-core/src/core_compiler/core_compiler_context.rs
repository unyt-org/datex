use crate::core_compiler::ByteCursor;
use crate::core_compiler::shared_value_tracking::SharedValueTracking;

pub struct CoreCompilerContext<'a> {
    cursor: &'a mut ByteCursor,
    shared_value_tracking: &'a mut SharedValueTracking
}