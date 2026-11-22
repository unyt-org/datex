use crate::{
    ast::{
        expressions::{
            DataExpression, DataExpressionData, RootPropertyAccess,
            ValueAccessType,
        },
        spanned::Spanned,
    },
    global::stack_index::StackIndex,
    parser::{
        Parser, SpannedParserError,
        errors::ParserError,
        lexer::{DecimalWithVariant, IntegerWithVariant, Token},
        utils::{
            IntegerOrDecimal, IntegerOrTypedInteger, parse_integer_literal,
            parse_integer_with_variant, unescape_text,
        },
    },
    prelude::*,
    values::core_values::{
        decimal::{Decimal, typed_decimal::TypedDecimal},
        endpoint::Endpoint,
        instant::Instant,
    },
};
use core::str::FromStr;

impl Parser {
    pub(crate) fn parse_atom(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(match self.peek()?.token.clone() {
            Token::LeftCurly => self.parse_map()?,
            Token::LeftBracket => self.parse_list()?,
            Token::TypeExpressionStart => {
                self.parse_wrapped_type_expression()?
            }
            Token::LeftParen => self.parse_parenthesized_statements()?,
            Token::If => self.parse_if_else()?,
            Token::Function | Token::Procedure => {
                self.parse_callable_definition()?
            }
            Token::Compile => self.parse_compile_expression()?,

            Token::Placeholder => self.parse_placeholder()?,

            Token::True => self.parse_true()?,
            Token::False => self.parse_false()?,
            Token::Null => self.parse_null()?,
            Token::Identifier(name) => self.parse_identifier(name)?,
            Token::Tag(_) => self.parse_tagged_value()?,
            Token::StackIndex(stack_index) => {
                self.parse_stack_index(stack_index)?
            }
            Token::RootPropertyAccess(root_property_access) => {
                self.parse_root_property_access(root_property_access)?
            }
            Token::StringLiteral(value) => self.parse_string_literal(value)?,
            Token::Infinity => self.parse_infinity()?,
            Token::Nan => self.parse_nan()?,
            Token::Endpoint(endpoint_name) => {
                self.parse_endpoint(endpoint_name)?
            }
            Token::IntegerLiteral(integer_literal) => {
                self.parse_integer_literal(integer_literal)?
            }

            Token::BinaryIntegerLiteral(integer_with_variant)
            | Token::HexadecimalIntegerLiteral(integer_with_variant)
            | Token::OctalIntegerLiteral(integer_with_variant) => {
                self.parse_integer_with_variant(integer_with_variant)?
            }

            Token::DecimalLiteral(decimal_with_variant) => {
                self.parse_decimal_with_variant(decimal_with_variant)?
            }

            Token::FractionLiteral(fraction) => {
                self.parse_fraction_literal(fraction)?
            }

            Token::IsoDateTime(iso_string) => {
                self.parse_iso_datetime(iso_string)?
            }

            _ => {
                return Err(SpannedParserError {
                    error: ParserError::UnexpectedToken {
                        expected: vec![
                            Token::LeftCurly,
                            Token::LeftBracket,
                            Token::TypeExpressionStart,
                            Token::LeftParen,
                            Token::True,
                            Token::False,
                            Token::Null,
                            Token::Identifier("".to_string()),
                            Token::Placeholder,
                            Token::Tag("".to_string()),
                            Token::StringLiteral("".to_string()),
                            Token::Infinity,
                            Token::Nan,
                            Token::Endpoint("".to_string()),
                            Token::IntegerLiteral("".to_string()),
                            Token::BinaryIntegerLiteral(IntegerWithVariant {
                                value: "".to_string(),
                                variant: None,
                            }),
                            Token::HexadecimalIntegerLiteral(
                                IntegerWithVariant {
                                    value: "".to_string(),
                                    variant: None,
                                },
                            ),
                            Token::OctalIntegerLiteral(IntegerWithVariant {
                                value: "".to_string(),
                                variant: None,
                            }),
                            Token::DecimalLiteral(DecimalWithVariant {
                                value: "".to_string(),
                                variant: None,
                            }),
                            Token::FractionLiteral("".to_string()),
                        ],
                        found: self.peek()?.token.clone(),
                    },
                    span: self.peek()?.span.clone(),
                });
            }
        })
    }

