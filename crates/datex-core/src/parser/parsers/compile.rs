use crate::{
    ast::{
        expressions::{CompileExpression, NormalExpression, NormalExpressionData},
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};
impl Parser {
    pub(crate) fn parse_compile_expression(
        &mut self,
    ) -> Result<NormalExpression, SpannedParserError> {
        let start = self.expect(Token::Compile)?.span.start;

        let compile_expression = self.parse_expression(0)?;

        Ok(NormalExpressionData::Compile(CompileExpression {
            expression: compile_expression,
        })
        .with_span(start..self.get_current_source_position()))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                BinaryOperation, CompileExpression, NormalExpressionData,
                Statements, expression::Expression,
            },
            spanned::Spanned,
        },
        global::operators::{BinaryOperator, binary::ArithmeticOperator},
        parser::tests::parse,
        prelude::*,
        values::core_values::integer::Integer,
    };

    #[test]
    fn parse_empty_compile() {
        let expr = parse("compile ()");
        assert_eq!(
            expr.data(),
            &NormalExpressionData::Compile(CompileExpression {
                expression: NormalExpressionData::Statements(Statements {
                    statements: vec![],
                    is_terminated: false,
                    unbounded: None,
                })
                .with_default_span()
            })
        );
    }

    #[test]
    fn parse_compile_with_expression() {
        let expr = parse("compile (1 + 2)");
        assert_eq!(
            expr.data(),
            &NormalExpressionData::Compile(CompileExpression {
                expression: (NormalExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (NormalExpressionData::Integer(Integer::from(1))
                            .with_default_span()),
                        operator: BinaryOperator::Arithmetic(
                            ArithmeticOperator::Add
                        ),
                        right: (NormalExpressionData::Integer(Integer::from(2))
                            .with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span())
            })
        );
    }

    #[test]
    fn parse_compile_with_single_literal() {
        let expr = parse("compile 42");
        assert_eq!(
            expr.data(),
            &NormalExpressionData::Compile(CompileExpression {
                expression: (NormalExpressionData::Integer(Integer::from(42))
                    .with_default_span())
            })
        );
    }
}
