use crate::{
    ast::{
        expressions::{DatexExpression, DatexExpressionData, WhileLoop},
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};

use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_while_loop(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        let start = self.expect(Token::While)?.span.start;

        let condition = Box::new(self.parse_parenthesized_statements()?);
        let body = Box::new(self.parse_parenthesized_statements()?);

        Ok(
            DatexExpressionData::WhileLoop(WhileLoop { condition, body })
                .with_span(start..self.get_current_source_position()),
        )
    }
}
