use crate::{
    runtime::execution::execution_loop::state::RuntimeExecutionStack,
    values::value_container::ValueContainer,
};

use crate::prelude::*;
pub struct MemoryDump {
    pub stack: Vec<Option<ValueContainer>>,
}

#[cfg(feature = "decompiler")]
impl core::fmt::Display for MemoryDump {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (index, value) in self.stack.iter().enumerate() {
            match value {
                Some(vc) => {
                    let decompiled = crate::decompiler::decompile_value(
                        vc,
                        crate::decompiler::DecompileOptions::colorized(),
                    );
                    writeln!(f, "#{index}: {decompiled}")?
                }
                None => writeln!(f, "#{index}: <uninitialized>")?,
            }
        }
        if self.stack.is_empty() {
            writeln!(f, "<no slots allocated>")?;
        }
        Ok(())
    }
}

impl RuntimeExecutionStack {
    /// Returns a dump of the current stack values.
    pub fn stack_dump(&self) -> MemoryDump {
        MemoryDump {
            stack: self
                .values
                .clone()
        }
    }
}
