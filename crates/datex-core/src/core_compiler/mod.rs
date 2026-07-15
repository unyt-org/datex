//! This module contains the core compiler logic for DATEX, including [value_compiler] and [type_compiler] needed for compilation.
pub mod core_compilation_context;
pub mod injected_values;
mod preamble;
pub mod shared_value_tracking;
mod traits;
pub mod type_compiler;
pub mod update_compiler;
pub mod value_compiler;

pub use traits::*;
use crate::core_compiler::core_compilation_context::{CoreCompilationContext, DXBWithSharedValues};
use crate::runtime::pointer_availability_lookup::PointerAvailabilityLookup;
use crate::values::core_values::endpoint::Endpoint;

/// Compiles a [DXBWithSharedValues] with the given compile handler callback function.
pub fn core_compile(
    pointer_availability_lookup: &PointerAvailabilityLookup,
    endpoints: &[Endpoint],
    compile_handler: impl FnOnce(&mut CoreCompilationContext)
) -> DXBWithSharedValues {
    let mut core_context = CoreCompilationContext::new_for_endpoints(
        pointer_availability_lookup,
        endpoints
    );
    
    compile_handler(&mut core_context);

    core_context.into_dxb_with_shared_values()
}