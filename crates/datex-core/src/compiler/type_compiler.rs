use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    compiler::{
        context::CompilationContext, error::CompilerError,
        precompiler::precompiled_ast::AstMetadata, scope::CompilationScope,
    },
    core_compiler::value_compiler::append_big_integer,
    global::type_instruction_codes::TypeInstructionCode,
    utils::buffers::{append_u8, append_u32},
    values::core_values::integer::Integer,
};

use crate::prelude::*;
use core::cell::RefCell;
use binrw::io::Write;

/// Compilation functions for type expressions.
impl CompilationContext {
    pub fn append_type_instruction_code(&mut self, code: TypeInstructionCode) {
        append_u8(&mut self.cursor, code as u8);
    }

    // TODO #452: Handle other types

    pub fn insert_type_literal_integer(&mut self, integer: &Integer) {
        self.append_type_instruction_code(
            TypeInstructionCode::TYPE_LITERAL_INTEGER,
        );
        append_big_integer(&mut self.cursor, integer);
    }

    pub fn insert_type_literal_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len();

        if len < 256 {
            self.append_type_instruction_code(
                TypeInstructionCode::TYPE_LITERAL_SHORT_TEXT,
            );
            append_u8(&mut self.cursor, len as u8);
        } else {
            self.append_type_instruction_code(
                TypeInstructionCode::TYPE_LITERAL_TEXT,
            );
            append_u32(&mut self.cursor, len as u32);
        }

        self.cursor.write_all(bytes);
    }
}

pub fn compile_type_expression(
    ctx: &mut CompilationContext,
    expr: &TypeExpression,
    _ast_metadata: &Rc<RefCell<AstMetadata>>,
    mut scope: CompilationScope,
) -> Result<CompilationScope, CompilerError> {
    match &expr.data {
        TypeExpressionData::Integer(integer) => {
            ctx.insert_type_literal_integer(integer);
        }
        TypeExpressionData::Text(text) => {
            ctx.insert_type_literal_text(text);
        }
        TypeExpressionData::Range(range) => {
            ctx.append_type_instruction_code(TypeInstructionCode::TYPE_RANGE);
            scope =
                compile_type_expression(ctx, &range.end, _ast_metadata, scope)?;
            scope = compile_type_expression(
                ctx,
                &range.start,
                _ast_metadata,
                scope,
            )?;
        }
        _ => core::todo!("#453 Undescribed by author."),
    }
    Ok(scope)
}
