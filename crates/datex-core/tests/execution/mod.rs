use datex_core::{
    compile,
    compiler::{CompileOptions, compile_template},
    disassembler::{disassemble_body_to_string, options::DisassemblerOptions},
    runtime::{
        Runtime,
        execution::{
            ExecutionInput, ExecutionOptions, execute_dxb_sync,
            execution_input::ExecutionCallerMetadata,
        },
    },
    values::{core_values::list::List, value_container::ValueContainer},
};

pub mod local_values;
pub mod shared_values;

pub fn compile_and_execute(input: ValueContainer) -> ValueContainer {
    compile_and_execute_multiple(vec![input]).remove(0)
}

fn compile_and_execute_multiple(
    input: Vec<ValueContainer>,
) -> Vec<ValueContainer> {
    let runtime = Runtime::stub();
    let script = format!("[{}]", "?,".repeat(input.len()));

    let (dxb, _) = compile_template(
        &script,
        input.into_iter().map(Some).collect::<Vec<_>>(),
        CompileOptions::default(),
        runtime.clone(),
    )
    .unwrap();

    println!(
        "{}",
        disassemble_body_to_string(&dxb.dxb, DisassemblerOptions::default())
    );

    let result = execute_dxb_sync(ExecutionInput::new(
        dxb,
        ExecutionCallerMetadata::local_default(),
        ExecutionOptions { verbose: true },
        runtime,
    ))
    .unwrap()
    .unwrap();

    let list: List = result.try_as().expect("Failed to convert result to List");
    list.into()
}
