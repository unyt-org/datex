use crate::{
    ast::{
        expressions::{DatexExpression, DatexExpressionData, Loop},
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};

use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_for_loop(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        let start = self.expect(Token::For)?.span.start;

        let condition = Some(Box::new(self.parse_parenthesized_statements()?));
        let body = Box::new(self.parse_parenthesized_statements()?);

        Ok(DatexExpressionData::Loop(Loop {
            condition,
            body,
        })
        .with_span(start..self.get_current_source_position()))
    }
}
