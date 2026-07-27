use core::ops::Range;

use crate::{
    ast::{
        expressions::{DatexExpression, DatexExpressionData},
        spanned::Spanned,
    },
    parser::{
        Parser, SpannedParserError,
        errors::ParserError,
        lexer::{SpannedToken, Token},
    },
};

use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_identifier_string(
        &mut self,
    ) -> Result<(String, Range<usize>), SpannedParserError> {
        let string = match &self.peek()?.token {
            // treat plain identifiers as text keys
            Token::Identifier(name) => Ok(name.clone()),

            // map reserved keywords to text keys
            // TODO #661: add more keywords as needed
            t @ Token::True
            | t @ Token::False
            | t @ Token::TypeDeclaration
            | t @ Token::Compile
            | t @ Token::If
            | t @ Token::Else
            | t @ Token::Is
            | t @ Token::Matches
            | t @ Token::And
            | t @ Token::Or => Ok(t.as_const_str().unwrap().into()),
            _ => Err(SpannedParserError {
                error: ParserError::UnexpectedToken {
                    expected: vec![Token::Identifier("".to_string())],
                    found: self.peek()?.token.clone(),
                },
                span: self.peek()?.span.clone(),
            }),
        }?;

        Ok((string, self.advance()?.span))
    }

    pub(crate) fn parse_key(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        Ok(match self.peek()?.token.clone() {
            // allow integers as keys
            Token::IntegerLiteral(value) => {
                self.parse_integer_literal(value)?
            }
            // allow string literals as keys
            Token::StringLiteral(value) => self.parse_string_literal(value)?,

            // allow parenthesized statements as keys
            Token::LeftParen => self.parse_parenthesized_statements()?,

            _ => self
                .parse_identifier_string()
                .map(|(string, span)| {
                    DatexExpressionData::Text(string.into()).with_span(span)
                })
                .map_err(|err| {
                    err.with_expected_tokens(vec![
                        Token::IntegerLiteral("".to_string()),
                        Token::StringLiteral("".to_string()),
                        Token::Identifier("".to_string()),
                        Token::LeftParen,
                    ])
                })?,
        })
    }
}
