use crate::{
    compiler::error::{
        CompilerError, SimpleOrDetailedCompilerError, SpannedCompilerError,
    },
    global::{
        dxb_block::DXBBlock,
        operators::assignment::AssignmentOperator,
        protocol_structures::{
            block_header::BlockHeader, encrypted_header::EncryptedHeader,
            routing_header::RoutingHeader,
        },
    },
};
use core::cell::RefCell;
use binrw::BinWrite;
use binrw::io::Write;
use crate::{
    ast::expressions::{
        BinaryOperation, ComparisonOperation, DatexExpression,
        DatexExpressionData, RemoteExecution, Slot, Statements, UnaryOperation,
        UnboundedStatement, UnboxAssignment, VariableAccess,
        VariableAssignment, VariableDeclaration, VariableKind,
    },
    compiler::{
        context::{CompilationContext},
        error::{
            DetailedCompilerErrorsWithMaybeRichAst,
            SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst,
        },
        metadata::CompileMetadata,
        scope::CompilationScope,
        type_compiler::compile_type_expression,
    },
    global::{instruction_codes::InstructionCode, slots::InternalSlot},
    libs::core::CoreLibPointerId,
    prelude::*,
};

use crate::{
    ast::resolved_variable::VariableId,
    core_compiler::value_compiler::{
        append_boolean, append_decimal, append_encoded_integer,
        append_endpoint, append_float_as_i16, append_float_as_i32,
        append_get_internal_ref, append_get_shared_ref, append_instruction_code,
        append_integer, append_key_string, append_text, append_typed_decimal,
    },
    parser::{Parser, ParserOptions},
    runtime::execution::context::ExecutionMode,
    shared_values::{
        pointer_address::PointerAddress,
        shared_container::SharedContainerMutability,
    },
    time::Instant,
    utils::buffers::{append_u8, append_u16, append_u32},
    values::{core_values::decimal::Decimal, value_container::ValueContainer},
};
use log::{debug, info};
use precompiler::{
    options::PrecompilerOptions,
    precompile_ast,
    precompiled_ast::{AstMetadata, RichAst, VariableMetadata},
};
use crate::ast::expressions::ValueAccessType;
use crate::core_compiler::value_compiler::{append_instruction, append_instruction_code_new, append_regular_instruction, append_shared_container, append_statements_preamble, append_value};
use crate::global::protocol_structures::injected_variable_type::{InjectedVariableType, LocalInjectedVariableType, SharedInjectedVariableType};
use crate::global::protocol_structures::instruction_data::{InstructionBlockData, ModifyStackValue, SetSharedContainerValue, StackIndex};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::shared_values::pointer::PointerReferenceMutability;

pub mod context;
pub mod error;
pub mod metadata;
pub mod scope;
pub mod type_compiler;

pub mod precompiler;
#[cfg(feature = "std")]
pub mod workspace;

#[derive(Clone, Default)]
pub struct CompileOptions {
    pub compile_scope: CompilationScope,
    pub parser_options: ParserOptions,
}