    pub(crate) fn parse_wrapped_type_expression(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        let start_token = self.advance()?;
        let type_expression = self.parse_type_expression(0);
        match type_expression {
            Ok(type_expression) => {
                // expect closing ')'
                let end_token = self.expect(Token::RightAngle)?;
                let span = start_token.span.start..end_token.span.end;
                Ok(DataExpressionData::TypeExpression(type_expression)
                    .with_span(span))
            }
            Err(e) => self.collect_error_and_continue(e),
        }
    }

    pub(crate) fn parse_integer_literal(
        &mut self,
        integer_literal: String,
    ) -> Result<DataExpression, SpannedParserError> {
        let span = self.advance()?.span.clone();
        Ok(match parse_integer_literal(integer_literal) {
            Ok(IntegerOrDecimal::Integer(integer)) => {
                DataExpressionData::Integer(integer)
            }
            Ok(IntegerOrDecimal::TypedInteger(typed_integer)) => {
                DataExpressionData::TypedInteger(typed_integer)
            }
            Ok(IntegerOrDecimal::Decimal(decimal)) => {
                DataExpressionData::Decimal(decimal)
            }
            Ok(IntegerOrDecimal::TypedDecimal(typed_decimal)) => {
                DataExpressionData::TypedDecimal(typed_decimal)
            }
            Err(e) => {
                return self.collect_error_and_continue(SpannedParserError {
                    error: ParserError::NumberParseError(e),
                    span,
                });
            }
        }
        .with_span(span))
    }

    pub(crate) fn parse_integer_with_variant(
        &mut self,
        integer_with_variant: IntegerWithVariant,
    ) -> Result<DataExpression, SpannedParserError> {
        let token = self.advance()?;
        let span = token.span.clone();

        Ok(
            match parse_integer_with_variant(integer_with_variant, token.token)
            {
                Ok(IntegerOrTypedInteger::Integer(integer)) => {
                    DataExpressionData::Integer(integer)
                }
                Ok(IntegerOrTypedInteger::TypedInteger(typed_integer)) => {
                    DataExpressionData::TypedInteger(typed_integer)
                }
                Err(e) => {
                    return self.collect_error_and_continue(
                        SpannedParserError {
                            error: ParserError::NumberParseError(e),
                            span,
                        },
                    );
                }
            }
            .with_span(span),
        )
    }

    pub(crate) fn parse_decimal_with_variant(
        &mut self,
        decimal_with_variant: DecimalWithVariant,
    ) -> Result<DataExpression, SpannedParserError> {
        let value = decimal_with_variant.value;
        let variant = decimal_with_variant.variant;

        let span = self.advance()?.span.clone();
        let res = match variant {
            Some(var) => TypedDecimal::try_from_string_and_variant(&value, var)
                .map(DataExpressionData::TypedDecimal),
            None => Decimal::try_from_string(&value)
                .map(DataExpressionData::Decimal),
        };
        match res {
            Ok(expr) => Ok(expr.with_span(span)),
            Err(e) => self.collect_error_and_continue(SpannedParserError {
                error: ParserError::NumberParseError(e),
                span,
            }),
        }
    }

