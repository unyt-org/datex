use crate::pipeline_tests::setup::{Defined, Undefined};
use core::marker::PhantomData;
use datex_core::{
    disassembler::InstructionTree,
    global::protocol_structures::instructions::Instruction,
    values::value_container::ValueContainer,
};

#[cfg(feature = "ast")]
use datex_core::ast::expressions::DatexExpression;
use datex_core::compiler::precompiler::precompiled_ast::RichAst;
#[cfg(feature = "parser")]
use datex_core::parser::lexer::Token;

pub enum SameAsInputOrCustom<T> {
    SameAsInput,
    Custom(T),
}

impl<T> SameAsInputOrCustom<T> {
    /// Collapses the enum into a value, panicking if the value is `SameAsInput`.
    pub fn no_input(&self) -> T {
        match self {
            SameAsInputOrCustom::SameAsInput => panic!("Expected input to be defined, but it was not."),
            SameAsInputOrCustom::Custom(value) => value.clone(),
        }
    }
    
    /// Collapses the enum into a value, returning either a custom value or the provided input value.
    pub fn with_input(self, input: T) -> T {
        match self {
            SameAsInputOrCustom::SameAsInput => input,
            SameAsInputOrCustom::Custom(value) => value,
        }
    }
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
    pub(crate) tokens: Option<&'a [Token]>,
    #[cfg(feature = "ast")]
    pub(crate) ast: Option<DatexExpression>,
    pub(crate) rich_ast: Option<RichAst>,
    pub(crate) instructions: Option<InstructionTree<Instruction>>,
    pub(crate) source_code: Option<SameAsInputOrCustom<String>>,
    pub(crate) datex_value: Option<SameAsInputOrCustom<Option<&'a ValueContainer>>>,
    pub(crate) rust_value: Option<SameAsInputOrCustom<T>>,

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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: Some(SameAsInputOrCustom::SameAsInput),
            datex_value: self.datex_value,
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: Some(SameAsInputOrCustom::SameAsInput),
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: self.datex_value,
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: Some(SameAsInputOrCustom::Custom(source.into())),
            datex_value: self.datex_value,
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: self.datex_value,
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
        value: Option<impl Into<&'a ValueContainer>>,
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: Some(SameAsInputOrCustom::Custom(value.into())),
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: self.datex_value,
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
            rich_ast: self.rich_ast,
            instructions: self.instructions,
            source_code: self.source_code,
            datex_value: self.datex_value,
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
            rich_ast: self.rich_ast,
            instructions: Some(ast.into()),
            source_code: self.source_code,
            datex_value: self.datex_value,
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
        rich_ast: None,
        instructions: None,
        source_code: None,
        datex_value: None,
        rust_value: None,
        _marker: PhantomData,
    }
}
