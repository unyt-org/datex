use crate::ast::expressions::DatexExpression;
use crate::decompiler::{DecompileOptions};
use converter::AstToSourceCodeConverter;
use crate::ast::spanned::Spanned;
use apply_syntax_highlighting::apply_syntax_highlighting;
use crate::preludes::derive::ValueContainer;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::prelude::*;
use core::fmt::Display;
use core::fmt::Formatter;

mod converter;
mod apply_syntax_highlighting;

/// Decompiles a single DATEX compatible value into a human-readable string representation.
pub fn value_to_source_code(
    value: &impl ToDatexExpressionData,
    options: DecompileOptions,
) -> String {
    let ast = value.to_datex_expression_data().with_default_span();
    ast_to_source_code(ast, options)
}

/// Decompiles a single DATEX compatible value into a human-readable string representation.
pub fn value_to_source_code_default(value: &impl ToDatexExpressionData) -> String {
    value_to_source_code(value, DecompileOptions::default())
}


pub fn ast_to_source_code(ast: DatexExpression, options: DecompileOptions) -> String {
    let colorized = options.formatting_options.colorized;
    let converter = AstToSourceCodeConverter::new(options.formatting_options);
    // convert AST to source code
    let source = converter.format(&ast);
    if colorized {
        apply_syntax_highlighting(source).unwrap()
    } else {
        source
    }
}