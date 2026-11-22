use crate::{
    ast::{
        expressions::{Conditional, DataExpression, DataExpressionData},
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};

impl Parser {
    pub(crate) fn parse_if_else(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        let start = self.expect(Token::If)?.span.start;

        let condition = self.parse_parenthesized_statements()?;
        let then_branch = self.parse_parenthesized_statements()?;

        let else_branch = if let Ok(token) = self.peek()
            && token.token == Token::Else
        {
            self.advance()?;
            Some(self.parse_parenthesized_statements_or_if_else()?)
        } else {
            None
        };

        Ok(DataExpressionData::Conditional(Conditional {
            condition: (condition),
            then_branch: (then_branch),
            else_branch,
        })
        .with_span(start..self.get_current_source_position()))
    }

    fn parse_parenthesized_statements_or_if_else(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        if let Ok(token) = self.peek()
            && token.token == Token::If
        {
            self.parse_if_else()
        } else {
            self.parse_parenthesized_statements()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::{
                Conditional, DataExpressionData, expression::Expression,
            },
            spanned::Spanned,
        },
        parser::tests::parse,
    };
    #[test]
    fn parse_single_if() {
        let expr = parse("if (true) (42)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Conditional(Conditional {
                condition: DataExpressionData::Boolean(true.into())
                    .with_default_span(),
                then_branch: DataExpressionData::Integer(42.into())
                    .with_default_span(),
                else_branch: None,
            })
        )
    }

    #[test]
    fn parse_if_else() {
        let expr = parse("if (false) (0) else (1)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Conditional(Conditional {
                condition: DataExpressionData::Boolean(false.into())
                    .with_default_span(),
                then_branch: DataExpressionData::Integer(0.into())
                    .with_default_span(),
                else_branch: Some(
                    DataExpressionData::Integer(1.into()).with_default_span()
                ),
            })
        )
    }

    #[test]
    fn parse_nested_if_else() {
        let expr = parse("if (true) (1) else if (false) (2) else (3)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Conditional(Conditional {
                condition: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                then_branch: (DataExpressionData::Integer(1.into())
                    .with_default_span()),
                else_branch: Some(
                    DataExpressionData::Conditional(Conditional {
                        condition: DataExpressionData::Boolean(false.into())
                            .with_default_span(),
                        then_branch: DataExpressionData::Integer(2.into())
                            .with_default_span(),
                        else_branch: Some(
                            DataExpressionData::Integer(3.into())
                                .with_default_span()
                        ),
                    })
                    .with_default_span()
                ),
            })
        )
    }
}
