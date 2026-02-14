use crate::{
    ast::{
        expressions::{
            CallableDeclaration, CallableKind, DatexExpression,
            DatexExpressionData,
        },
        spanned::Spanned,
        type_expressions::TypeExpression,
    },
    parser::{Parser, SpannedParserError, lexer::Token},
};
use crate::ast::expressions::CompileExpression;
use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_compile_expression(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        let start = self.expect(Token::Compile)?.span.start;

        let compile_expression = self.parse_expression(0)?;
        
        Ok(DatexExpressionData::Compile(CompileExpression {
            expression: Box::new(compile_expression)
        }).with_span(start..self.get_current_source_position()))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                DatexExpressionData, Statements,
            },
            spanned::Spanned,
        },
        parser::tests::parse,
        prelude::*,
    };
    use crate::ast::expressions::{BinaryOperation, CompileExpression};
    use crate::global::operators::binary::ArithmeticOperator;
    use crate::global::operators::BinaryOperator;
    use crate::values::core_values::integer::Integer;

    #[test]
    fn parse_empty_compile() {
        let expr = parse("compile ()");
        assert_eq!(
            expr.data,
            DatexExpressionData::Compile(CompileExpression {
                expression: Box::new(
                    DatexExpressionData::Statements(Statements {
                        statements: vec![],
                        is_terminated: false,
                        unbounded: None,
                    }).with_default_span()
                )
            })
        );
    }
    
    #[test]
    fn parse_compile_with_expression() {
        let expr = parse("compile (1 + 2)");
        assert_eq!(
            expr.data,
            DatexExpressionData::Compile(CompileExpression {
                expression: Box::new(
                    DatexExpressionData::BinaryOperation(  BinaryOperation {
                        left: Box::new(DatexExpressionData::Integer(Integer::from(1)).with_default_span()),
                        operator: BinaryOperator::Arithmetic(ArithmeticOperator::Add),
                        right: Box::new(DatexExpressionData::Integer(Integer::from(2)).with_default_span()),
                        ty: None,
                    }).with_default_span()
                )
            })
        );
    }
    
    #[test]
    fn parse_compile_with_single_literal() {
        let expr = parse("compile 42");
        assert_eq!(
            expr.data,
            DatexExpressionData::Compile(CompileExpression {
                expression: Box::new(
                    DatexExpressionData::Integer(Integer::from(42)).with_default_span()
                )
            })
        );
    }
}
