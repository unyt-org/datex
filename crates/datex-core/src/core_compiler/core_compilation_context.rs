use core::cell::RefCell;

use crate::{
    core_compiler::{
        buffer_provider::BufferProvider,
        preamble::append_injected_values_preamble,
        shared_value_tracking::SharedValueTracking,
        to_instructions::{ToInstructions},
        type_compiler::append_type_instruction,
        value_compiler::{append_shared_container_from_preamble, append_value},
        value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
    shared_values::SharedContainer,
    types::r#type::Type,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use binrw::io::Cursor;
use crate::core_compiler::value_compiler::append_instruction;
use crate::instruction::Instruction;

pub type ByteCursor = Cursor<Vec<u8>>;

#[derive(Debug)]
pub struct DXBWithSharedValues {
    pub dxb: Vec<u8>,
    /// Shared values that can be accessed inside the execution
    pub shared_values: Vec<SharedContainer>,
}
impl DXBWithSharedValues {
    /// Create a new [DXBWithSharedValues] with the provided DXB bytecode and shared values.
    pub fn new(dxb: Vec<u8>, shared_values: Vec<SharedContainer>) -> Self {
        DXBWithSharedValues { dxb, shared_values }
    }

    /// Create a new [DXBWithSharedValues] with the provided DXB bytecode and shared values.
    /// Tries to upgrade referenced shared containers to owned containers if possible.
    ///
    /// # Safety
    /// The caller must ensure for all shared_values that no other [OwnedSharedContainer] for the same
    /// inner value exists if move_indicator of the [ReferencedSharedContainer] is set.
    pub unsafe fn new_with_upgraded_owned_containers(
        dxb: Vec<u8>,
        shared_values: Vec<SharedContainer>,
    ) -> Self {
        // Force convert referenced containers with move_indicator flag
        // to OwnedSharedContainer.
        let shared_values = shared_values
            .into_iter()
            .map(|shared_container| {
                unsafe { shared_container.try_upgrade_to_owned() }
                    .map(SharedContainer::Owned)
                    .unwrap_or_else(SharedContainer::Referenced)
            })
            .collect();
        DXBWithSharedValues { dxb, shared_values }
    }

    pub fn into_dxb(self) -> Vec<u8> {
        self.dxb
    }
}

#[derive(Debug, Clone)]
pub struct CompileInput<'a> {
    pub pointer_lookup: &'a PointerAvailabilityLookup,
    pub receivers: &'a [Endpoint],
}
impl<'a> CompileInput<'a> {
    pub fn new(
        pointer_lookup: &'a PointerAvailabilityLookup,
        receivers: &'a [Endpoint],
    ) -> Self {
        CompileInput {
            pointer_lookup,
            receivers,
        }
    }
}

/// # Safety
/// This function is unsafe because it creates a CompileInput with a static lifetime, which may lead to dangling references if used incorrectly. It should only be used in tests where the leaked memory is acceptable
pub(crate) unsafe fn default_compile_input<'a>() -> CompileInput<'a> {
    CompileInput {
        pointer_lookup: Box::leak(Box::new(
            PointerAvailabilityLookup::default(),
        )),
        receivers: Box::leak(Box::new([])),
    }
}

#[derive(Debug)]
pub struct CoreCompilationContext<'a> {
    pub cursor: ByteCursor,
    pub shared_value_tracking: RefCell<SharedValueTracking<'a>>,
    pub input: CompileInput<'a>,
}

/// # Safety
/// This function is unsafe because it creates a CoreCompilationContext
pub(crate) unsafe fn default_core_compilation_context<'a>()
-> CoreCompilationContext<'a> {
    CoreCompilationContext::new(Vec::new(), unsafe { default_compile_input() })
}

impl<'a> CoreCompilationContext<'a> {
    /// Create a new core compilation context with an initial byte input buffer
    pub fn new(
        buffer: Vec<u8>,
        input: CompileInput<'a>,
    ) -> CoreCompilationContext<'a> {
        CoreCompilationContext {
            cursor: Cursor::new(buffer),
            shared_value_tracking: RefCell::new(SharedValueTracking::new(
                input.pointer_lookup,
                input.receivers,
            )),
            input,
        }
    }

    /// Create a new core compilation context for the provided receiver endpoints
    pub fn new_for_endpoints(
        pointer_lookup: &'a PointerAvailabilityLookup,
        receivers: &'a [Endpoint],
    ) -> CoreCompilationContext<'a> {
        let input = CompileInput::new(pointer_lookup, receivers);
        CoreCompilationContext::new(vec![], input)
    }

    pub fn cursor(&self) -> &Cursor<Vec<u8>> {
        &self.cursor
    }

    /// Finalizes the compilation context by appending a preamble with the injected shared values,
    /// and returns the final byte buffer and the list of shared values that were moved or referenced during compilation
    pub fn into_dxb_with_shared_values(self) -> DXBWithSharedValues {
        let tracked_values = self.shared_value_tracking;
        let inner = tracked_values.into_inner().into_tracked_values();
        let (combined_buffer, top_level_values) =
            append_injected_values_preamble(inner, self.cursor.into_inner());
        // SAFETY: it is assumed that the tracked values from the shared value tracking were
        // moved inside the compilation and are no longer accessible from the outside
        unsafe {
            DXBWithSharedValues::new_with_upgraded_owned_containers(
                combined_buffer,
                top_level_values,
            )
        }
    }
}

impl BufferProvider for CoreCompilationContext<'_> {
    fn cursor_mut(&mut self) -> &mut ByteCursor {
        &mut self.cursor
    }
}

impl<'ctx> ValueVisitor<'ctx> for CoreCompilationContext<'ctx> {
    /// Appends a value container.
    /// For local values, the value is just serialized
    /// For shared values, the container is registered in the context shared value tracking
    fn visit_value_container(&mut self, value_container: &ValueContainer) {
        // TODO can we pass value container by reference?
        match value_container {
            ValueContainer::Local(value) => append_value(self, value),
            ValueContainer::Shared(reference) => {
                append_shared_container_from_preamble(self, reference);
            }
        }
    }

    fn visit_type(&mut self, ty: &Type) {
        let instructions =
            ty.to_instructions(self).collect::<Vec<Instruction>>();

        for instruction in instructions {
            append_instruction(self.cursor_mut(), instruction);
        }
    }

    fn shared_value_tracking(
        &self,
    ) -> Option<&RefCell<SharedValueTracking<'ctx>>> {
        Some(&self.shared_value_tracking)
    }
}