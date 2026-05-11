use crate::{
    ast::{
        expressions::{
            DatexExpression, DatexExpressionData,
        },
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, errors::ParserError, lexer::Token},
};
use crate::ast::expressions::TagExpression;
use crate::parser::lexer::SpannedToken;
use crate::parser::parsers::expression::UNARY_BP;
use crate::prelude::*;
impl Parser {
    pub(crate) fn parse_tagged_value(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        // expect next token to be a tag
        let tag = self.expect_tag()?;

        // if followed by no tokens, return empty tag
        if !self.has_more_tokens() {
            Ok(DatexExpressionData::Tag(TagExpression {
                tag,
                expression: None,
            }).with_default_span())
        }
        else {
            Ok(match self.peek()?.token.clone() {
                // if followed by bracket, handle inner expression
                Token::LeftCurly | Token::LeftBracket | Token::LeftParen => {
                    let maybe_expression = self.parse_expression(UNARY_BP);
                    let expression = self.recover_on_error(
                        maybe_expression,
                        &[], // TODO: recover?
                    )?;
                    DatexExpressionData::Tag(TagExpression {
                        tag,
                        expression: Some(Box::new(expression)),
                    }).with_default_span()
                }
                // else, return empty tag
                _ => DatexExpressionData::Tag(TagExpression {
                    tag,
                    expression: None,
                }).with_default_span()
            })
        }
    }

    /// Consumes the next token and expects it to be a tag, returning the tag string if successful.
    fn expect_tag(
        &mut self,
    ) -> Result<String, SpannedParserError> {
        match self.advance()? {
            SpannedToken {
                token: Token::Tag(tag),
                span,
            } => Ok(tag[1..].to_string()), // remove leading '#' from tag
            token => Err(SpannedParserError {
                error: ParserError::UnexpectedToken {
                    expected: vec![Token::Tag("tag".to_string())],
                    found: token.token.clone(),
                },
                span: token.span.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                DatexExpressionData,
            },
        },
        parser::tests::{parse},
        prelude::*,
    };
    use crate::ast::expressions::{ComparisonOperation, List, Map, TagExpression};
    use crate::ast::spanned::Spanned;
    use crate::global::operators::ComparisonOperator;
    use crate::values::core_values::integer::typed_integer::TypedInteger;

    #[test]
    fn parse_empty_tag() {
        let expr = parse("#MyTag");
        assert_eq!(
            expr.data,
            DatexExpressionData::Tag(TagExpression { tag: "MyTag".to_string(), expression: None })
        );
    }

    #[test]
    fn parse_tagged_map() {
        let expr = parse("#MyTag { a: 42u8 }");
        assert_eq!(
            expr.data,
            DatexExpressionData::Tag(TagExpression {
                tag: "MyTag".to_string(),
                expression: Some(Box::new(DatexExpressionData::Map(Map {
                    entries: vec![
                        (
                            DatexExpressionData::Text("a".to_string()).with_default_span(),
                            DatexExpressionData::TypedInteger(TypedInteger::U8(42)).with_default_span(),
                        )
                    ]
                }).with_default_span()))
            })
        );
    }

    #[test]
    fn parse_tagged_array() {
        let expr = parse("#MyTag [true, false]");
        assert_eq!(
            expr.data,
            DatexExpressionData::Tag(TagExpression {
                tag: "MyTag".to_string(),
                expression: Some(Box::new(DatexExpressionData::List(List {
                    items: vec![
                        DatexExpressionData::Boolean(true).with_default_span(),
                        DatexExpressionData::Boolean(false).with_default_span(),
                    ]
                }).with_default_span()))
            })
        );
    }

    #[test]
    fn parse_tagged_single_value() {
        let expr = parse("#MyTag (42u8)");
        assert_eq!(
            expr.data,
            DatexExpressionData::Tag(TagExpression {
                tag: "MyTag".to_string(),
                expression: Some(Box::new(DatexExpressionData::TypedInteger(TypedInteger::U8(42)).with_default_span()))
            })
        );
    }

    #[test]
    fn parse_list_of_tagged_values() {
        let expr = parse("[#Tag1 { a: 1u8 }, #Tag2, #Tag3 (42u8)]");

        assert_eq!(
            expr.data,
            DatexExpressionData::List(List {
                items: vec![
                    DatexExpressionData::Tag(TagExpression {
                        tag: "Tag1".to_string(),
                        expression: Some(Box::new(DatexExpressionData::Map(Map {
                            entries: vec![
                                (
                                    DatexExpressionData::Text("a".to_string()).with_default_span(),
                                    DatexExpressionData::TypedInteger(TypedInteger::U8(1)).with_default_span(),
                                )
                            ]
                        }).with_default_span()))
                    }).with_default_span(),
                    DatexExpressionData::Tag(TagExpression {
                        tag: "Tag2".to_string(),
                        expression: None
                    }).with_default_span(),
                    DatexExpressionData::Tag(TagExpression {
                        tag: "Tag3".to_string(),
                        expression: Some(Box::new(DatexExpressionData::TypedInteger(TypedInteger::U8(42)).with_default_span()))
                    }).with_default_span(),
                ]
            })
        );
    }

    #[test]
    fn parse_precedence() {
        let expr = parse("#Test (4u8) == 4u8");
        assert_eq!(
            expr.data,
            DatexExpressionData::ComparisonOperation(ComparisonOperation {
                operator: ComparisonOperator::StructuralEqual,
                left: Box::new(DatexExpressionData::Tag(TagExpression {
                    tag: "Test".to_string(),
                    expression: Some(Box::new(DatexExpressionData::TypedInteger(TypedInteger::U8(4)).with_default_span()))
                }).with_default_span()),
                right: Box::new(DatexExpressionData::TypedInteger(TypedInteger::U8(4)).with_default_span()),
            })
        );
    }
}
