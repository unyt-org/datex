use crate::{
    ast::{
        expressions::{DatexExpression, DatexExpressionData, Set},
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
    prelude::*,
};

use crate::prelude::*;
use alloc::vec;

impl Parser {
    pub fn parse_set(&mut self) -> Result<DatexExpression, SpannedParserError> {
        let start = self.expect(Token::SetOpen)?.span.start;
        let mut items = Vec::new();

        while self.peek()?.token != Token::SetClose {
            let maybe_expression = self.parse_expression(0);
            let expression = self.recover_on_error(
                maybe_expression,
                &[Token::Comma, Token::SetClose],
            )?;
            items.push(expression);

            if self.peek()?.token == Token::Comma {
                self.advance()?;
            }
        }

        let end = self.expect(Token::SetClose)?.span.end;
        Ok(DatexExpressionData::Set(Set { items }).with_span(start..end))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{DatexExpressionData, Set},
            spanned::Spanned,
        },
        parser::{errors::ParserError, lexer::Token, tests::parse},
        prelude::*,
    };
    use core::assert_matches;

    #[test]
    fn parse_empty_set() {
        let expr = parse("<||>");
        assert_eq!(expr.data, DatexExpressionData::Set(Set { items: vec![] }));
    }

    #[test]
    fn parse_simple_set() {
        let expr = parse("<|1|>");
        assert_eq!(
            expr.data,
            DatexExpressionData::Set(Set {
                items: vec![
                    DatexExpressionData::Integer(1.into()).with_default_span(),
                ]
            })
        );
    }

    #[test]
    fn parse_set_with_multiple_elements() {
        let expr = parse("<|1, 2, 3|>");
        assert_eq!(
            expr.data,
            DatexExpressionData::Set(Set {
                items: vec![
                    DatexExpressionData::Integer(1.into()).with_default_span(),
                    DatexExpressionData::Integer(2.into()).with_default_span(),
                    DatexExpressionData::Integer(3.into()).with_default_span(),
                ]
            })
        );
    }
}
