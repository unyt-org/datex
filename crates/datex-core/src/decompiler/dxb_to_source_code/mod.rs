pub mod ast_from_bytecode;

use crate::{
    decompiler::{DecompileOptions, ast_to_source_code::ast_to_source_code},
    dxb_parser::body::DXBParserError,
    prelude::*,
};
use ast_from_bytecode::ast_from_bytecode;

/// Decompiles a DXB bytecode body into a human-readable string representation.
pub fn dxb_to_source_code(
    dxb_body: &[u8],
    options: DecompileOptions,
) -> Result<String, DXBParserError> {
    let ast = ast_from_bytecode(dxb_body)?;
    Ok(ast_to_source_code(ast, options))
}