impl CompileOptions {
    pub fn new_with_scope(compile_scope: CompilationScope) -> Self {
        CompileOptions {
            compile_scope,
            parser_options: ParserOptions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StaticValueOrDXB {
    StaticValue(Option<ValueContainer>),
    DXB(Vec<u8>),
}

impl From<Vec<u8>> for StaticValueOrDXB {
    fn from(dxb: Vec<u8>) -> Self {
        StaticValueOrDXB::DXB(dxb)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum VariableModel {
    /// A variable that is declared once and never reassigned afterward
    /// e.g. `const a = 42;`
    Constant,
    /// A variable that can be reassigned by updating the slot value
    /// e.g. `var a = 42; a = 69;`
    VariableSlot,
}

impl From<VariableRepresentation> for VariableModel {
    fn from(value: VariableRepresentation) -> Self {
        match value {
            VariableRepresentation::Constant => VariableModel::Constant,
            VariableRepresentation::VariableSlot => {
                VariableModel::VariableSlot
            }
        }
    }
}

impl VariableModel {
    /// Determines the variable model based on the variable kind and metadata.
    pub fn infer(
        variable_kind: VariableKind,
        variable_metadata: Option<VariableMetadata>,
        execution_mode: ExecutionMode,
    ) -> Self {
        // const variables are always constant
        if variable_kind == VariableKind::Const {
            VariableModel::Constant
        }
        // otherwise, we use VariableSlot (default for `var` variables)
        else {
            VariableModel::VariableSlot
        }
    }

    pub fn infer_from_ast_metadata_and_type(
        ast_metadata: &AstMetadata,
        variable_id: Option<VariableId>,
        variable_kind: VariableKind,
        execution_mode: ExecutionMode,
    ) -> Self {
        let variable_metadata =
            variable_id.and_then(|id| ast_metadata.variable_metadata(id));
        Self::infer(variable_kind, variable_metadata.cloned(), execution_mode)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum VariableRepresentation {
    Constant,
    VariableSlot,
}

/// Represents a variable in the DATEX script.
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub kind: VariableKind,
    pub index: StackIndex,
    pub representation: VariableRepresentation,
}

impl Variable {
    pub fn new_const(name: String, index: StackIndex) -> Self {
        Variable {
            name,
            kind: VariableKind::Const,
            index,
            representation: VariableRepresentation::Constant,
        }
    }

    pub fn new_variable_slot(
        name: String,
        kind: VariableKind,
        index: StackIndex,
    ) -> Self {
        Variable {
            name,
            kind,
            index,
            representation: VariableRepresentation::VariableSlot,
        }
    }
}

/// Compiles a DATEX script text into a single DXB block including routing and block headers.
/// This function is used to create a block that can be sent over the network.
pub fn compile_block(
    datex_script: &str,
) -> Result<Vec<u8>, SimpleOrDetailedCompilerError> {
    let (body, _) = compile_script(datex_script, CompileOptions::default())?;

    let routing_header = RoutingHeader::default();

    let block_header = BlockHeader::default();
    let encrypted_header = EncryptedHeader::default();

    let block =
        DXBBlock::new(routing_header, block_header, encrypted_header, body);

    let bytes = block.to_bytes();
    Ok(bytes)
}

/// Compiles a DATEX script text into a DXB body
pub fn compile_script(
    datex_script: &str,
    options: CompileOptions,
) -> Result<(Vec<u8>, CompilationScope), SpannedCompilerError> {
    compile_template(datex_script, &[], options)
}

/// Directly extracts a static value from a DATEX script as a `ValueContainer`.
/// This only works if the script does not contain any dynamic values or operations.
/// All JSON-files can be compiled to static values, but not all DATEX scripts.
pub fn extract_static_value_from_script(
    datex_script: &str,
) -> Result<Option<ValueContainer>, SpannedCompilerError> {
    let valid_parse_result = Parser::parse_with_default_options(datex_script)?;
    extract_static_value_from_ast(&valid_parse_result)
        .map(Some)
        .map_err(SpannedCompilerError::from)
}

/// Converts a DATEX script template text with inserted values into an AST with metadata
/// If the script does not contain any dynamic values or operations, the static result value is
/// directly returned instead of the AST.
pub fn compile_script_or_return_static_value(
    datex_script: &str,
    mut options: CompileOptions,
) -> Result<(StaticValueOrDXB, CompilationScope), SpannedCompilerError> {
    let ast = parse_datex_script_to_rich_ast_simple_error(
        datex_script,
        &mut options,
    )?;
    let mut compilation_context = CompilationContext::new(
        Vec::with_capacity(256),
        vec![],
        options.compile_scope.execution_mode,
    );
    // FIXME #480: no clone here
    let scope = compile_ast(ast.clone(), &mut compilation_context, options)?;
    if compilation_context.has_non_static_value {
        Ok((StaticValueOrDXB::DXB(compilation_context.into_buffer()), scope))
    } else {
        // try to extract static value from AST
        extract_static_value_from_ast(&ast.ast)
            .map(|value| (StaticValueOrDXB::StaticValue(Some(value)), scope))
            .map_err(SpannedCompilerError::from)
    }
}

/// Ensure that the root ast node is a statements node
/// Returns if the initial ast was terminated
fn ensure_statements(
    ast: &mut DatexExpression,
    unbounded_section: Option<UnboundedStatement>,
) -> bool {
    if let DatexExpressionData::Statements(Statements {
        is_terminated,
        unbounded,
        ..
    }) = &mut ast.data
    {
        *unbounded = unbounded_section;
        *is_terminated
    } else {
        // wrap in statements
        let original_ast = ast.clone();
        ast.data = DatexExpressionData::Statements(Statements {
            statements: vec![original_ast],
            is_terminated: false,
            unbounded: unbounded_section,
        });
        false
    }
}

/// Parses and precompiles a DATEX script template text with inserted values into an AST with metadata
/// Only returns the first occurring error
pub fn parse_datex_script_to_rich_ast_simple_error(
    datex_script: &str,
    options: &mut CompileOptions,
) -> Result<RichAst, SpannedCompilerError> {
    // TODO #481: do this (somewhere else)
    // // shortcut if datex_script is "?" - call compile_value_container directly
    // if datex_script == "?" {
    //     if inserted_values.len() != 1 {
    //         return Err(CompilerError::InvalidPlaceholderCount);
    //     }
    //     let result =
    //         compile_value_container(inserted_values[0]).map(StaticValueOrAst::from)?;
    //     return Ok((result, options.compile_scope));
    // }
    let parse_start = Instant::now();
    let mut valid_parse_result =
        Parser::parse(datex_script, options.parser_options.clone())?;

    // make sure to append a statements block for the first block in ExecutionMode::Unbounded
    let is_terminated = if let ExecutionMode::Unbounded { has_next } =
        options.compile_scope.execution_mode
    {
        ensure_statements(
            &mut valid_parse_result,
            Some(UnboundedStatement {
                is_first: !options.compile_scope.was_used,
                is_last: !has_next,
            }),
        )
    } else {
        matches!(
            valid_parse_result.data,
            DatexExpressionData::Statements(Statements {
                is_terminated: true,
                ..
            })
        )
    };
    debug!(" [parse took {} ms]", parse_start.elapsed().as_millis());
    let precompile_start = Instant::now();
    let res = precompile_to_rich_ast(
        valid_parse_result,
        &mut options.compile_scope,
        PrecompilerOptions {
            detailed_errors: false,
        },
    )
    .map_err(|e| match e {
        SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Simple(e) => e,
        _ => unreachable!(), // because detailed_errors: false
    })
    .inspect(|ast| {
        // store information about termination (last semicolon) in metadata
        ast.metadata.borrow_mut().is_terminated = is_terminated;
    });
    debug!(
        " [precompile took {} ms]",
        precompile_start.elapsed().as_millis()
    );
    res
}

/// Parses and precompiles a DATEX script template text with inserted values into an AST with metadata
/// Returns all occurring errors and the AST if one or more errors occur.
pub fn parse_datex_script_to_rich_ast_detailed_errors(
    datex_script: &str,
    options: &mut CompileOptions,
) -> Result<RichAst, DetailedCompilerErrorsWithMaybeRichAst> {
    let (ast, parser_errors) =
        Parser::parse_collecting_with_default_options(datex_script)
            .into_ast_and_errors();
    precompile_to_rich_ast(
        ast,
        &mut options.compile_scope,
        PrecompilerOptions {
            detailed_errors: true,
        },
    )
    .map_err(|e| match e {
        SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Detailed(
            mut e,
        ) => {
            // append parser errors to detailed errors
            e.errors.errors.extend(
                parser_errors.into_iter().map(SpannedCompilerError::from),
            );
            e.into()
        }
        _ => unreachable!(), // because detailed_errors: true
    })
}

/// Compiles a DATEX script template text with inserted values into a DXB body
pub fn compile_template(
    datex_script: &str,
    inserted_values: &[Option<ValueContainer>],
    mut options: CompileOptions,
) -> Result<(Vec<u8>, CompilationScope), SpannedCompilerError> {
    let ast = parse_datex_script_to_rich_ast_simple_error(
        datex_script,
        &mut options,
    )?;
    let mut compilation_context = CompilationContext::new(
        Vec::with_capacity(256),
        // TODO #482: no clone here
        inserted_values.to_vec(),
        options.compile_scope.execution_mode,
    );
    let compile_start = Instant::now();
    let res = compile_ast(ast, &mut compilation_context, options)
        .map(|scope| (compilation_context.into_buffer(), scope))
        .map_err(SpannedCompilerError::from);
    debug!(
        " [compile_ast took {} ms]",
        compile_start.elapsed().as_millis()
    );
    res
}

/// Compiles a precompiled DATEX AST, returning the compilation context and scope
fn compile_ast(
    ast: RichAst,
    compilation_context: &mut CompilationContext,
    options: CompileOptions,
) -> Result<CompilationScope, CompilerError> {
    info!("ast {:#?}", ast.metadata.borrow());
    let compilation_scope =
        compile_rich_ast(compilation_context, ast, options.compile_scope)?;
    Ok(compilation_scope)
}

/// Tries to extract a static value from a DATEX expression AST.
/// If the expression is not a static value (e.g., contains a placeholder or dynamic operation),
/// it returns an error.
fn extract_static_value_from_ast(
    ast: &DatexExpression,
) -> Result<ValueContainer, CompilerError> {
    if let DatexExpressionData::Placeholder(_) = ast.data {
        return Err(CompilerError::NonStaticValue);
    }
    ValueContainer::try_from(&ast.data)
        .map_err(|_| CompilerError::NonStaticValue)
}

/// Macro for compiling a DATEX script template text with inserted values into a DXB body,
/// behaves like the format! macro.
/// Example:
/// ```
/// use datex_core::compile;
/// compile!("4 + ?", 42);
/// compile!("? + ?", 1, 2);
#[macro_export]
macro_rules! compile {
    ($fmt:literal $(, $arg:expr )* $(,)?) => {
        {
            let script: &str = $fmt.into();
            let values: &[Option<$crate::values::value_container::ValueContainer>] = &[$(Some($arg.into())),*];

            $crate::compiler::compile_template(&script, values, $crate::compiler::CompileOptions::default())
        }
    }
}

/// Precompiles a DATEX expression AST into an AST with metadata.
fn precompile_to_rich_ast(
    valid_parse_result: DatexExpression,
    scope: &mut CompilationScope,
    precompiler_options: PrecompilerOptions,
) -> Result<RichAst, SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst> {
    // if static execution mode and scope already used, return error
    if scope.execution_mode == ExecutionMode::Static && scope.was_used {
        return Err(
            SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Simple(
                SpannedCompilerError::from(
                    CompilerError::OnceScopeUsedMultipleTimes,
                ),
            ),
        );
    }

    // set was_used to true
    scope.was_used = true;

    let rich_ast = if let Some(precompiler_data) = &scope.precompiler_data {
        // precompile the AST, adding metadata for variables etc.
        precompile_ast(
            valid_parse_result,
            &mut precompiler_data.precompiler_scope_stack.borrow_mut(),
            precompiler_data.rich_ast.metadata.clone(),
            precompiler_options,
        )?
    } else {
        // if no precompiler data, just use the AST with default metadata
        RichAst::new_without_metadata(valid_parse_result)
    };

    Ok(rich_ast)
}

pub fn compile_rich_ast(
    compilation_context: &mut CompilationContext,
    rich_ast: RichAst,
    scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    compile_expression(
        compilation_context,
        rich_ast,
        CompileMetadata::outer(),
        scope,
    )
}

fn compile_expression(
    compilation_context: &mut CompilationContext,
    rich_ast: RichAst,
    meta: CompileMetadata,
    mut scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    let metadata = rich_ast.metadata;
    let ast = rich_ast.ast;

    let DatexExpression { data, span, ty } = ast;

    match data {
        DatexExpressionData::Integer(int) => {
            append_integer(compilation_context.cursor(), &int)?;
        }
        DatexExpressionData::TypedInteger(typed_int) => {
            append_encoded_integer(compilation_context.cursor(), &typed_int)?;
        }
        DatexExpressionData::Decimal(decimal) => match &decimal {
            Decimal::Finite(big_decimal) if big_decimal.is_integer() => {
                if let Some(int) = big_decimal.to_i16() {
                    append_float_as_i16(compilation_context.cursor(), int);
                } else if let Some(int) = big_decimal.to_i32() {
                    append_float_as_i32(compilation_context.cursor(), int);
                } else {
                    append_decimal(compilation_context.cursor(), &decimal)?;
                }
            }
            _ => {
                append_decimal(compilation_context.cursor(), &decimal)?;
            }
        },
        DatexExpressionData::TypedDecimal(typed_decimal) => {
            append_typed_decimal(
                &mut compilation_context.core_context,
                &typed_decimal,
            )?;
        }
        DatexExpressionData::Text(text) => {
            append_text(compilation_context.cursor(), &text)?;
        }
        DatexExpressionData::Boolean(boolean) => {
            append_boolean(compilation_context.cursor(), boolean)?;
        }
        DatexExpressionData::Endpoint(endpoint) => {
            append_endpoint(compilation_context.cursor(), &endpoint)?;
        }
        DatexExpressionData::Null => {
            append_regular_instruction(
                compilation_context.cursor(),
                RegularInstruction::Null,
            )?;
        }
        DatexExpressionData::List(list) => {
            match list.items.len() {
                0..=255 => {
                    compilation_context
                        .append_instruction_code(InstructionCode::SHORT_LIST);
                    append_u8(
                        compilation_context.cursor(),
                        list.items.len() as u8,
                    );
                }
                _ => {
                    compilation_context
                        .append_instruction_code(InstructionCode::LIST);
                    append_u32(
                        compilation_context.cursor(),
                        list.items.len() as u32, // FIXME #671: conversion from usize to u32
                    );
                }
            }
            for item in list.items {
                scope = compile_expression(
                    compilation_context,
                    RichAst::new(item, &metadata),
                    CompileMetadata::default(),
                    scope,
                )?;
            }
        }
        DatexExpressionData::Map(map) => {
            // TODO #434: Handle string keyed maps (structs)
            match map.entries.len() {
                0..=255 => {
                    compilation_context
                        .append_instruction_code(InstructionCode::SHORT_MAP);
                    append_u8(
                        compilation_context.cursor(),
                        map.entries.len() as u8,
                    );
                }
                _ => {
                    compilation_context
                        .append_instruction_code(InstructionCode::MAP);
                    append_u32(
                        compilation_context.cursor(),
                        map.entries.len() as u32, // FIXME #672: conversion from usize to u32
                    );
                }
            }
            for (key, value) in map.entries {
                scope = compile_key_value_entry(
                    compilation_context,
                    key,
                    value,
                    &metadata,
                    scope,
                )?;
            }
        }
        DatexExpressionData::Placeholder(placeholder_type) => {
            // FIXME #720
            let placeholder = compilation_context
                .inserted_values
                .get_mut(compilation_context.inserted_value_index)
                .expect("Placeholder index out of bounds");
            if let Some(value_container) = placeholder.take() {
                // TODO: validate in precompiler that the value container is actually a shared value

                match value_container {
                    ValueContainer::Local(value) => {
                        match placeholder_type {
                            ValueAccessType::SharedRef | ValueAccessType::SharedRefMut => return Err(CompilerError::SharedRefToNonSharedValue),
                            ValueAccessType::MoveOrCopy => {
                                append_value(
                                    compilation_context.core_context(),
                                    &value,
                                )?;
                            }
                            ValueAccessType::Clone => {
                                append_value(
                                    compilation_context.core_context(),
                                    &value,
                                )?;
                            }
                            ValueAccessType::Borrow => {
                                append_value(
                                    compilation_context.core_context(),
                                    &value,
                                )?;
                            }
                        }
                    }
                    ValueContainer::Shared(shared_container) => {
                        match placeholder_type {
                            ValueAccessType::SharedRefMut => {
                                let shared_container = shared_container
                                    .try_derive_mutable_reference()
                                    .map_err(|_| CompilerError::SharedMutRefToImmutableValue)?;
                                append_shared_container(
                                    compilation_context.core_context(),
                                    &shared_container,
                                    true,
                                )?;
                            }
                            ValueAccessType::SharedRef => {
                                append_shared_container(
                                    compilation_context.core_context(),
                                    &shared_container.derive_reference(),
                                    true,
                                )?;
                            },
                            ValueAccessType::MoveOrCopy => {
                                shared_container.assert_owned()
                                    .map_err(|_| CompilerError::InvalidConversionFromRefToOwnedValue)?;
                                append_shared_container(
                                    compilation_context.core_context(),
                                    &shared_container,
                                    true,
                                )?;
                            },
                            ValueAccessType::Clone => {
                                let cloned = shared_container.value_container();
                                match cloned {
                                    ValueContainer::Local(value) => {
                                        append_value(
                                            compilation_context.core_context(),
                                            &value,
                                        )?;
                                    }
                                    ValueContainer::Shared(shared_container) => {
                                        append_shared_container(
                                            compilation_context.core_context(),
                                            &shared_container,
                                            true,
                                        )?;
                                    }
                                }
                            },
                            ValueAccessType::Borrow => {
                                append_shared_container(
                                    compilation_context.core_context(),
                                    &shared_container.derive_reference(),
                                    true,
                                )?;
                            }
                        };
                    }
                }
            } else {
                // TODO
                // compilation_context
                //     .append_instruction_code(InstructionCode::CLONE_STACK_VALUE);
                // compilation_context.insert_virtual_slot_address(
                //     InjectedParentVariable::local(
                //         compilation_context.inserted_value_index as u32,
                //     ),
                // );
            }
            compilation_context.inserted_value_index += 1;
        }

        // statements
        DatexExpressionData::Statements(Statements {
            mut statements,
            is_terminated,
            unbounded,
        }) => {
            compilation_context.mark_has_non_static_value();
            // if single statement and not terminated, just compile the expression
            // (not for unbounded execution mode)
            if unbounded.is_none() && statements.len() == 1 && !is_terminated {
                scope = compile_expression(
                    compilation_context,
                    RichAst::new(statements.remove(0), &metadata),
                    CompileMetadata::default(),
                    scope,
                )?;
            } else {
                let is_outer_context = meta.is_outer_context();

                // if not outer context, new scope
                let mut child_scope = if is_outer_context {
                    scope
                } else {
                    scope.push()
                };

                if let Some(UnboundedStatement { is_first, .. }) = unbounded {
                    // if this is the first section of an unbounded statements block, mark as unbounded
                    if is_first {
                        compilation_context.append_instruction_code(
                            InstructionCode::UNBOUNDED_STATEMENTS,
                        );
                    }
                    // if not first, don't insert any instruction code
                }
                // otherwise, statements with fixed length
                else {
                    append_statements_preamble(
                        compilation_context.cursor(),
                        statements.len(),
                        is_terminated,
                    );
                }

                for statement in statements.into_iter() {
                    child_scope = compile_expression(
                        compilation_context,
                        RichAst::new(statement, &metadata),
                        CompileMetadata::default(),
                        child_scope,
                    )?;
                }
                if !meta.is_outer_context() {
                    // set parent scope
                    scope = child_scope
                        .pop()
                        .ok_or(CompilerError::ScopePopError)?;
                } else {
                    scope = child_scope;
                }

                // if this is the last section of an unbounded statements block, add closing instruction
                if let Some(UnboundedStatement { is_last: true, .. }) =
                    unbounded
                {
                    compilation_context.append_instruction_code(
                        InstructionCode::UNBOUNDED_STATEMENTS_END,
                    );
                    // append termination flag
                    append_u8(
                        compilation_context.cursor(),
                        if is_terminated { 1 } else { 0 },
                    );
                }
            }
        }

        // unary operations (negation, not, etc.)
        DatexExpressionData::UnaryOperation(UnaryOperation {
            operator,
            expression,
        }) => {
            compilation_context
                .append_instruction_code(InstructionCode::from(&operator));
            scope = compile_expression(
                compilation_context,
                RichAst::new(*expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // operations (add, subtract, multiply, divide, etc.)
        DatexExpressionData::BinaryOperation(BinaryOperation {
            operator,
            left,
            right,
            ..
        }) => {
            compilation_context.mark_has_non_static_value();
            // append binary code for operation if not already current binary operator
            compilation_context
                .append_instruction_code(InstructionCode::from(&operator));
            scope = compile_expression(
                compilation_context,
                RichAst::new(*left, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
            scope = compile_expression(
                compilation_context,
                RichAst::new(*right, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // comparisons (e.g., equal, not equal, greater than, etc.)
        DatexExpressionData::ComparisonOperation(ComparisonOperation {
            operator,
            left,
            right,
        }) => {
            compilation_context.mark_has_non_static_value();
            // append binary code for operation if not already current binary operator
            compilation_context
                .append_instruction_code(InstructionCode::from(&operator));
            scope = compile_expression(
                compilation_context,
                RichAst::new(*left, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
            scope = compile_expression(
                compilation_context,
                RichAst::new(*right, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // apply
        DatexExpressionData::Apply(apply) => {
            compilation_context.mark_has_non_static_value();

            // append apply instruction code
            let len = apply.arguments.len();
            match len {
                0 => {
                    compilation_context
                        .append_instruction_code(InstructionCode::APPLY_ZERO);
                }
                1 => {
                    compilation_context
                        .append_instruction_code(InstructionCode::APPLY_SINGLE);
                }
                // u16 argument count
                2..=65_535 => {
                    compilation_context
                        .append_instruction_code(InstructionCode::APPLY);
                    // add argument count
                    append_u16(
                        compilation_context.cursor(),
                        apply.arguments.len() as u16,
                    );
                }
                _ => return Err(CompilerError::TooManyApplyArguments),
            }

            // compile arguments
            for argument in apply.arguments.iter() {
                scope = compile_expression(
                    compilation_context,
                    RichAst::new(argument.clone(), &metadata),
                    CompileMetadata::default(),
                    scope,
                )?;
            }

            // compile function expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*apply.base, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        DatexExpressionData::PropertyAccess(property_access) => {
            compilation_context.mark_has_non_static_value();

            // depending on the key, handle different property accesses
            match &property_access.property.data {
                // simple text key if length fits in u8
                DatexExpressionData::Text(key) if key.len() <= 255 => {
                    compile_text_property_access(compilation_context, key)
                }
                // index access if integer fits in u32
                DatexExpressionData::Integer(index)
                    if let Some(index) = index.as_u32() =>
                {
                    compile_index_property_access(compilation_context, index)
                }
                _ => {
                    scope = compile_dynamic_property_access(
                        compilation_context,
                        &property_access.property,
                        scope,
                    )?;
                }
            }

            // compile base expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*property_access.base, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        DatexExpressionData::GenericInstantiation(_generic_instantiation) => {
            // NOTE: might already be handled in type compilation
            todo!("#674 Undescribed by author.")
        }

        DatexExpressionData::PropertyAssignment(property_assignment) => {
            compilation_context.mark_has_non_static_value();

            // depending on the key, handle different property assignments
            match &property_assignment.property.data {
                // simple text key if length fits in u8
                DatexExpressionData::Text(key) if key.len() <= 255 => {
                    compile_text_property_assignment(compilation_context, key)
                }
                // index access if integer fits in u32
                DatexExpressionData::Integer(index)
                    if let Some(index) = index.as_u32() =>
                {
                    compile_index_property_assignment(
                        compilation_context,
                        index,
                    )
                }
                _ => {
                    scope = compile_dynamic_property_assignment(
                        compilation_context,
                        &property_assignment.property,
                        scope,
                    )?;
                }
            }

            // compile assigned expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(
                    *property_assignment.assigned_expression,
                    &metadata,
                ),
                CompileMetadata::default(),
                scope,
            )?;

            // compile base expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*property_assignment.base, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // variables
        // declaration
        DatexExpressionData::VariableDeclaration(VariableDeclaration {
            id,
            name,
            kind,
            type_annotation: _,
            init_expression: value,
        }) => {
            compilation_context.mark_has_non_static_value();

            // push to stack
            let stack_index = scope.get_next_stack_index();
            compilation_context
                .append_instruction_code(InstructionCode::PUSH_TO_STACK);
            // compile expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*value, &metadata),
                CompileMetadata::default(),
                scope,
            )?;

            let variable_model =
                VariableModel::infer_from_ast_metadata_and_type(
                    &metadata.borrow(),
                    id,
                    kind,
                    compilation_context.execution_mode,
                );

            // create new variable depending on the model
            let variable = match variable_model {
                VariableModel::Constant => Variable::new_const(
                    name.clone(),
                    stack_index,
                ),
                VariableModel::VariableSlot => Variable::new_variable_slot(
                    name.clone(),
                    kind,
                    stack_index,
                ),
            };

            scope.register_variable_slot(variable);
        }

        DatexExpressionData::RequestSharedRef(shared_reference) => {
            compilation_context.mark_has_non_static_value();
            append_get_shared_ref(
                compilation_context.core_context(),
                &shared_reference.address,
                &shared_reference.mutability,
            )
        }

        // assignment
        DatexExpressionData::VariableAssignment(VariableAssignment {
            operator,
            name,
            expression,
            ..
        }) => {
            compilation_context.mark_has_non_static_value();
            // get variable slot address
            let (stack_index, kind) = scope
                .resolve_variable_name(&name, None)
                .map_err(|_| CompilerError::AssignmentToExternalVariable(name.clone()))?
                .ok_or_else(|| {
                    CompilerError::UndeclaredVariable(name.clone())
                })?;

            // TODO #484: check not needed, is already handled in precompiler - can we guarantee this?
            // if const, return error
            if kind == VariableKind::Const {
                return Err(CompilerError::AssignmentToConst(name.clone()));
            }

            match operator {
                None => {
                    // append binary code to load variable
                    info!(
                        "append variable - stack index: {stack_index:?}, name: {name}"
                    );
                    append_regular_instruction(
                        compilation_context.cursor(),
                        RegularInstruction::SetStackValue(stack_index),
                    )?;
                }
                Some(operator @ AssignmentOperator::AddAssign)
                | Some(operator @ AssignmentOperator::SubtractAssign) => {
                    // TODO #435: handle mut type
                    // // if immutable reference, return error
                    // if mut_type == Some(ReferenceMutability::Immutable) {
                    //     return Err(
                    //         CompilerError::AssignmentToImmutableReference(
                    //             name.clone(),
                    //         ),
                    //     );
                    // }
                    // // if immutable value, return error
                    // else if mut_type == None {
                    //     return Err(CompilerError::AssignmentToImmutableValue(
                    //         name.clone(),
                    //     ));
                    // }

                    append_regular_instruction(
                        compilation_context.cursor(),
                        RegularInstruction::ModifyStackValue(ModifyStackValue {
                            index: stack_index,
                            operator,
                        }),
                    )?;
                }
                op => core::todo!("#436 Handle assignment operator: {op:?}"),
            }

            // compile expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        DatexExpressionData::UnboxAssignment(UnboxAssignment {
            operator,
            unbox_expression,
            assigned_expression,
        }) => {
            compilation_context.mark_has_non_static_value();

            append_regular_instruction(
                compilation_context.cursor(),
                RegularInstruction::SetSharedContainerValue(SetSharedContainerValue {
                    operator,
                }),
            )?;

            // compile unbox expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*unbox_expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;

            // compile assigned expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*assigned_expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // variable access
        DatexExpressionData::VariableAccess(VariableAccess {
            name,
            access_type,
            ..
        }) => {
            compilation_context.mark_has_non_static_value();

            let slot_type = match access_type {
                ValueAccessType::SharedRefMut => InjectedVariableType::Shared(SharedInjectedVariableType::RefMut),
                ValueAccessType::SharedRef => InjectedVariableType::Shared(SharedInjectedVariableType::Ref),
                // TODO: map to local slot types depending on type
                ValueAccessType::MoveOrCopy => InjectedVariableType::Shared(SharedInjectedVariableType::Move),
                // TODO:
                ValueAccessType::Clone => InjectedVariableType::Local(LocalInjectedVariableType::Move),
                ValueAccessType::Borrow => InjectedVariableType::Shared(SharedInjectedVariableType::Move),
            };

            let slot_access = match access_type {
                ValueAccessType::SharedRefMut => InstructionCode::GET_STACK_VALUE_SHARED_REF_MUT,
                ValueAccessType::SharedRef => InstructionCode::GET_STACK_VALUE_SHARED_REF,
                ValueAccessType::MoveOrCopy => InstructionCode::TAKE_STACK_VALUE,
                ValueAccessType::Clone => InstructionCode::CLONE_STACK_VALUE,
                ValueAccessType::Borrow => InstructionCode::BORROW_STACK_VALUE,
            };

            // get variable slot address
            let (stack_index, ..) = scope
                .resolve_variable_name_with_slot_type(&name, slot_type)
                .ok_or_else(|| {
                    CompilerError::UndeclaredVariable(name.clone())
                })?;
            // append binary code to load variable
            compilation_context
                .append_instruction_code(slot_access);
            compilation_context.insert_stack_index(stack_index);
        }

        // remote execution
        DatexExpressionData::RemoteExecution(RemoteExecution {
            left: caller,
            right: script,
            injected_variable_count
        }) => {
            compilation_context.mark_has_non_static_value();

            // compile remote execution block
            let mut execution_block_ctx = CompilationContext::new(
                Vec::with_capacity(256),
                vec![],
                ExecutionMode::Static,
            );

            let stack_index_offset = StackIndex(injected_variable_count.unwrap()); // must be set by precompiler

            let external_scope = compile_rich_ast(
                &mut execution_block_ctx,
                RichAst::new(*script, &metadata),
                CompilationScope::new_with_external_parent_scope(scope, stack_index_offset),
            )?;
            // reset to current scope
            let external_parent_scope = external_scope
                .pop_external()
                .ok_or(CompilerError::ScopePopError)?;

            scope = *external_parent_scope.scope;

            // insert remote execution instruction
            append_regular_instruction(
                compilation_context.cursor(),
                RegularInstruction::RemoteExecution(InstructionBlockData {
                    // block size (len of compilation_context.buffer)
                    length: execution_block_ctx.cursor().get_ref().len() as u32,
                    injected_variable_count: external_parent_scope.injected_variables.len() as u32,
                    injected_variables: external_parent_scope.injected_variables,
                    body: execution_block_ctx.into_buffer(),
                })
            )?;

            // insert compiled caller expression
            scope = compile_expression(
                compilation_context,
                RichAst::new(*caller, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // named slot
        DatexExpressionData::Slot(Slot::Named(name)) => {
            match name.as_str() {
                "endpoint" => {
                    compilation_context.append_instruction_code(
                        InstructionCode::GET_INTERNAL_SLOT,
                    );
                    append_u32(
                        compilation_context.cursor(),
                        InternalSlot::ENDPOINT as u32,
                    );
                }
                "caller" => {
                    compilation_context.append_instruction_code(
                        InstructionCode::GET_INTERNAL_SLOT,
                    );
                    append_u32(
                        compilation_context.cursor(),
                        InternalSlot::CALLER as u32,
                    );
                }
                "env" => {
                    compilation_context.append_instruction_code(
                        InstructionCode::GET_INTERNAL_SLOT,
                    );
                    append_u32(
                        compilation_context.cursor(),
                        InternalSlot::ENV as u32,
                    );
                }
                "core" => append_get_internal_ref(
                    compilation_context.cursor(),
                    PointerAddress::from(CoreLibPointerId::Core)
                        .internal_bytes()
                        .unwrap(),
                ),
                _ => {
                    // invalid slot name
                    return Err(CompilerError::InvalidSlotName(name.clone()));
                }
            }
        }

        // refs
        DatexExpressionData::GetRef(create_ref) => {
            compilation_context.mark_has_non_static_value();
            // TODO #764: handle lifetimes, mutability, correctly (in precompiler)
            // TODO #765: handle move/clone
            scope = compile_expression(
                compilation_context,
                RichAst::new(*create_ref.expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        // shared refs
        DatexExpressionData::GetSharedRef(create_shared_ref) => {
            compilation_context.mark_has_non_static_value();
            compilation_context
                .append_instruction_code(match create_shared_ref.mutability {
                    PointerReferenceMutability::Immutable => {
                        InstructionCode::GET_SHARED_REF
                    }
                    PointerReferenceMutability::Mutable => {
                        InstructionCode::GET_SHARED_REF_MUT
                    }
                });
            scope = compile_expression(
                compilation_context,
                RichAst::new(*create_shared_ref.expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }
        // shared values
        DatexExpressionData::CreateShared(create_shared) => {
            compilation_context.mark_has_non_static_value();
            let mutability = create_shared.mutability;

            compilation_context.append_instruction_code(match mutability {
                SharedContainerMutability::Immutable => {
                    InstructionCode::CREATE_SHARED
                }
                SharedContainerMutability::Mutable => {
                    InstructionCode::CREATE_SHARED_MUT
                }
            });
            scope = compile_expression(
                compilation_context,
                RichAst::new(*create_shared.expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        DatexExpressionData::TypeExpression(type_expression) => {
            compilation_context
                .append_instruction_code(InstructionCode::TYPE_EXPRESSION);
            scope = compile_type_expression(
                compilation_context,
                &type_expression,
                &metadata,
                scope,
            )?;
        }
        DatexExpressionData::Range(range_dec) => {
            compilation_context.append_instruction_code(InstructionCode::RANGE);

            scope = compile_expression(
                compilation_context,
                RichAst::new(*range_dec.start, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
            scope = compile_expression(
                compilation_context,
                RichAst::new(*range_dec.end, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        DatexExpressionData::Unbox(unbox) => {
            compilation_context.mark_has_non_static_value();
            compilation_context.append_instruction_code(InstructionCode::UNBOX);
            scope = compile_expression(
                compilation_context,
                RichAst::new(*unbox.expression, &metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }

        data => {
            log::error!("Unhandled expression in compiler: {:?}", data);
            let ast = DatexExpression { data, span, ty };
            return Err(CompilerError::UnexpectedTerm(Box::new(ast)));
        }
    }

    Ok(scope)
}

fn compile_key_value_entry(
    compilation_context: &mut CompilationContext,
    key: DatexExpression,
    value: DatexExpression,
    metadata: &Rc<RefCell<AstMetadata>>,
    mut scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    match key.data {
        // text -> insert key string
        DatexExpressionData::Text(text) => {
            append_key_string(compilation_context.cursor(), &text);
        }
        // other -> insert key as dynamic
        _ => {
            compilation_context
                .append_instruction_code(InstructionCode::KEY_VALUE_DYNAMIC);
            scope = compile_expression(
                compilation_context,
                RichAst::new(key, metadata),
                CompileMetadata::default(),
                scope,
            )?;
        }
    };
    // insert value
    scope = compile_expression(
        compilation_context,
        RichAst::new(value, metadata),
        CompileMetadata::default(),
        scope,
    )?;
    Ok(scope)
}

fn compile_text_property_access(
    compilation_context: &mut CompilationContext,
    key: &str,
) {
    compilation_context
        .append_instruction_code(InstructionCode::GET_PROPERTY_TEXT);
    // append key length as u8
    append_u8(compilation_context.cursor(), key.len() as u8);
    // append key bytes
    compilation_context.cursor().write_all(key.as_bytes());
}

fn compile_text_property_assignment(
    compilation_context: &mut CompilationContext,
    key: &str,
) {
    compilation_context
        .append_instruction_code(InstructionCode::SET_PROPERTY_TEXT);
    // append key length as u8
    append_u8(compilation_context.cursor(), key.len() as u8);
    // append key bytes
    compilation_context.cursor().write_all(key.as_bytes());
}

fn compile_index_property_access(
    compilation_context: &mut CompilationContext,
    index: u32,
) {
    compilation_context
        .append_instruction_code(InstructionCode::GET_PROPERTY_INDEX);
    append_u32(compilation_context.cursor(), index);
}

fn compile_index_property_assignment(
    compilation_context: &mut CompilationContext,
    index: u32,
) {
    compilation_context
        .append_instruction_code(InstructionCode::SET_PROPERTY_INDEX);
    append_u32(compilation_context.cursor(), index);
}

fn compile_dynamic_property_access(
    compilation_context: &mut CompilationContext,
    key_expression: &DatexExpression,
    scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    compilation_context
        .append_instruction_code(InstructionCode::GET_PROPERTY_DYNAMIC);
    // compile key expression
    compile_expression(
        compilation_context,
        RichAst::new(
            key_expression.clone(),
            &Rc::new(RefCell::new(AstMetadata::default())),
        ),
        CompileMetadata::default(),
        scope,
    )
}

fn compile_dynamic_property_assignment(
    compilation_context: &mut CompilationContext,
    key_expression: &DatexExpression,
    scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    compilation_context
        .append_instruction_code(InstructionCode::SET_PROPERTY_DYNAMIC);
    // compile key expression
    compile_expression(
        compilation_context,
        RichAst::new(
            key_expression.clone(),
            &Rc::new(RefCell::new(AstMetadata::default())),
        ),
        CompileMetadata::default(),
        scope,
    )
}

#[cfg(test)]
pub mod tests {
    use super::{
        CompilationContext, CompileOptions, StaticValueOrDXB, compile_ast,
        compile_script, compile_script_or_return_static_value,
        compile_template, parse_datex_script_to_rich_ast_simple_error,
    };

    use crate::{assert_instructions_equal, assert_regular_instructions_equal, compiler::scope::CompilationScope, global::{
        instruction_codes::InstructionCode,
        type_instruction_codes::TypeInstructionCode,
    }, libs::core::CoreLibPointerId, runtime::execution::context::ExecutionMode};

    use crate::{
        compiler::error::CompilerError, prelude::*,
        shared_values::pointer_address::PointerAddress,
        values::core_values::integer::typed_integer::TypedInteger,
    };
    use alloc::format;
    use core::assert_matches;
    use log::*;
    use crate::disassembler::print_disassembled;
    use crate::global::protocol_structures::injected_variable_type::{InjectedVariableType, LocalInjectedVariableType, SharedInjectedVariableType};
    use crate::global::protocol_structures::instruction_data::{InstructionBlockData, InstructionBlockDataDebugFlat, InstructionBlockDataDebugTree, IntegerData, StackIndex, StatementsData, UInt8Data};
    use crate::global::protocol_structures::instructions::Instruction;
    use crate::global::protocol_structures::regular_instructions::RegularInstruction;
    use crate::values::core_values::integer::Integer;

    fn compile_and_log(datex_script: &str) -> Vec<u8> {
        let (result, _) =
            compile_script(datex_script, CompileOptions::default()).unwrap();
        info!(
            "{:?}",
            result
                .iter()
                .map(|x| InstructionCode::try_from(*x).map(|x| x.to_string()))
                .map(|x| x.unwrap_or_else(|_| "Unknown".to_string()))
                .collect::<Vec<_>>()
        );
        result
    }

    fn get_compilation_context(script: &str) -> CompilationContext {
        let mut options = CompileOptions::default();
        let ast =
            parse_datex_script_to_rich_ast_simple_error(script, &mut options)
                .unwrap();

        let mut compilation_context = CompilationContext::new(
            Vec::with_capacity(256),
            vec![],
            options.compile_scope.execution_mode,
        );
        compile_ast(ast, &mut compilation_context, options).unwrap();
        compilation_context
    }

    fn compile_datex_script_debug_unbounded(
        datex_script_parts: impl Iterator<Item = &'static str>,
    ) -> impl Iterator<Item = Vec<u8>> {
        let datex_script_parts = datex_script_parts.collect::<Vec<_>>();
        gen move {
            let mut compilation_scope =
                CompilationScope::new(ExecutionMode::unbounded());
            let len = datex_script_parts.len();
            for (index, script_part) in
                datex_script_parts.into_iter().enumerate()
            {
                // if last part, compile and return static value if possible
                if index == len - 1 {
                    compilation_scope.mark_as_last_execution();
                }
                let (dxb, new_compilation_scope) = compile_script(
                    script_part,
                    CompileOptions::new_with_scope(compilation_scope),
                )
                .unwrap();
                compilation_scope = new_compilation_scope;
                yield dxb;
            }
        }
    }

    fn assert_unbounded_input_matches_output(
        input: Vec<&'static str>,
        expected_output: Vec<Vec<u8>>,
    ) {
        let input = input.into_iter();
        let expected_output = expected_output.into_iter();
        for (result, expected) in
            compile_datex_script_debug_unbounded(input.into_iter())
                .zip(expected_output.into_iter())
        {
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn simple_multiplication() {
        let lhs: u8 = 1;
        let rhs: u8 = 2;
        let datex_script = format!("{lhs}u8 * {rhs}u8"); // 1 * 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::MULTIPLY.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );
    }

    #[test]
    fn simple_multiplication_close() {
        let lhs: u8 = 1;
        let rhs: u8 = 2;
        let datex_script = format!("{lhs}u8 * {rhs}u8;"); // 1 * 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                1,
                1, // terminated
                InstructionCode::MULTIPLY.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );
    }

    #[test]
    fn is_operator() {
        // TODO #151: compare refs
        let datex_script = "1u8 is 2u8".to_string();
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::IS.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2
            ]
        );

        let datex_script =
            "const a = shared mut 42u8; const b = 'mut 69u8; a is b".to_string(); // a is b
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                3,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::CREATE_SHARED_MUT.into(),
                InstructionCode::UINT_8.into(),
                42,
                // val b = 69;
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::GET_SHARED_REF_MUT.into(),
                InstructionCode::UINT_8.into(),
                69,
                // a is b
                InstructionCode::IS.into(),
                InstructionCode::TAKE_STACK_VALUE.into(),
                0,
                0,
                0,
                0, // slot address for a
                InstructionCode::TAKE_STACK_VALUE.into(),
                1,
                0,
                0,
                0, // slot address for b
            ]
        );
    }

    #[test]
    fn equality_operator() {
        let lhs: u8 = 1;
        let rhs: u8 = 2;
        let datex_script = format!("{lhs}u8 == {rhs}u8"); // 1 == 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::STRUCTURAL_EQUAL.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );

        let datex_script = format!("{lhs}u8 === {rhs}u8"); // 1 === 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::EQUAL.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );

        let datex_script = format!("{lhs}u8 != {rhs}u8"); // 1 != 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::NOT_STRUCTURAL_EQUAL.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );
        let datex_script = format!("{lhs}u8 !== {rhs}u8"); // 1 !== 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::NOT_EQUAL.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );
    }

    #[test]
    fn simple_addition() {
        let lhs: u8 = 1;
        let rhs: u8 = 2;
        let datex_script = format!("{lhs}u8 + {rhs}u8"); // 1 + 2
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs
            ]
        );

        let datex_script = format!("{lhs}u8 + {rhs}u8;"); // 1 + 2;
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                1,
                1, // terminated
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                lhs,
                InstructionCode::UINT_8.into(),
                rhs,
            ]
        );
    }

    #[test]
    fn multi_addition() {
        let op1: u8 = 1;
        let op2: u8 = 2;
        let op3: u8 = 3;
        let op4: u8 = 4;

        let datex_script = format!("{op1}u8 + {op2}u8 + {op3}u8 + {op4}u8"); // 1 + 2 + 3 + 4
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::ADD.into(),
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                op1,
                InstructionCode::UINT_8.into(),
                op2,
                InstructionCode::UINT_8.into(),
                op3,
                InstructionCode::UINT_8.into(),
                op4,
            ]
        );
    }

    #[test]
    fn mixed_calculation() {
        let op1: u8 = 1;
        let op2: u8 = 2;
        let op3: u8 = 3;
        let op4: u8 = 4;

        let datex_script = format!("{op1}u8 * {op2}u8 + {op3}u8 * {op4}u8"); // 1 + 2 + 3 + 4
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::MULTIPLY.into(),
                InstructionCode::UINT_8.into(),
                op1,
                InstructionCode::UINT_8.into(),
                op2,
                InstructionCode::MULTIPLY.into(),
                InstructionCode::UINT_8.into(),
                op3,
                InstructionCode::UINT_8.into(),
                op4,
            ]
        );
    }

    #[test]
    fn complex_addition() {
        let a: u8 = 1;
        let b: u8 = 2;
        let c: u8 = 3;
        let datex_script = format!("{a}u8 + ({b}u8 + {c}u8)"); // 1 + (2 + 3)
        let result = compile_and_log(&datex_script);

        assert_eq!(
            result,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                a,
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                b,
                InstructionCode::UINT_8.into(),
                c,
            ]
        );
    }

    #[test]
    fn complex_addition_and_subtraction() {
        let a: u8 = 1;
        let b: u8 = 2;
        let c: u8 = 3;
        let datex_script = format!("{a}u8 + ({b}u8 - {c}u8)"); // 1 + (2 - 3)
        let result = compile_and_log(&datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                a,
                InstructionCode::SUBTRACT.into(),
                InstructionCode::UINT_8.into(),
                b,
                InstructionCode::UINT_8.into(),
                c,
            ]
        );
    }

    #[test]
    fn integer_u8() {
        let val = 42;
        let datex_script = format!("{val}u8"); // 42
        let result = compile_and_log(&datex_script);
        assert_eq!(result, vec![InstructionCode::UINT_8.into(), val,]);
    }

    #[test]
    fn range_i64() {
        let start = 128i64;
        let end = 256i64;
        let datex_script = format!("{start}..{end}");
        let result = compile_and_log(&datex_script);

        assert_instructions_equal!(
            &result,
            [
                Instruction::Regular(RegularInstruction::Range),
                Instruction::Regular(RegularInstruction::Integer(IntegerData(Integer::new(start)))),
                Instruction::Regular(RegularInstruction::Integer(IntegerData(Integer::new(end))))
            ]
        )
    }

    // Test for decimal
    #[test]
    fn decimal() {
        let datex_script = "42.0";
        let result = compile_and_log(datex_script);
        let bytes = 42_i16.to_le_bytes();

        let mut expected: Vec<u8> =
            vec![InstructionCode::DECIMAL_AS_INT_16.into()];
        expected.extend(bytes);

        assert_eq!(result, expected);
    }

    /// Test for test that is less than 256 characters
    #[test]
    fn short_text() {
        let val = "unyt";
        let datex_script = format!("\"{val}\""); // "unyt"
        let result = compile_and_log(&datex_script);
        let mut expected: Vec<u8> =
            vec![InstructionCode::SHORT_TEXT.into(), val.len() as u8];
        expected.extend(val.bytes());
        assert_eq!(result, expected);
    }

    // Test empty list
    #[test]
    fn empty_list() {
        // TODO #437: support list constructor (apply on type)
        let datex_script = "[]";
        // const x = mut 42;
        let result = compile_and_log(datex_script);
        let expected: Vec<u8> = vec![
            InstructionCode::SHORT_LIST.into(),
            0, // length
        ];
        assert_eq!(result, expected);
    }

    // Test list with single element
    #[test]
    fn single_element_list() {
        // TODO #438: support list constructor (apply on type)
        let datex_script = "[42u8]";
        let result = compile_and_log(datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_LIST.into(),
                1, // length
                InstructionCode::UINT_8.into(),
                42,
            ]
        );
    }

    // Test list with multiple elements
    #[test]
    fn multi_element_list() {
        let datex_script = "[1u8, 2u8, 3u8]";
        let result = compile_and_log(datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_LIST.into(),
                3, // length
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2,
                InstructionCode::UINT_8.into(),
                3,
            ]
        );

        // trailing comma
        let datex_script = "[1u8, 2u8, 3u8,]";
        let result = compile_and_log(datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_LIST.into(),
                3, // length
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2,
                InstructionCode::UINT_8.into(),
                3,
            ]
        );
    }

    // Test list with expressions inside
    #[test]
    fn list_with_expressions() {
        let datex_script = "[1u8 + 2u8, 3u8 * 4u8]";
        let result = compile_and_log(datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_LIST.into(),
                2, // length
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2,
                InstructionCode::MULTIPLY.into(),
                InstructionCode::UINT_8.into(),
                3,
                InstructionCode::UINT_8.into(),
                4,
            ]
        );
    }

    // Nested lists
    #[test]
    fn nested_lists() {
        let datex_script = "[1u8, [2u8, 3u8], 4u8]";
        let result = compile_and_log(datex_script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_LIST.into(),
                3, // length
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::SHORT_LIST.into(),
                2, // length
                InstructionCode::UINT_8.into(),
                2,
                InstructionCode::UINT_8.into(),
                3,
                InstructionCode::UINT_8.into(),
                4,
            ]
        );
    }

    // map with text key
    #[test]
    fn map_with_text_key() {
        let datex_script = "{\"key\": 42u8}";
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SHORT_MAP.into(),
            1, // length
            InstructionCode::KEY_VALUE_SHORT_TEXT.into(),
            3, // length of "key"
            b'k',
            b'e',
            b'y',
            InstructionCode::UINT_8.into(),
            42,
        ];
        assert_eq!(result, expected);
    }

    // map with integer key
    #[test]
    fn map_integer_key() {
        let datex_script = "{(10u8): 42u8}";
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SHORT_MAP.into(),
            1, // length
            InstructionCode::KEY_VALUE_DYNAMIC.into(),
            InstructionCode::UINT_8.into(),
            10,
            InstructionCode::UINT_8.into(),
            42,
        ];
        assert_eq!(result, expected);
    }

    // map with long text key (>255 bytes)
    #[test]
    fn map_with_long_text_key() {
        let long_key = "a".repeat(300);
        let datex_script = format!("{{\"{long_key}\": 42u8}}");
        let result = compile_and_log(&datex_script);
        let mut expected: Vec<u8> = vec![
            InstructionCode::SHORT_MAP.into(),
            1, // length
            InstructionCode::KEY_VALUE_DYNAMIC.into(),
            InstructionCode::TEXT.into(),
        ];
        expected.extend((long_key.len() as u32).to_le_bytes());
        expected.extend(long_key.as_bytes());
        expected.extend(vec![InstructionCode::UINT_8.into(), 42]);
        assert_eq!(result, expected);
    }

    // map with dynamic key (expression)
    #[test]
    fn map_with_dynamic_key() {
        let datex_script = "{(1u8 + 2u8): 42u8}";
        let result = compile_and_log(datex_script);
        let expected = [
            InstructionCode::SHORT_MAP.into(),
            1, // length
            InstructionCode::KEY_VALUE_DYNAMIC.into(),
            InstructionCode::ADD.into(),
            InstructionCode::UINT_8.into(),
            1,
            InstructionCode::UINT_8.into(),
            2,
            InstructionCode::UINT_8.into(),
            42,
        ];
        assert_eq!(result, expected);
    }

    // map with multiple keys (text, integer, expression)
    #[test]
    fn map_with_multiple_keys() {
        let datex_script = "{key: 42u8, (4u8): 43u8, (1u8 + 2u8): 44u8}";
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SHORT_MAP.into(),
            3, // length
            InstructionCode::KEY_VALUE_SHORT_TEXT.into(),
            3, // length of "key"
            b'k',
            b'e',
            b'y',
            InstructionCode::UINT_8.into(),
            42,
            InstructionCode::KEY_VALUE_DYNAMIC.into(),
            InstructionCode::UINT_8.into(),
            4,
            InstructionCode::UINT_8.into(),
            43,
            InstructionCode::KEY_VALUE_DYNAMIC.into(),
            InstructionCode::ADD.into(),
            InstructionCode::UINT_8.into(),
            1,
            InstructionCode::UINT_8.into(),
            2,
            InstructionCode::UINT_8.into(),
            44,
        ];
        assert_eq!(result, expected);
    }

    // empty map
    #[test]
    fn empty_map() {
        let datex_script = "{}";
        let result = compile_and_log(datex_script);
        let expected: Vec<u8> = vec![
            InstructionCode::SHORT_MAP.into(),
            0, // length
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn allocate_slot() {
        let script = "const a = 42u8";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
            ]
        );
    }

    #[test]
    fn allocate_slot_with_value() {
        let script = "const a = 42u8; a + 1u8";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::ADD.into(),
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
                InstructionCode::UINT_8.into(),
                1,
            ]
        );
    }

    #[test]
    fn allocate_scoped_slots() {
        let script = "const a = 42u8; (const a = 43u8; a); a";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                3,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                43,
                InstructionCode::TAKE_STACK_VALUE.into(),
                1,
                0,
                0,
                0,
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
            ]
        );
    }

    #[test]
    fn allocate_scoped_slots_with_parent_variables() {
        let script =
            "const a = 42u8; const b = 41u8; (const a = 43u8; a; b); a";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                4,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                41,
                InstructionCode::SHORT_STATEMENTS.into(),
                3,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                43,
                InstructionCode::TAKE_STACK_VALUE.into(),
                2,
                0,
                0,
                0,
                InstructionCode::TAKE_STACK_VALUE.into(),
                1,
                0,
                0,
                0,
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
            ]
        );
    }

    #[test]
    fn allocate_shared() {
        let script = "const a = shared 42u8";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::CREATE_SHARED.into(),
                InstructionCode::UINT_8.into(),
                42,
            ]
        );
    }

    #[test]
    fn read_shared() {
        let script = "const a = shared 42u8; a";
        let result = compile_and_log(script);
        assert_eq!(
            result,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::CREATE_SHARED.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
            ]
        );
    }

    #[test]
    fn compile() {
        let result = compile_template(
            "? + ?",
            &[
                Some(TypedInteger::from(1u8).into()),
                Some(TypedInteger::from(2u8).into()),
            ],
            CompileOptions::default(),
        );
        assert_eq!(
            result.unwrap().0,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2
            ]
        );
    }

    #[test]
    fn compile_macro() {
        let a = TypedInteger::from(1u8);
        let result = compile!("?", a);
        assert_eq!(result.unwrap().0, vec![InstructionCode::UINT_8.into(), 1,]);
    }

    #[test]
    fn compile_macro_multi() {
        let result =
            compile!("? + ?", TypedInteger::from(1u8), TypedInteger::from(2u8));
        assert_eq!(
            result.unwrap().0,
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2
            ]
        );
    }

    // TODO #721:
    // #[cfg(feature = "std")]
    // fn get_json_test_string(file_path: &str) -> String {
    //     // read json from test file
    //     let file_path = format!("benches/json/{file_path}");
    //     let file_path = std::path::Path::new(&file_path);
    //     let file =
    //         std::fs::File::open(file_path).expect("Failed to open test.json");
    //     let mut reader = std::io::BufReader::new(file);
    //     let mut json_string = String::new();
    //     reader
    //         .read_to_string(&mut json_string)
    //         .expect("Failed to read test.json");
    //     json_string
    // }
    //
    // #[test]
    // #[cfg(feature = "std")]
    // fn json_to_dxb_large_file() {
    //     let json = get_json_test_string("test3.json");
    //     let _ = compile_script(&json, CompileOptions::default())
    //         .expect("Failed to parse JSON string");
    // }

    #[test]
    fn static_value_detection() {
        // non-static
        let script = "1 + 2";
        let compilation_scope = get_compilation_context(script);
        assert!(compilation_scope.has_non_static_value);

        let script = "1 2";
        let compilation_scope = get_compilation_context(script);
        assert!(compilation_scope.has_non_static_value);

        let script = "1;2";
        let compilation_scope = get_compilation_context(script);
        assert!(compilation_scope.has_non_static_value);

        let script = r#"{("x" + "y"): 1}"#;
        let compilation_scope = get_compilation_context(script);
        assert!(compilation_scope.has_non_static_value);

        // static
        let script = "1";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);

        let script = "[]";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);

        let script = "{}";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);

        let script = "[1,2,3]";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);

