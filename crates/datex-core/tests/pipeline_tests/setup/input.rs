use crate::pipeline_tests::setup::{
    Defined, Undefined, output, output::PipelineOutput,
};
use core::marker::PhantomData;
use datex_core::values::value_container::ValueContainer;

/// A builder for pipeline input,
/// allowing the user to specify the source code, the Datex value, and the Rust value.
pub struct PipelineInput<T, SourceCode, Value, RustValue> {
    source_code: Option<String>,
    value: Option<ValueContainer>,
    rust_value: Option<T>,

    _marker: PhantomData<(SourceCode, Value, RustValue)>,
}

impl<T, Value, RustValue> PipelineInput<T, Undefined, Value, RustValue> {
    /// Define a DATEX source code string input for which the pipeline will be tested.
    pub fn source_code(
        self,
        source: impl Into<String>,
    ) -> PipelineInput<T, Defined, Value, RustValue> {
        PipelineInput {
            source_code: Some(source.into()),
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<T, SourceCode, RustValue>
    PipelineInput<T, SourceCode, Undefined, RustValue>
{
    /// Define a DATEX value ([ValueContainer]) input for which the pipeline will be tested.
    pub fn datex_value(
        self,
        value: impl Into<ValueContainer>,
    ) -> PipelineInput<T, SourceCode, Defined, RustValue> {
        PipelineInput {
            source_code: self.source_code,
            value: Some(value.into()),
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<SourceCode, Value> PipelineInput<(), SourceCode, Value, Undefined> {
    /// Define a Rust value input for which the pipeline will be tested.
    pub fn rust_value<T>(
        self,
        value: T,
    ) -> PipelineInput<T, SourceCode, Value, Defined> {
        PipelineInput {
            source_code: self.source_code,
            value: self.value,
            rust_value: Some(value.into()),
            _marker: PhantomData,
        }
    }
}

impl<T, SourceCode, Value, RustValue>
    PipelineInput<T, SourceCode, Value, RustValue>
{
    /// Assert that every given input produces all expected outputs, as defined by the given [PipelineOutput].
    pub fn expect<
        OutT,
        OutputTokens,
        OutputAst,
        OutputInstructions,
        OutputSourceCode,
        OutputValue,
        OutputRustValue,
    >(
        self,
        output: PipelineOutput<
            OutT,
            SourceCode,
            Value,
            RustValue,
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
}

pub fn input() -> PipelineInput<(), Undefined, Undefined, Undefined> {
    PipelineInput {
        source_code: None,
        value: None,
        rust_value: None,
        _marker: PhantomData,
    }
}
