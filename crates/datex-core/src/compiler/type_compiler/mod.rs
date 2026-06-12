use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    compiler::{
        context::CompilationContext, error::CompilerError,
        precompiler::precompiled_ast::AstMetadata, scope::CompilationScope,
    },
    core_compiler::{
        type_compiler::append_type_instruction,
        value_compiler::append_big_integer,
    },
    global::protocol_structures::{
        instructions::Instruction, type_instructions::TypeInstruction,
    },
    utils::buffers::{append_u8, append_u32},
    values::core_values::integer::Integer,
};

use crate::prelude::*;
use binrw::io::Write;
use core::cell::RefCell;
pub mod type_expression_to_instruction;
