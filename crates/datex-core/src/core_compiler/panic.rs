//! Defines the `Panic` struct used for representing panic messages
//! used by the core compiler.
use crate::{
    core_compiler::{
        core_compilation_context::{CompileInput, CoreCompilationContext},
        value_compiler::ApplyWithArgs,
    },
    instruction::regular_instruction::RegularInstruction,
    libs::core::{core_lib_id::CoreLibId, value_id::CoreLibValueId},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::value_container::ValueContainer,
};
use core::fmt::Display;

#[derive(Debug)]
pub struct Panic {
    message: String,
}

impl Panic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Converts the panic message into a standalone
    /// response block dxb
    pub fn to_bytes(
        self,
        compile_input: CompileInput,
        cache: &mut SharedReferencesCache,
    ) -> Vec<u8> {
        CoreCompilationContext::new(compile_input)
            .compile_to_dxb_with_shared_values(
                &ApplyWithArgs::new(
                    RegularInstruction::get_core_lib_value(
                        CoreLibId::Value(CoreLibValueId::Panic).into(),
                    ),
                    vec![ValueContainer::Local(self.message.into())],
                ),
                cache,
            )
            .dxb
    }
}

impl Display for Panic {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Panic: {}", self.message)
    }
}