    pub(crate) fn parse_placeholder(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Placeholder(ValueAccessType::MoveOrCopy)
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_true(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Boolean(true.into())
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_false(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Boolean(false.into())
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_null(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Null.with_span(self.advance()?.span))
    }

    pub(crate) fn parse_identifier(
        &mut self,
        name: String,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(
            DataExpressionData::Identifier(name)
                .with_span(self.advance()?.span),
        )
    }

    pub(crate) fn parse_infinity(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Decimal(Decimal::Infinity)
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_nan(
        &mut self,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::Decimal(Decimal::Nan)
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_endpoint(
        &mut self,
        endpoint_name: String,
    ) -> Result<DataExpression, SpannedParserError> {
        let span = self.advance()?.span.clone();
        match Endpoint::from_str(endpoint_name.as_str()) {
            Err(e) => self.collect_error_and_continue(SpannedParserError {
                error: ParserError::InvalidEndpointName {
                    name: endpoint_name,
                    details: e,
                },
                span,
            }),
            Ok(endpoint) => {
                Ok(DataExpressionData::Endpoint(endpoint).with_span(span))
            }
        }
    }

    pub(crate) fn parse_stack_index(
        &mut self,
        index: String,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::StackIndex(StackIndex(
            index[1..].parse::<u32>().unwrap(),
        ))
        .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_root_property_access(
        &mut self,
        property_access: String,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(DataExpressionData::RootPropertyAccess(RootPropertyAccess {
            property_name: property_access[2..].to_string(), // remove leading '$.'
        })
        .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_string_literal(
        &mut self,
        value: String,
    ) -> Result<DataExpression, SpannedParserError> {
        let unescaped = unescape_text(&value);
        Ok(DataExpressionData::Text(unescaped.into())
            .with_span(self.advance()?.span))
    }

    pub(crate) fn parse_fraction_literal(
        &mut self,
        fraction: String,
    ) -> Result<DataExpression, SpannedParserError> {
        let span = self.advance()?.span.clone();
        // remove all underscores from fraction string
        let fraction: String = fraction.chars().filter(|&c| c != '_').collect();
        match Decimal::try_from_string(&fraction) {
            Ok(decimal) => {
                Ok(DataExpressionData::Decimal(decimal).with_span(span))
            }
            Err(e) => self.collect_error_and_continue(SpannedParserError {
                error: ParserError::NumberParseError(e),
                span,
            }),
        }
    }

    pub(crate) fn parse_iso_datetime(
        &mut self,
        iso_string: String,
    ) -> Result<DataExpression, SpannedParserError> {
        let span = self.advance()?.span.clone();
        let instant = Instant::instant_from_iso(&iso_string);
        Ok(DataExpressionData::Instant(instant).with_span(span))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                CloneExpression, DataExpressionData, DeriveSharedRef,
                RootPropertyAccess, Statements, TypeExpressionData, Union,
                ValueAccessType, expression::Expression,
            },
            spanned::Spanned,
        },
        global::stack_index::StackIndex,
        parser::{
            errors::ParserError,
            parser_result::ParserResult,
            tests::{
                parse, try_parse_and_collect_errors,
                try_parse_and_return_on_first_error,
            },
        },
        prelude::*,
        shared_values::ReferenceMutability,
        values::core_values::{
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::{Endpoint, InvalidEndpointError},
            instant::Instant,
            integer::typed_integer::TypedInteger,
        },
    };
    use core::assert_matches;

    #[test]
    fn parse_boolean_true() {
        let expr = parse("true");
        assert_eq!(expr.data(), &DataExpressionData::Boolean(true.into()));
    }

    #[test]
    fn parse_boolean_false() {
        let expr = parse("false");
        assert_eq!(expr.data(), &DataExpressionData::Boolean(false.into()));
    }

    #[test]
    fn parse_null() {
        let expr = parse("null");
        assert_eq!(expr.data(), &DataExpressionData::Null);
    }

    #[test]
    fn parse_identifier() {
        let expr = parse("myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Identifier("myVar".to_string())
        );
    }

    #[test]
    fn parse_string_literal() {
        let expr = parse("\"Hello, World!\"");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Text("Hello, World!".into())
        );
    }

    #[test]
    fn parse_infinity() {
        let expr = parse("infinity");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(Decimal::Infinity)
        );
    }

    #[test]
    fn parse_nan() {
        let expr = parse("nan");
        assert_eq!(expr.data(), &DataExpressionData::Decimal(Decimal::Nan));
    }

    #[test]
    fn parse_endpoint() {
        let expr = parse("@example");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Endpoint(Endpoint::new("@example"))
        );
    }

    #[test]
    fn parse_invalid_endpoint() {
        let result = try_parse_and_return_on_first_error("@x");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().error,
            ParserError::InvalidEndpointName {
                name: "@x".to_string(),
                details: InvalidEndpointError::MinLengthNotMet
            }
        );
    }

    #[test]
    fn parse_invalid_endpoint_and_continue() {
        let result = try_parse_and_collect_errors("@x; true");
        assert_matches!(result, ParserResult::Invalid { .. });
        let ast = result.ast();
        let errors = result.errors().unwrap();

        assert_eq!(
            ast.data(),
            &DataExpressionData::Statements(Statements {
                statements: vec![
                    DataExpressionData::Recover.with_default_span(),
                    DataExpressionData::Boolean(true.into())
                        .with_default_span(),
                ],
                is_terminated: false,
                unbounded: None,
            })
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error,
            ParserError::InvalidEndpointName {
                name: "@x".to_string(),
                details: InvalidEndpointError::MinLengthNotMet
            }
        );
    }

    #[test]
    fn parse_stack_index() {
        let expr = parse("\\42");
        assert_eq!(
            expr.data(),
            &DataExpressionData::StackIndex(StackIndex(42))
        );
    }

    #[test]
    fn parse_root_property_access() {
        let expr = parse("$.myProperty");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RootPropertyAccess(RootPropertyAccess {
                property_name: "myProperty".to_string()
            })
        );
    }

