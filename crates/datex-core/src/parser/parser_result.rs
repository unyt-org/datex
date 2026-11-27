use crate::{
    ast::expressions::NormalExpression, parser::errors::SpannedParserError,
};

use crate::prelude::*;
#[derive(Debug, Clone)]
pub struct ValidDatexParseResult {
    pub ast: NormalExpression,
}

#[derive(Debug, Clone)]
pub struct InvalidDatexParseResult {
    pub ast: NormalExpression,
    pub errors: Vec<SpannedParserError>,
}

#[derive(Debug, Clone)]
pub enum ParserResult {
    Valid(ValidDatexParseResult),
    Invalid(InvalidDatexParseResult),
}

impl ParserResult {
    pub fn is_valid(&self) -> bool {
        core::matches!(self, ParserResult::Valid { .. })
    }
    pub fn unwrap(self) -> ValidDatexParseResult {
        match self {
            ParserResult::Valid(result) => result,
            ParserResult::Invalid(InvalidDatexParseResult {
                errors, ..
            }) => {
                panic!("Parsing failed with errors: {:?}", errors)
            }
        }
    }
    pub fn errors(&self) -> Option<&Vec<SpannedParserError>> {
        match self {
            ParserResult::Valid { .. } => None,
            ParserResult::Invalid(InvalidDatexParseResult {
                errors, ..
            }) => Some(errors),
        }
    }

    pub fn ast(&self) -> &NormalExpression {
        match self {
            ParserResult::Valid(result) => &result.ast,
            ParserResult::Invalid(InvalidDatexParseResult { ast, .. }) => ast,
        }
    }

    pub fn into_ast_and_errors(
        self,
    ) -> (NormalExpression, Vec<SpannedParserError>) {
        match self {
            ParserResult::Valid(result) => (result.ast, vec![]),
            ParserResult::Invalid(InvalidDatexParseResult { ast, errors }) => {
                (ast, errors)
            }
        }
    }

    pub fn to_result(self) -> Result<NormalExpression, Vec<SpannedParserError>> {
        match self {
            ParserResult::Valid(result) => Ok(result.ast),
            ParserResult::Invalid(InvalidDatexParseResult {
                errors, ..
            }) => Err(errors),
        }
    }
}
