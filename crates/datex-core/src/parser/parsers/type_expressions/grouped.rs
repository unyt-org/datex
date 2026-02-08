use crate::{
    ast::{
        spanned::Spanned,
        type_expressions::{TypeExpression, TypeExpressionData},
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};

impl Parser {
    pub(crate) fn parse_type_grouped(
        &mut self,
    ) -> Result<TypeExpression, SpannedParserError> {
        let start = self.expect(Token::LeftParen)?.span.start;

        // if right parenthesis follows immediately, it's a unit type
        if self.peek()?.token == Token::RightParen {
            let end = self.expect(Token::RightParen)?.span.end;
            return Ok(TypeExpressionData::Unit.with_span(start..end));
        }

        let inner_expression = self.parse_type_expression(0)?;

        let end = self.expect(Token::RightParen)?.span.end;
        Ok(inner_expression.data.with_span(start..end))
    }
}