        let script = "{a: 2}";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);

        // because of unary - 42
        let script = "-42";
        let compilation_scope = get_compilation_context(script);
        assert!(!compilation_scope.has_non_static_value);
    }

    #[test]
    fn compile_auto_static_value_detection() {
        let script = "1u8";
        let (res, _) = compile_script_or_return_static_value(
            script,
            CompileOptions::default(),
        )
        .unwrap();
        assert_matches!(
            res,
            StaticValueOrDXB::StaticValue(val) if val == Some(TypedInteger::from(1u8).into())
        );

        let script = "1u8 + 2u8";
        let (res, _) = compile_script_or_return_static_value(
            script,
            CompileOptions::default(),
        )
        .unwrap();
        assert_matches!(
            res,
            StaticValueOrDXB::DXB(code) if code == vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2,
            ]
        );
    }

    #[test]
    fn nested_statements() {
        flexi_logger::init();
        let script = r#"
            var x = 1u8;
            (
                var y = 2u8;
                clone x;
                y;
            );
            var z = 3u8;
            x;
            z;
        "#;
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        print_disassembled(&res);
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {statements_count: 5, terminated: true}),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(1)),
                RegularInstruction::ShortStatements(StatementsData {statements_count: 3, terminated: true}),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(2)),
                RegularInstruction::CloneStackValue(StackIndex(0)),
                RegularInstruction::TakeStackValue(StackIndex(1)),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(3)),
                RegularInstruction::TakeStackValue(StackIndex(0)),
                RegularInstruction::TakeStackValue(StackIndex(1)),
            ]
        );
    }


    #[test]
    fn remote_execution() {
        let script = "42u8 :: 43u8";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block
                // block size (2 bytes)
                2,
                0,
                0,
                0,
                // injected slots (0)
                0,
                0,
                0,
                0,
                // literal value 43
                InstructionCode::UINT_8.into(),
                43,
                // --- end of block
                // caller (literal value 42 for test)
                InstructionCode::UINT_8.into(),
                42,
            ]
        );
    }

    #[test]
    fn remote_execution_expression() {
        let script = "42u8 :: 1u8 + 2u8";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block
                // block size (5 bytes)
                5,
                0,
                0,
                0,
                // injected slots (0)
                0,
                0,
                0,
                0,
                // expression: 1 + 2
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                1,
                InstructionCode::UINT_8.into(),
                2,
                // --- end of block
                // caller (literal value 42 for test)
                InstructionCode::UINT_8.into(),
                42,
            ]
        );
    }


    #[test]
    fn remote_execution_invalid_reassignment_of_external_variable() {
        flexi_logger::init();
        let script = "var x = 42u8; 1u8 :: (x = 43u8)";
        let result = compile_script(script, CompileOptions::default());
        assert!(result.is_err());
        assert_matches!(
            result.err().unwrap().error,
            CompilerError::AssignmentToExternalVariable(name) if name == "x"
        );
    }

    #[test]
    fn remote_execution_injected_const() {
        let script = "const x = 42u8; 1u8 :: x";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {statements_count: 2, terminated: false}),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(42)),
                RegularInstruction::_RemoteExecutionDebugFlat(InstructionBlockDataDebugFlat {
                    length: 5,
                    injected_variable_count: 1,
                    // FIXME should be local
                    injected_variables: vec![(StackIndex(0), InjectedVariableType::Shared(SharedInjectedVariableType::Move))],
                    body: vec![
                        Instruction::Regular(RegularInstruction::TakeStackValue(StackIndex(0))),
                    ]
                }),
                RegularInstruction::UInt8(UInt8Data(1)),
            ]
        );
    }

    #[test]
    fn remote_execution_injected_shared_move() {
        // var x only refers to a value, not a ref, but since it is transferred to a
        // remote context, its state is synced via a ref (VariableReference model)
        let script = "const x = shared 42u8; 1u8 :: x;";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {statements_count: 2, terminated: true}),
                RegularInstruction::PushToStack,
                RegularInstruction::CreateShared,
                RegularInstruction::UInt8(UInt8Data(42)),
                RegularInstruction::_RemoteExecutionDebugFlat(InstructionBlockDataDebugFlat {
                    length: 5,
                    injected_variable_count: 1,
                    injected_variables: vec![(StackIndex(0), InjectedVariableType::Shared(SharedInjectedVariableType::Move))],
                    body: vec![
                        Instruction::Regular(RegularInstruction::TakeStackValue(StackIndex(0))),
                    ],
                }),
                RegularInstruction::UInt8(UInt8Data(1)),
            ]
        )
    }

    #[test]
    fn remote_execution_injected_shared_ref() {
        let script = "const x = shared 42u8; 1u8 :: 'x";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {statements_count: 2, terminated: false}),
                RegularInstruction::PushToStack,
                RegularInstruction::CreateShared,
                RegularInstruction::UInt8(UInt8Data(42)),
                RegularInstruction::RemoteExecution(InstructionBlockData {
                    length: 5,
                    injected_variable_count: 1,
                    injected_variables: vec![(StackIndex(0), InjectedVariableType::Shared(SharedInjectedVariableType::Move))],
                    body: vec![
                        InstructionCode::GET_STACK_VALUE_SHARED_REF.into(),
                        // slot index as u32
                        0,
                        0,
                        0,
                        0,
                    ]
                }),
                RegularInstruction::UInt8(UInt8Data(1)),
            ]
        )
    }

    #[test]
    fn remote_execution_injected_consts() {
        let script = "const x = 42u8; const y = 69u8; 1u8 :: x + y";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {statements_count: 2, terminated: true}),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(42)),
                RegularInstruction::PushToStack,
                RegularInstruction::UInt8(UInt8Data(69)),
                RegularInstruction::RemoteExecution(InstructionBlockData {
                    length: 11,
                    injected_variable_count: 2,
                    injected_variables: vec![
                        // FIXME should be local
                        (StackIndex(0), InjectedVariableType::Shared(SharedInjectedVariableType::Move)),
                        (StackIndex(1), InjectedVariableType::Shared(SharedInjectedVariableType::Move)),
                    ],
                    body: vec![
                        InstructionCode::ADD.into(),
                        InstructionCode::TAKE_STACK_VALUE.into(),
                        // slot index as u32
                        0,
                        0,
                        0,
                        0,
                        InstructionCode::TAKE_STACK_VALUE.into(),
                        // slot index as u32
                        1,
                        0,
                        0,
                        0,
                    ],
                }),
                RegularInstruction::UInt8(UInt8Data(1)),
            ]
        );
    }

    #[test]
    fn remote_execution_shadow_const() {
        let script =
            "const x = 42u8; const y = 69u8; 1u8 :: (const x = 5u8; x + y)";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                3,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                69,
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block
                // block size (21 bytes)
                21,
                0,
                0,
                0,
                // injected slots (1)
                1,
                0,
                0,
                0,
                // slot 1 (y)
                1,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                // allocate slot for x
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                5,
                // expression: x + y
                InstructionCode::ADD.into(),
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                1,
                0,
                0,
                0,
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
                // --- end of block
                // caller (literal value 1 for test)
                InstructionCode::UINT_8.into(),
                1,
            ]
        );
    }

    #[test]
    fn remote_execution_nested() {
        let script = "const x = 42u8; (1u8 :: (2u8 :: x))";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();

        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block 1
                // block size (20 bytes)
                21,
                0,
                0,
                0,
                // injected slots (1)
                1,
                0,
                0,
                0,
                // slot 0
                0,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                // nested remote execution
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block 2
                // block size (5 bytes)
                5,
                0,
                0,
                0,
                // injected slots (1)
                1,
                0,
                0,
                0,
                // slot 0
                0,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
                // --- end of block 2
                // caller (literal value 2 for test)
                InstructionCode::UINT_8.into(),
                2,
                // -- end of block 1
                // caller (literal value 1 for test)
                InstructionCode::UINT_8.into(),
                1,
            ]
        );
    }

    #[test]
    fn remote_execution_nested2() {
        let script = "const x = 42u8; const y = 43u8; (1u8 :: (y :: x))";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();

        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                3,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                42,
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                43,
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block 1
                // block size (24 bytes)
                24,
                0,
                0,
                0,
                // injected slots (2)
                2,
                0,
                0,
                0,
                // slot 1
                0,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                // slot 0
                1,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                // nested remote execution
                InstructionCode::REMOTE_EXECUTION.into(),
                // --- start of block 2
                // block size (5 bytes)
                5,
                0,
                0,
                0,
                // injected slots (1)
                1,
                0,
                0,
                0,
                // slot 0
                0,
                0,
                0,
                0,
                // FIXME
                InjectedVariableType::Shared(SharedInjectedVariableType::Move).into(),
                InstructionCode::TAKE_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
                // --- end of block 2
                // caller (literal value 2 for test)
                InstructionCode::TAKE_STACK_VALUE.into(),
                1,
                0,
                0,
                0,
                // --- end of block 1
                // caller (literal value 1 for test)
                InstructionCode::UINT_8.into(),
                1,
            ]
        );
    }

    #[test]
    fn assignment_to_const() {
        let script = "const a = 42; a = 43";
        let result = compile_script(script, CompileOptions::default())
            .map_err(|e| e.error);
        assert_matches!(result, Err(CompilerError::AssignmentToConst { .. }));
    }

    #[test]
    fn assignment_to_const_mut() {
        let script = "const a = &mut 42; a = 43";
        let result = compile_script(script, CompileOptions::default())
            .map_err(|e| e.error);
        assert_matches!(result, Err(CompilerError::AssignmentToConst { .. }));
    }

    #[test]
    fn internal_assignment_to_const_mut() {
        let script = "const a = &mut 42; *a = 43";
        let result = compile_script(script, CompileOptions::default());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn addition_to_const_mut_ref() {
        let script = "const a = &mut 42; *a += 1;";
        let result = compile_script(script, CompileOptions::default());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn addition_to_const_variable() {
        let script = "const a = 42; a += 1";
        let result = compile_script(script, CompileOptions::default())
            .map_err(|e| e.error);
        assert_matches!(result, Err(CompilerError::AssignmentToConst { .. }));
    }

    #[test]
    fn internal_slot_endpoint() {
        let script = "#endpoint";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::GET_INTERNAL_SLOT.into(),
                // slot index as u32
                0,
                0xff,
                0xff,
                0xff
            ]
        );
    }

    #[test]
    fn internal_slot_caller() {
        let script = "#caller";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::GET_INTERNAL_SLOT.into(),
                // slot index as u32
                2,
                0xff,
                0xff,
                0xff
            ]
        );
    }

    // this is not a valid Datex script, just testing the compiler
    #[test]
    fn unbox() {
        let script = "*10u8";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::UNBOX.into(),
                InstructionCode::UINT_8.into(),
                // integer as u8
                10,
            ]
        );
    }

    #[test]
    fn unbox_slot() {
        let script = "const x = 10u8; *x";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS.into(),
                2,
                0, // not terminated
                InstructionCode::PUSH_TO_STACK.into(),
                InstructionCode::UINT_8.into(),
                10,
                InstructionCode::UNBOX.into(), // FIXME: should not be added for local values (precompiler)
                InstructionCode::BORROW_STACK_VALUE.into(),
                // slot index as u32
                0,
                0,
                0,
                0,
            ]
        );
    }

    #[test]
    fn type_literal_integer() {
        let script = "type<1>";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        assert_eq!(
            res,
            vec![
                InstructionCode::TYPE_EXPRESSION.into(),
                TypeInstructionCode::TYPE_LITERAL_INTEGER.into(),
                // slot index as u32
                2,
                1,
                0,
                0,
                0,
                1
            ]
        );
    }

    #[test]
    fn type_core_type_integer() {
        let script = "integer";
        let (res, _) =
            compile_script(script, CompileOptions::default()).unwrap();
        let mut instructions: Vec<u8> =
            vec![InstructionCode::GET_INTERNAL_SHARED_REF.into()];
        // pointer id
        instructions.append(
            &mut PointerAddress::from(CoreLibPointerId::Integer(None))
                .bytes()
                .to_vec(),
        );
        assert_eq!(res, instructions);
    }

    #[test]
    fn compile_continuous_terminated_script() {
        let input = vec!["1u8", "2u8", "3u8;"];
        let expected_output = vec![
            vec![
                InstructionCode::UNBOUNDED_STATEMENTS.into(),
                InstructionCode::UINT_8.into(),
                1,
            ],
            vec![InstructionCode::UINT_8.into(), 2],
            vec![
                InstructionCode::UINT_8.into(),
                3,
                InstructionCode::UNBOUNDED_STATEMENTS_END.into(),
                1, // terminated
            ],
        ];

        assert_unbounded_input_matches_output(input, expected_output);
    }

    #[test]
    fn compile_continuous_unterminated_script() {
        let input = vec!["1u8", "2u8 + 3u8", "3u8"];
        let expected_output = vec![
            vec![
                InstructionCode::UNBOUNDED_STATEMENTS.into(),
                InstructionCode::UINT_8.into(),
                1,
            ],
            vec![
                InstructionCode::ADD.into(),
                InstructionCode::UINT_8.into(),
                2,
                InstructionCode::UINT_8.into(),
                3,
            ],
            vec![
                InstructionCode::UINT_8.into(),
                3,
                InstructionCode::UNBOUNDED_STATEMENTS_END.into(),
                0, // unterminated
            ],
        ];

        assert_unbounded_input_matches_output(input, expected_output);
    }

    #[test]
    fn compile_continuous_complex() {
        let input = vec!["1u8", "integer"];
        let expected_output = vec![
            vec![
                InstructionCode::UNBOUNDED_STATEMENTS.into(),
                InstructionCode::UINT_8.into(),
                1,
            ],
            vec![
                InstructionCode::GET_INTERNAL_SHARED_REF.into(),
                // pointer id for integer
                100,
                0,
                0,
                InstructionCode::UNBOUNDED_STATEMENTS_END.into(),
                0, // unterminated
            ],
        ];

        assert_unbounded_input_matches_output(input, expected_output);
    }

    #[test]
    fn test_get_property_text() {
        let datex_script = r#""test".example"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::GET_PROPERTY_TEXT.into(),
            7, // length of "example"
            b'e',
            b'x',
            b'a',
            b'm',
            b'p',
            b'l',
            b'e',
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_property_text_quoted() {
        let datex_script = r#""test"."example""#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::GET_PROPERTY_TEXT.into(),
            7, // length of "example"
            b'e',
            b'x',
            b'a',
            b'm',
            b'p',
            b'l',
            b'e',
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_property_index() {
        let datex_script = r#""test".42"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::GET_PROPERTY_INDEX.into(),
            // u32 index 42
            42,
            0,
            0,
            0,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_property_dynamic() {
        let datex_script = r#""test".(1u8 + 2u8)"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::GET_PROPERTY_DYNAMIC.into(),
            // property expression: 1 + 2
            InstructionCode::ADD.into(),
            InstructionCode::UINT_8.into(),
            1,
            InstructionCode::UINT_8.into(),
            2,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_set_property_text() {
        let datex_script = r#""test".example = 42u8"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SET_PROPERTY_TEXT.into(),
            7, // length of "example"
            b'e',
            b'x',
            b'a',
            b'm',
            b'p',
            b'l',
            b'e',
            // value to set
            InstructionCode::UINT_8.into(),
            42,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_set_property_index() {
        let datex_script = r#""test".42 = 43u8"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SET_PROPERTY_INDEX.into(),
            // u32 index 42
            42,
            0,
            0,
            0,
            // value to set
            InstructionCode::UINT_8.into(),
            43,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_set_property_dynamic() {
        let datex_script = r#""test".(1u8 + 2u8) = 43u8"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SET_PROPERTY_DYNAMIC.into(),
            // property expression: 1 + 2
            InstructionCode::ADD.into(),
            InstructionCode::UINT_8.into(),
            1,
            InstructionCode::UINT_8.into(),
            2,
            // value to set
            InstructionCode::UINT_8.into(),
            43,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apply_no_arguments() {
        let datex_script = r#""test"()"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::APPLY_ZERO.into(),
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apply_one_argument() {
        let datex_script = r#""test" 42u8"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::APPLY_SINGLE.into(),
            // argument
            InstructionCode::UINT_8.into(),
            42,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_apply_multiple_arguments() {
        let datex_script = r#""test"(1u8, 2u8, 3u8)"#;
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::APPLY.into(),
            3, // number of arguments
            0,
            // argument 1
            InstructionCode::UINT_8.into(),
            1,
            // argument 2
            InstructionCode::UINT_8.into(),
            2,
            // argument 3
            InstructionCode::UINT_8.into(),
            3,
            // base value
            InstructionCode::SHORT_TEXT.into(),
            4, // length of "test"
            b't',
            b'e',
            b's',
            b't',
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn clone_local_value() {
        let datex_script = "var x = 10u8; var y = clone x; x";
        let result = compile_and_log(datex_script);
        let expected = vec![
            InstructionCode::SHORT_STATEMENTS.into(),
            3,
            0, // not terminated
            InstructionCode::PUSH_TO_STACK.into(),
            InstructionCode::UINT_8.into(),
            10,
            InstructionCode::PUSH_TO_STACK.into(),
            InstructionCode::CLONE_STACK_VALUE.into(),
            // slot index as u32
            0,
            0,
            0,
            0,
            InstructionCode::TAKE_STACK_VALUE.into(),
            // slot index as u32
            0,
            0,
            0,
            0,
        ];
        assert_eq!(result, expected);
    }
}
