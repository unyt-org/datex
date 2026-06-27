use datex_core::{
    compile,
    compiler::{CompileOptions, compile_template, scope::CompilationScope},
    disassembler::{options::DisassemblerOptions},
    runtime::{
        Runtime,
        execution::{
            ExecutionInput, ExecutionOptions, execute_dxb_sync,
            execution_input::ExecutionCallerMetadata,
        },
    },
    shared_values::{
        ReferenceMutability, ReferencedSharedContainer, SharedContainer,
    },
    values::{
        core_values::{endpoint::Endpoint, list::List},
        value_container::ValueContainer,
    },
};
use datex_core::disassembler::print_disassembled_with_options;

pub mod local_values;
pub mod shared_values;

/// Compiles and executes a script that takes a single value input.
/// Compiles local values and owned shared values as "?", and shared references as "'?" or "'mut ?" depending on their mutability.
pub fn compile_and_execute(input: ValueContainer) -> ValueContainer {
    compile_and_execute_multiple(vec![input]).remove(0)
}

/// Compiles and executes a script that takes multiple value inputs as a list.
/// Compiles local values and owned shared values as "?", and shared references as "'?" or "'mut ?" depending on their mutability.
fn compile_and_execute_multiple(
    input: Vec<ValueContainer>,
) -> Vec<ValueContainer> {
    let runtime = Runtime::stub();
    let script = format!(
        "[{}]",
        input
            .iter()
            .map(|value| {
                match value {
                    ValueContainer::Shared(SharedContainer::Referenced(
                        reference,
                    )) if reference.reference_mutability()
                        == ReferenceMutability::Immutable =>
                    {
                        "'?"
                    }
                    ValueContainer::Shared(SharedContainer::Referenced(
                        reference,
                    )) if reference.reference_mutability()
                        == ReferenceMutability::Mutable =>
                    {
                        "'mut ?"
                    }
                    _ => "?",
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    // FIXME can we make this cleaner, by using one of the other 10000 helper functions doing the same shit

    let (dxb, _) = compile_template(
        &script,
        input.into_iter().map(Some).collect::<Vec<_>>(),
        CompileOptions::new(CompilationScope::default(), vec![Endpoint::LOCAL]),
        runtime.clone(),
    )
    .unwrap();

    print_disassembled_with_options(&dxb.dxb, DisassemblerOptions::default());

    let result = execute_dxb_sync(ExecutionInput::new(
        dxb,
        ExecutionCallerMetadata::local_default(),
        ExecutionOptions { verbose: true },
        runtime,
    ))
    .unwrap()
    .unwrap();

    let list: List = result
        .try_into_value()
        .expect("Failed to convert result to List");
    list.into()
}
