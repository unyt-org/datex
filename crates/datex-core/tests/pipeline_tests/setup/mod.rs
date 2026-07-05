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
    todo!()
}