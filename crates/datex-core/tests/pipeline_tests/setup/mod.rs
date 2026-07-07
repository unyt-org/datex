mod input;
mod output;

pub struct Undefined;

pub struct Defined;

use datex_core::{
    ast::expressions::DatexExpression,
    compiler::{
        CompileOptions, compile_rich_ast, error::SpannedCompilerError,
        precompile_ast_simple_error, precompiler::precompiled_ast::RichAst,
    },
    core_compiler::core_compilation_context::DXBWithSharedValues,
    disassembler::{InstructionTree, disassemble_body},
    dxb_parser::body::DXBParserError,
    global::protocol_structures::instructions::{
        Instruction, NestedInstructionResolutionStrategy,
    },
    parser::{
        Parser, ParserOptions,
        errors::SpannedParserError,
        lexer,
        lexer::{InvalidToken, SpannedToken, Token},
    },
    runtime::{Runtime, execution::ExecutionError},
    values::value_container::ValueContainer,
};
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
    output: PipelineOutput<
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
    >,
) {
    // go through all stages of the pipeline, comparing outputs at each stage if provided
    let tokens = lexer_stage(source_code.clone(), output.tokens)
        .expect("Lexer stage failed");
    let ast = parser_stage(tokens, output.ast).expect("Parser stage failed");
    let rich_ast = precompiler_stage(ast, output.rich_ast)
        .expect("Precompiler stage failed");
    let dxb_with_shared_values = compiler_stage(rich_ast, output.instructions)
        .expect("Compiler stage failed");
    let _result = execution_stage(
        dxb_with_shared_values,
        output.datex_value.map(|v| v.no_input()),
        output.rust_value.map(|v| v.no_input()),
        output.source_code.map(|v| v.with_input(source_code)),
    )
    .expect("Execution stage failed");

    // TODO: additional end-to-end check: source code to final value
}

pub enum ValidationError {
    LexerError(Vec<InvalidToken>),
    ParserError(SpannedParserError),
    PrecompilerError(SpannedCompilerError),
    CompilerError(SpannedCompilerError),
    CompilerDisassemblerError(DXBParserError),
    ExecutionError(ExecutionError),
    OutputMismatch(String),
}

impl From<Vec<InvalidToken>> for ValidationError {
    fn from(errors: Vec<InvalidToken>) -> Self {
        ValidationError::LexerError(errors)
    }
}

impl From<SpannedParserError> for ValidationError {
    fn from(err: SpannedParserError) -> Self {
        ValidationError::ParserError(err)
    }
}

/// Lexer stage, converts source code to tokens
pub fn lexer_stage(
    source_code: String,
    compare_tokens: Option<&[Token]>,
) -> Result<Vec<SpannedToken>, ValidationError> {
    let (spanned_output_tokens, errors) =
        lexer::get_spanned_tokens_from_source(&source_code);
    if !errors.is_empty() {
        return Err(errors.into());
    }

    // compare output tokens with expected tokens if provided
    if let Some(expected_tokens) = compare_tokens {
        let output_tokens: Vec<Token> = spanned_output_tokens
            .iter()
            .map(|spanned| spanned.token.clone())
            .collect();

        if output_tokens != expected_tokens {
            return Err(ValidationError::OutputMismatch(format!(
                "Lexer output tokens do not match expected tokens.\nExpected: {:?}\nGot: {:?}",
                expected_tokens, output_tokens
            )));
        }
    }

    Ok(spanned_output_tokens)
}

/// Parser stage, converts tokens to AST
pub fn parser_stage(
    tokens: Vec<SpannedToken>,
    compare_ast: Option<DatexExpression>,
) -> Result<DatexExpression, ValidationError> {
    let ast = Parser::parse_tokens(tokens, vec![], ParserOptions::default())?;

    if let Some(expected_ast) = compare_ast
        && ast != expected_ast
    {
        return Err(ValidationError::OutputMismatch(format!(
            "Parser output AST does not match expected AST.\nExpected: {:?}\nGot: {:?}",
            expected_ast, ast
        )));
    }
    Ok(ast)
}

/// Precompiler stage, converts AST to Rich AST
pub fn precompiler_stage(
    ast: DatexExpression,
    compare_rich_ast: Option<RichAst>,
) -> Result<RichAst, ValidationError> {
    let rich_ast = precompile_ast_simple_error(
        ast,
        &mut CompileOptions::default(),
        Runtime::stub(),
    )
    .map_err(|e| ValidationError::PrecompilerError(e))?;

    if let Some(expected_rich_ast) = compare_rich_ast
        && rich_ast != expected_rich_ast
    {
        return Err(ValidationError::OutputMismatch(format!(
            "Precompiler output Rich AST does not match expected Rich AST.\nExpected: {:?}\nGot: {:?}",
            expected_rich_ast, rich_ast
        )));
    }

    Ok(rich_ast)
}

pub fn compiler_stage(
    rich_ast: RichAst,
    compare_instructions: Option<InstructionTree<Instruction>>,
) -> Result<DXBWithSharedValues, ValidationError> {
    let (dxb_with_shared_values, _scope) = compile_rich_ast(
        rich_ast,
        vec![],
        CompileOptions::default(),
        Runtime::stub(),
    )
    .map_err(|e| ValidationError::CompilerError(e))?;

    if let Some(expected_instructions) = compare_instructions {
        let (instructions, parser_error) = disassemble_body(
            &dxb_with_shared_values.dxb,
            NestedInstructionResolutionStrategy::ResolveNestedScopesTree,
        );
        if let Some(parser_error) = parser_error {
            return Err(ValidationError::CompilerDisassemblerError(
                parser_error,
            ));
        }

        if instructions.flatten() != expected_instructions.flatten() {
            return Err(ValidationError::OutputMismatch(format!(
                "Compiler output instructions do not match expected instructions.\nExpected: {:?}\nGot: {:?}",
                expected_instructions, instructions
            )));
        }
    }

    Ok(dxb_with_shared_values)
}

pub fn execution_stage<T>(
    dxb_with_shared_values: DXBWithSharedValues,
    compare_value: Option<Option<&ValueContainer>>,
    compare_rust_value: Option<T>,
    compare_source_code: Option<String>,
) -> Result<Option<ValueContainer>, ValidationError> {
    let runtime = Runtime::stub();
    let result = runtime
        .execute_dxb_sync(dxb_with_shared_values, None, true)
        .map_err(|e| ValidationError::ExecutionError(e))?;

    if let Some(expected_value) = compare_value {
        if &result != expected_value {
            return Err(ValidationError::OutputMismatch(format!(
                "Execution output value does not match expected value.\nExpected: {:?}\nGot: {:?}",
                expected_value, result
            )));
        }
    }

    // TODO: compare rust value and source code

    Ok(result)
}
