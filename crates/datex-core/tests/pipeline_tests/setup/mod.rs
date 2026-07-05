mod input;
mod output;

pub struct Undefined;

pub struct Defined;

pub use input::*;
pub use output::*;


pub fn validate_pipeline<
    InT,
    InSourceCode, 
    InValue, 
    InRustValue,
    OutT,
    OutputTokens,
    OutputAst,
    OutputInstructions,
    OutputSourceCode,
    OutputValue,
    OutputRustValue,
>(
    input: PipelineInput<InT, InSourceCode, InValue, InRustValue>,
    output: PipelineOutput<
        OutT,
        InSourceCode,
        InValue,
        InRustValue,
        OutputTokens,
        OutputAst,
        OutputInstructions,
        OutputSourceCode,
        OutputValue,
        OutputRustValue,
    >,
) {
    // test each input against all outputs
    if let Some(source_code) = input.source_code {
        validate_source_code_input(source_code, output); 
    }
}

pub fn validate_source_code_input<
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    SourceCode,
    Value,
    RustValue,
>(
    source_code: String,
    output: PipelineOutput<T, InputSourceCode, InputValue, InputRustValue, Tokens, Ast, Instructions, SourceCode, Value, RustValue>
) {
    // stage: parser
    // stage: compiler
    // stage execution
    
    // additional end-to-end check: source code to final value
    todo!()
}