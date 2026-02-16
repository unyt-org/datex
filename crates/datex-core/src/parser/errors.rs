use crate::{
    ast::expressions::DatexExpression,
    global::operators::UnaryOperator,
    parser::lexer::Token,
    values::core_values::{
        endpoint::InvalidEndpointError, error::NumberParseError,
    },
};

use crate::prelude::*;
use core::ops::Range;
use crate::utils::maybe_action::ErrorCollector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    /// invalid token encountered during lexing
    InvalidToken,
    /// unexpected token encountered during parsing
    UnexpectedToken {
        expected: Vec<Token>,
        found: Token,
    },
    ExpectedMoreTokens,
    InvalidEndpointName {
        name: String,
        details: InvalidEndpointError,
    },
    InvalidAssignmentTarget,
    NumberParseError(NumberParseError),
    InvalidUnaryOperation {
        operator: UnaryOperator,
    },
    InvalidTypeVariantAccess,
    // used in internal parser logic to indicate a failed parse attempt that lead to a rollback
    CouldNotMatchGenericParams,
}

#[derive(Debug)]
pub struct DetailedParserErrorsWithAst {
    pub ast: DatexExpression, // TODO #657: rename to DatexAstNode
    pub errors: Vec<SpannedParserError>,
}

#[derive(Debug, Clone)]
pub struct SpannedParserError {
    pub error: ParserError,
    pub span: Range<usize>,
}

impl ErrorCollector<SpannedParserError> for Vec<SpannedParserError> {
    fn record_error(&mut self, error: SpannedParserError) {
        self.push(error);
    }
}