    #[test]
    fn parse_placeholder() {
        let expr = parse("?");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Placeholder(ValueAccessType::MoveOrCopy)
        );
    }

    #[test]
    fn parse_placeholder_shared_ref() {
        let expr = parse("'?");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                expression: (DataExpressionData::Placeholder(
                    ValueAccessType::MoveOrCopy
                )
                .with_default_span()),
                mutability: ReferenceMutability::Immutable,
            })
        );
    }

    #[test]
    fn parse_placeholder_shared_ref_mut() {
        let expr = parse("'mut ?");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                expression: (DataExpressionData::Placeholder(
                    ValueAccessType::MoveOrCopy
                )
                .with_default_span()),
                mutability: ReferenceMutability::Mutable,
            })
        );
    }

    #[test]
    fn parse_clone() {
        let expr = parse("clone ?");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Clone(CloneExpression {
                expression: (DataExpressionData::Placeholder(
                    ValueAccessType::MoveOrCopy
                )
                .with_default_span())
            })
        );
    }

    #[test]
    fn parse_integer_literal() {
        let expr = parse("12345");
        assert_eq!(expr.data(), &DataExpressionData::Integer(12345.into()));
    }

    #[test]
    fn parse_integer_literal_with_underscores() {
        let expr = parse("12_345_678");
        assert_eq!(expr.data(), &DataExpressionData::Integer(12345678.into()));
    }

    #[test]
    fn parse_negative_integer_literal() {
        let expr = parse("-6789");
        assert_eq!(expr.data(), &DataExpressionData::Integer((-6789).into()));
    }

    #[test]
    fn parse_typed_integer_literal() {
        let expr = parse("12345i32");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedInteger(TypedInteger::I32(12345))
        );
    }

    #[test]
    fn parse_hex_integer_literal() {
        let expr = parse("0x1A3F");
        assert_eq!(expr.data(), &DataExpressionData::Integer(0x1A3F.into()));
    }

    #[test]
    fn parse_binary_integer_literal() {
        let expr = parse("0b1101");
        assert_eq!(expr.data(), &DataExpressionData::Integer(0b1101.into()));
    }

    #[test]
    fn parse_octal_integer_literal() {
        let expr = parse("0o755");
        assert_eq!(expr.data(), &DataExpressionData::Integer(0o755.into()));
    }

    #[test]
    fn parse_typed_hex_integer_literal() {
        let expr = parse("0xFFu8");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedInteger(TypedInteger::U8(0xFF))
        );
    }

    #[test]
    fn parse_typed_binary_integer_literal() {
        let expr = parse("0b1010i16");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedInteger(TypedInteger::I16(0b1010))
        );
    }

    #[test]
    fn parse_typed_octal_integer_literal() {
        let expr = parse("0o77u32");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedInteger(TypedInteger::U32(0o77))
        );
    }

    #[test]
    fn parse_decimal_literal() {
        let expr = parse("123.456");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("123.456").unwrap()
            )
        );
    }

    #[test]
    fn parse_typed_decimal_literal() {
        let expr = parse("78.9f32");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedDecimal(TypedDecimal::F32(78.9.into()))
        );
    }

    #[test]
    fn parse_negative_decimal_literal() {
        let expr = parse("-0.001");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("-0.001").unwrap()
            )
        );
    }

    #[test]
    fn parse_decimal_literal_exponent() {
        let expr = parse("1.23e4");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("1.23e4").unwrap()
            )
        );
    }

    #[test]
    fn parse_typed_decimal_literal_exponent() {
        let expr = parse("5.67e-8f64");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedDecimal(TypedDecimal::F64(
                5.67e-8.into()
            ))
        );
    }

    #[test]
    fn parse_int_decimal_literal_exponent() {
        let expr = parse("42e2");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("42e2").unwrap()
            )
        );
    }

    #[test]
    fn parse_typed_int_decimal_literal_exponent() {
        let expr = parse("100e3_f32");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypedDecimal(TypedDecimal::F32(100e3.into()))
        );
    }

    #[test]
    fn parse_fraction_literal() {
        let expr = parse("3/4");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("3/4").unwrap()
            )
        );
    }

    #[test]
    fn parse_fraction_literal_with_underscores() {
        let expr = parse("1_000/2_500");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("1000/2500").unwrap()
            )
        );
    }

    #[test]
    fn parse_negative_fraction_literal_with_underscores() {
        let expr = parse("-7_500/2_500");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Decimal(
                Decimal::try_from_string("-7500/2500").unwrap()
            )
        );
    }

    #[test]
    fn parse_iso_datetime_with_millis() {
        let expr = parse("2026-04-13T18:28:09.415Z");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Instant(Instant::instant_from_iso(
                "2026-04-13T18:28:09.415Z"
            ))
        );
    }

    #[test]
    fn parse_iso_datetime_without_millis() {
        let expr = parse("2026-04-13T18:28:09Z");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Instant(Instant::instant_from_iso(
                "2026-04-13T18:28:09Z"
            ))
        );
    }

    #[test]
    #[ignore = "Does panic"]
    fn parse_iso_datetime_without_seconds() {
        let expr = parse("2026-04-13T18:28Z");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Instant(Instant::instant_from_iso(
                "2026-04-13T18:28Z"
            ))
        );
    }

    #[test]
    fn parse_iso_datetime_epoch() {
        let expr = parse("1970-01-01T00:00:00.000Z");
        assert_eq!(expr.data(), &DataExpressionData::Instant(Instant(0)));
    }

    #[test]
    fn parse_type_expression() {
        let expr = parse("type<1 | 2>");
        assert_eq!(
            expr.data(),
            &DataExpressionData::TypeExpression(
                TypeExpressionData::Union(Union(vec![
                    TypeExpressionData::Integer(1.into()).with_default_span(),
                    TypeExpressionData::Integer(2.into()).with_default_span()
                ]))
                .with_default_span()
            )
        );
    }
}
