//! This module contains the core compiler logic for DATEX, including [value_compiler] and [type_compiler] needed for compilation.
pub mod core_compilation_context;
pub mod injected_values;
mod preamble;
pub mod shared_value_tracking;
pub mod traits;
pub mod type_compiler;
pub mod update_compiler;
pub mod value_compiler;
use crate::{
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::{
            CoreCompilationContext, DXBWithSharedValues,
        },
        value_compiler::{append_instruction, append_value_container},
    },
    instruction::{
        Instruction, regular_instruction::RegularInstruction,
        type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
    values::{
        core_values::endpoint::Endpoint, value::Value,
        value_container::ValueContainer,
    },
};
pub use traits::*;

pub enum InstructionInput {
    Instruction(Instruction),
    ValueContainer(ValueContainer),
}

impl InstructionInput {
    pub fn compile(self, ctx: &mut CoreCompilationContext) {
        match self {
            InstructionInput::Instruction(instruction) => {
                append_instruction(ctx.cursor_mut(), instruction);
            }
            InstructionInput::ValueContainer(value_container) => {
                append_value_container(ctx, &value_container);
            }
        }
    }
}

impl From<Instruction> for InstructionInput {
    fn from(instruction: Instruction) -> Self {
        InstructionInput::Instruction(instruction)
    }
}
impl From<RegularInstruction> for InstructionInput {
    fn from(instruction: RegularInstruction) -> Self {
        InstructionInput::Instruction(Instruction::Regular(instruction))
    }
}
impl From<TypeInstruction> for InstructionInput {
    fn from(instruction: TypeInstruction) -> Self {
        InstructionInput::Instruction(Instruction::Type(instruction))
    }
}
impl From<ValueContainer> for InstructionInput {
    fn from(value_container: ValueContainer) -> Self {
        InstructionInput::ValueContainer(value_container)
    }
}
impl From<Value> for InstructionInput {
    fn from(value: Value) -> Self {
        InstructionInput::ValueContainer(ValueContainer::Local(value))
    }
}

/// Compiles a [DXBWithSharedValues] with the given compile handler callback function.
pub fn core_compile(
    pointer_availability_lookup: &PointerAvailabilityLookup,
    endpoints: &[Endpoint],
    instructions_input: Vec<InstructionInput>,
) -> DXBWithSharedValues {
    let mut core_context = CoreCompilationContext::new_for_endpoints(
        pointer_availability_lookup,
        endpoints,
    );

    for instruction_input in instructions_input {
        instruction_input.compile(&mut core_context);
    }

    core_context.into_dxb_with_shared_values()
}
