#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteExecutionOptions {
    pub is_end_of_context: bool,
    pub mark_as_debug: bool,
}

impl RemoteExecutionOptions {
    pub fn new_with_end_of_context(is_end_of_context: bool) -> Self {
        RemoteExecutionOptions {
            is_end_of_context,
            mark_as_debug: false,
        }
    }
}
