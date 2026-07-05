use crate::pipeline_tests::setup::{Defined, Undefined};
use core::marker::PhantomData;
use datex_core::{
    disassembler::InstructionTree,
    global::protocol_structures::instructions::Instruction,
    values::value_container::ValueContainer,
};

#[cfg(feature = "ast")]
use datex_core::ast::expressions::DatexExpression;

#[cfg(feature = "parser")]
use datex_core::parser::lexer::Token;

enum SameAsInputOrCustom<T> {
    SameAsInput,
    Custom(T),
}

/// A builder for pipeline output assertion, allowing the user to specify
/// the expected states for all stage (parser, compiler, execution, ...)
pub struct PipelineOutput<
    'a,
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
> {
    #[cfg(feature = "parser")]
    tokens: Option<&'a [Token]>,
    #[cfg(feature = "ast")]
    ast: Option<DatexExpression>,
    instructions: Option<InstructionTree<Instruction>>,
    source_code: Option<SameAsInputOrCustom<String>>,
    value: Option<SameAsInputOrCustom<&'a ValueContainer>>,
    rust_value: Option<SameAsInputOrCustom<T>>,

    _marker: PhantomData<(
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Value,
        RustValue,
    )>,
}

impl<
    'a,
    T,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    Value,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        Defined,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        Undefined,
        Value,
        RustValue,
    >
{
    /// Assert that the decompiled source code of the output is the exact same as the input source code.
    pub fn source_code_same_as_input(
        self,
    ) -> PipelineOutput<
        'a,
        T,
        Defined,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        Defined,
        Value,
        RustValue,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: Some(SameAsInputOrCustom::SameAsInput),
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    T,
    InputSourceCode,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    SourceCode,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        Defined,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Undefined,
        RustValue,
    >
{
    /// Assert that the result DATEX value of the output is the exact same as the input DATEX value.
    pub fn datex_value_same_as_input(
        self,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        Defined,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Defined,
        RustValue,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: self.source_code,
            value: Some(SameAsInputOrCustom::SameAsInput),
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    Tokens,
    Ast,
    Instructions,
    SourceCode,
    Value,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        Defined,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Value,
        Undefined,
    >
{
    /// Assert that the result rust value of the output is the exact same as the input rust value.
    pub fn rust_value_same_as_input(
        self,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        Defined,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Value,
        Defined,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: self.source_code,
            value: self.value,
            rust_value: Some(SameAsInputOrCustom::SameAsInput),
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    Value,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        Undefined,
        Value,
        RustValue,
    >
{
    /// Define a decompiled source code that is expected as the output of the execution of
    /// the pipeline for the given inputs.
    pub fn source_code(
        self,
        source: impl Into<String>,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        Defined,
        Value,
        RustValue,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: Some(SameAsInputOrCustom::Custom(source.into())),
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    SourceCode,
    Value,
>
    PipelineOutput<
        'a,
        (),
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Value,
        Undefined,
    >
{
    /// Define a result rust value that is expected as the output of the execution of
    /// the pipeline for the given inputs.
    pub fn rust_value<T>(
        self,
        value: T,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Value,
        Defined,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: self.source_code,
            value: self.value,
            rust_value: Some(SameAsInputOrCustom::Custom(value)),
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    Instructions,
    SourceCode,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Undefined,
        RustValue,
    >
{
    /// Define a result DATEX value that is expected as the output of the execution of
    /// the pipeline for the given inputs.
    pub fn datex_value(
        self,
        value: impl Into<&'a ValueContainer>,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Instructions,
        SourceCode,
        Defined,
        RustValue,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: self.instructions,
            source_code: self.source_code,
            value: Some(SameAsInputOrCustom::Custom(value.into())),
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "parser")]
impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Ast,
    Instructions,
    SourceCode,
    Value,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Undefined,
        Ast,
        Instructions,
        SourceCode,
        Value,
        RustValue,
    >
{
    /// Define the tokens generated by the lexer that are expected for the given inputs.
    pub fn tokens(
        self,
        tokens: &'a [Token],
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Defined,
        Ast,
        Instructions,
        SourceCode,
        Value,
        RustValue,
    > {
        PipelineOutput {
            tokens: Some(tokens),
            ast: self.ast,
            instructions: self.instructions,
            source_code: self.source_code,
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "ast")]
impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Instructions,
    SourceCode,
    Value,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Undefined,
        Instructions,
        SourceCode,
        Value,
        RustValue,
    >
{
    /// Define the AST generated by the lexer that are expected for the given inputs.
    pub fn ast(
        self,
        ast: impl Into<DatexExpression>,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Defined,
        Instructions,
        SourceCode,
        Value,
        RustValue,
    > {
        PipelineOutput {
            tokens: self.tokens,
            ast: Some(ast.into()),
            instructions: self.instructions,
            source_code: self.source_code,
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

impl<
    'a,
    T,
    InputSourceCode,
    InputValue,
    InputRustValue,
    Tokens,
    Ast,
    SourceCode,
    Value,
    RustValue,
>
    PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Undefined,
        SourceCode,
        Value,
        RustValue,
    >
{
    /// Define the instructions generated by the lexer that are expected for the given inputs.
    pub fn instructions(
        self,
        ast: impl Into<InstructionTree<Instruction>>,
    ) -> PipelineOutput<
        'a,
        T,
        InputSourceCode,
        InputValue,
        InputRustValue,
        Tokens,
        Ast,
        Defined,
        SourceCode,
        Value,
        RustValue,
    > {
        PipelineOutput {
            #[cfg(feature = "parser")]
            tokens: self.tokens,
            #[cfg(feature = "ast")]
            ast: self.ast,
            instructions: Some(ast.into()),
            source_code: self.source_code,
            value: self.value,
            rust_value: self.rust_value,
            _marker: PhantomData,
        }
    }
}

pub fn output<'a, InputSourceCode, InputValue, InputRustValue>()
-> PipelineOutput<
    'a,
    (),
    InputSourceCode,
    InputValue,
    InputRustValue,
    Undefined,
    Undefined,
    Undefined,
    Undefined,
    Undefined,
    Undefined,
> {
    PipelineOutput {
        #[cfg(feature = "parser")]
        tokens: None,
        #[cfg(feature = "ast")]
        ast: None,
        instructions: None,
        source_code: None,
        value: None,
        rust_value: None,
        _marker: PhantomData,
    }
}
