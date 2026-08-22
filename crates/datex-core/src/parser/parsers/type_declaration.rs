use crate::{
    ast::{
        expressions::{
            DatexExpression, DatexExpressionData, EntityDeclarationExpression,
            TypeDeclarationExpression,
        },
        spanned::Spanned,
    },
    parser::{Parser, SpannedParserError, errors::ParserError, lexer::Token},
    prelude::*,
};
impl Parser {
    pub(crate) fn parse_entity_declaration(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        Ok(match self.advance()?.token {
            // handle var and const declarations
            Token::EntityTypeDeclaration => {
                let (name, _) = self.expect_identifier()?;

                // optional generic parameters
                // TODO #664: use generic parameters
                let _generic_params = if self.peek()?.token == Token::LeftAngle
                {
                    Some(self.parse_generic_parameters()?)
                } else {
                    None
                };

                // expect equals sign
                self.expect(Token::Assign)?;

                // initializer expression
                let definition = self.parse_type_expression(0)?;

                DatexExpressionData::EntityDeclaration(
                    EntityDeclarationExpression {
                        id: None,
                        name,
                        definition,
                        hoisted: false,
                    },
                )
                .with_default_span()
            }

            _ => {
                return Err(SpannedParserError {
                    error: ParserError::UnexpectedToken {
                        expected: vec![Token::EntityTypeDeclaration],
                        found: self.peek()?.token.clone(),
                    },
                    span: self.peek()?.span.clone(),
                });
            }
        })
    }

    pub(crate) fn parse_type_declaration(
        &mut self,
    ) -> Result<DatexExpression, SpannedParserError> {
        Ok(match self.advance()?.token {
            // handle var and const declarations
            Token::TypeAlias => {
                let (name, _) = self.expect_identifier()?;

                // optional generic parameters
                // TODO #664: use generic parameters
                let _generic_params = if self.peek()?.token == Token::LeftAngle
                {
                    Some(self.parse_generic_parameters()?)
                } else {
                    None
                };

                // expect equals sign
                self.expect(Token::Assign)?;

                // initializer expression
                let definition = self.parse_type_expression(0)?;

                DatexExpressionData::TypeDeclaration(
                    TypeDeclarationExpression {
                        id: None,
                        name,
                        definition,
                        hoisted: false,
                    },
                )
                .with_default_span()
            }

            _ => {
                return Err(SpannedParserError {
                    error: ParserError::UnexpectedToken {
                        expected: vec![Token::TypeAlias],
                        found: self.peek()?.token.clone(),
                    },
                    span: self.peek()?.span.clone(),
                });
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::{
                DatexExpressionData, EntityDeclarationExpression,
                TypeDeclarationExpression,
            },
            spanned::Spanned,
            type_expressions::TypeExpressionData,
        },
        parser::tests::parse,
        prelude::*,
    };

    #[test]
    fn parse_entity_type_declaration() {
        let expr = parse("entity myType = true");
        assert_eq!(
            expr.data(),
            &DatexExpressionData::EntityDeclaration(
                EntityDeclarationExpression {
                    id: None,
                    name: "myType".to_string(),
                    definition: TypeExpressionData::Boolean(true.into())
                        .with_default_span(),
                    hoisted: false,
                }
            )
        );
    }

    #[test]
    fn parse_type_alias_declaration() {
        let expr = parse("type myAlias = false");
        assert_eq!(
            expr.data(),
            &DatexExpressionData::TypeDeclaration(TypeDeclarationExpression {
                id: None,
                name: "myAlias".to_string(),
                definition: TypeExpressionData::Boolean(false.into())
                    .with_default_span(),
                hoisted: false,
            })
        );
    }

    // TODO #665: generic parameters parsing
    #[test]
    fn parse_entity_type_declaration_with_generic_parameters() {
        let expr = parse("entity myType<T, U> = true");
        assert_eq!(
            expr.data(),
            &DatexExpressionData::EntityDeclaration(
                EntityDeclarationExpression {
                    id: None,
                    name: "myType".to_string(),
                    definition: TypeExpressionData::Boolean(true.into())
                        .with_default_span(),
                    hoisted: false,
                }
            )
        );
    }
}
