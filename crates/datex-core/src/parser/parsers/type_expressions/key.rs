use crate::{
    ast::{
        spanned::Spanned,
        type_expressions::{TypeExpression, TypeExpressionData},
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};

use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_type_key(
        &mut self,
    ) -> Result<TypeExpression, SpannedParserError> {
        Ok(match self.peek()?.token.clone() {
            // allow grouped expressions as keys
            Token::LeftParen => self.parse_type_grouped()?,

            // allow integers as keys
            Token::IntegerLiteral(value) => {
                self.parse_type_integer_literal(value)?
            }
            // allow string literals as keys
            Token::StringLiteral(value) => {
                self.parse_type_string_literal(value)?
            }

            _ => self
                .parse_identifier_string()
                .map(|(string, span)| {
                    TypeExpressionData::Text(string.into()).with_span(span)
                })
                .map_err(|err| {
                    err.with_expected_tokens(vec![
                        Token::LeftParen,
                        Token::IntegerLiteral("".to_string()),
                        Token::StringLiteral("".to_string()),
                    ])
                })?,
        })
    }
}
