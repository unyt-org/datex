use crate::{
    ast::{
        expressions::{
            Apply, BinaryOperation, CloneExpression, ComparisonOperation,
            CreateMut, CreateShared, DataExpression, DataExpressionData,
            DeriveRef, DeriveSharedRef, GenericInstantiation,
            InterfaceMethodCall, PropertyAccess, PropertyAssignment,
            RangeDeclaration, RemoteExecution, RequestSharedRef,
            StackAssignment, UnaryOperation, Unbox, UnboxAssignment,
            VariableAssignment, expression::Expression,
        },
        spanned::Spanned,
    },
    global::operators::{
        ArithmeticUnaryOperator, BinaryOperator, ComparisonOperator,
        LogicalUnaryOperator, ModificationOperator, UnaryOperator,
        binary::{ArithmeticOperator, BitwiseOperator, LogicalOperator},
    },
    parser::{
        Parser,
        errors::{ParserError, SpannedParserError},
        lexer::{SpannedToken, Token},
    },
    prelude::*,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainerMutability,
    },
    types::type_definition_with_metadata::LocalReferenceMutability,
    values::core_values::error::NumberParseError,
};

pub static UNARY_BP: u8 = 29; // weaker than property access / apply, stronger than all other binary operators
const MAX_EXPRESSION_DEPTH: usize = 64;

impl Parser {
    pub(crate) fn parse_expression(
        &mut self,
        min_bp: u8,
    ) -> Result<DataExpression, SpannedParserError> {
        if self.expression_depth >= MAX_EXPRESSION_DEPTH {
            let span = match self.peek() {
                Ok(token) => token.span.clone(),
                Err(_) => 0..0,
            };
            return Err(SpannedParserError {
                error: ParserError::ExpressionNestingTooDeep,
                span,
            });
        }

        self.expression_depth += 1;

        let result = stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.parse_expression_inner(min_bp)
        });

        self.expression_depth -= 1;

        result
    }

    pub(crate) fn parse_expression_inner(
        &mut self,
        min_bp: u8,
    ) -> Result<DataExpression, SpannedParserError> {
        let mut lhs = self.parse_prefix()?;

        while self.has_more_tokens() {
            let (_, r_bp) =
                match Parser::infix_binding_power(&self.peek()?.token) {
                    Some(bp) if bp.0 >= min_bp => bp,
                    _ => break,
                };

            let op = self.peek()?.clone();

            lhs = self.parse_binary_operation(lhs, op, r_bp)?;
        }

        Ok(lhs)
    }

    fn parse_binary_operation(
        &mut self,
        lhs: DataExpression,
        op: SpannedToken,
        r_bp: u8,
    ) -> Result<DataExpression, SpannedParserError> {
        Ok(match op.token {
            Token::Arrow => {
                self.advance()?; // consume the operator

                let method_name = self.parse_identifier_string()?.0;
                let (rhs, end_token) = self.parse_apply_arguments()?;
                let span = lhs.span.start..end_token;
                DataExpressionData::InterfaceMethodCall(InterfaceMethodCall {
                    target: lhs,
                    method_name,
                    arguments: rhs,
                })
                .with_span(span)
            }
            // property access
            Token::Dot => {
                self.advance()?; // consume the dot

                let rhs = self.parse_key()?;
                // x->xxx

                let span = lhs.span.start..rhs.span.end;

                DataExpressionData::PropertyAccess(PropertyAccess {
                    base: (lhs),
                    property: (rhs),
                })
                .with_span(span)
            }
            // binary operations
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Caret
            | Token::And
            | Token::Or
            | Token::Ampersand
            | Token::Pipe => {
                self.advance()?; // consume the operator
                let rhs = self.parse_expression(r_bp)?;
                let span = lhs.span.start..rhs.span.end;

                DataExpressionData::BinaryOperation(BinaryOperation {
                    left: (lhs),
                    operator: Parser::binary_operator_from_token(&op),
                    right: (rhs),
                    ty: None,
                })
                .with_span(span)
            }

            // comparison operators
            Token::Equal
            | Token::Is
            | Token::StructuralEqual
            | Token::NotEqual
            | Token::NotStructuralEqual
            | Token::LessEqual
            | Token::RightAngle
            | Token::GreaterEqual => {
                self.advance()?; // consume the operator
                let rhs = self.parse_expression(r_bp)?;
                let span = lhs.span.start..rhs.span.end;

                DataExpressionData::ComparisonOperation(ComparisonOperation {
                    left: (lhs),
                    operator: Parser::comparison_operator_from_token(&op),
                    right: (rhs),
                })
                .with_span(span)
            }

            Token::LeftAngle => {
                let generic_params =
                    self.try_parse_generic_parameters_or_roll_back();
                match generic_params {
                    Ok((params, end_span)) => {
                        let span = lhs.span.start..end_span.end;
                        DataExpressionData::GenericInstantiation(
                            GenericInstantiation {
                                base: (lhs),
                                generic_arguments: params,
                            },
                        )
                        .with_span(span)
                    }
                    _ => {
                        self.advance()?; // consume the operator
                        let rhs = self.parse_expression(r_bp)?;
                        let span = lhs.span.start..rhs.span.end;

                        DataExpressionData::ComparisonOperation(
                            ComparisonOperation {
                                left: (lhs),
                                operator: ComparisonOperator::LessThan,
                                right: (rhs),
                            },
                        )
                        .with_span(span)
                    }
                }
            }

            // assignment (=)
            Token::Assign
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign => self.parse_assignment(lhs, op, r_bp)?,

            // remote execution operator
            Token::DoubleColon => {
                self.advance()?; // consume the operator
                let rhs = self.parse_expression(r_bp)?;
                let span = lhs.span.start..rhs.span.end;
                DataExpressionData::RemoteExecution(RemoteExecution {
                    left: (lhs),
                    right: (rhs),
                    injected_variable_count: None,
                })
                .with_span(span)
            }

            Token::Range => {
                self.advance()?; // consume the operator
                if matches!(lhs.data(), &DataExpressionData::Range(_)) {
                    return Err(SpannedParserError {
                        error: ParserError::InvalidToken,
                        span: lhs.span,
                    });
                }

                let rhs = self.parse_expression(r_bp)?;
                let span = lhs.span.start..rhs.span.end;
                DataExpressionData::Range(RangeDeclaration {
                    start: (lhs),
                    end: (rhs),
                })
                .with_span(span)
            }

            // apply
            _ => {
                let (args, end) = self.parse_apply_arguments()?;
                let span = lhs.span.start..end;

                DataExpressionData::Apply(Apply {
                    base: (lhs),
                    arguments: args,
                })
                .with_span(span)
            }
        })
    }

    fn parse_assignment(
        &mut self,
        lhs: DataExpression,
        op: SpannedToken,
        r_bp: u8,
    ) -> Result<DataExpression, SpannedParserError> {
        self.advance()?; // consume the operator
        let rhs = self.parse_expression(r_bp)?;
        let span = lhs.span.start..rhs.span.end;
        let assignment_operator = match op.token {
            Token::Assign => None,
            Token::AddAssign => Some(ModificationOperator::AddAssign),
            Token::SubAssign => Some(ModificationOperator::SubtractAssign),
            Token::MulAssign => Some(ModificationOperator::MultiplyAssign),
            Token::DivAssign => Some(ModificationOperator::DivideAssign),
            _ => unreachable!(),
        };

        // select assignment type based on lhs
        Ok(match *lhs.data {
            // variable assignment
            DataExpressionData::Identifier(name) => {
                DataExpressionData::VariableAssignment(VariableAssignment {
                    id: None,
                    name,
                    operator: assignment_operator,
                    expression: (rhs),
                })
            }
            // property assignment
            DataExpressionData::PropertyAccess(prop_access) => {
                DataExpressionData::PropertyAssignment(PropertyAssignment {
                    operator: assignment_operator,
                    base: prop_access.base,
                    property: prop_access.property,
                    assigned_expression: (rhs),
                })
            }
            // unbox assignment
            DataExpressionData::Unbox(unbox) => {
                DataExpressionData::UnboxAssignment(UnboxAssignment {
                    operator: assignment_operator,
                    unbox_expression: unbox.expression,
                    assigned_expression: (rhs),
                })
            }
            // slot assignment
            DataExpressionData::StackIndex(slot) => {
                DataExpressionData::StackAssignment(StackAssignment {
                    index: slot,
                    expression: (rhs),
                })
            }
            // invalid lhs for assignment
            _ => {
                return self.collect_error_and_continue(SpannedParserError {
                    error: ParserError::InvalidAssignmentTarget,
                    span: lhs.span.clone(),
                });
            }
        }
        .with_span(span))
    }

    // TODO #666: handle single value without parentheses as argument
    fn parse_apply_arguments(
        &mut self,
    ) -> Result<(Vec<DataExpression>, usize), SpannedParserError> {
        let op = self.peek()?.clone();

        match op.token {
            // multiple arguments
            Token::LeftParen => {
                self.advance()?; // consume '('
                let mut args = Vec::new();

                while self.peek()?.token != Token::RightParen {
                    args.push(self.parse_expression(0)?);

                    if self.peek()?.token == Token::Comma {
                        self.advance()?;
                    }
                }

                let end = self.expect(Token::RightParen)?.span.end;
                Ok((args, end))
            }
            // single argument without parentheses
            _ => {
                let arg = self.parse_atom()?;
                let end = arg.span.end;
                Ok((vec![arg], end))
            }
        }
    }

    fn binary_operator_from_token(token: &SpannedToken) -> BinaryOperator {
        match token.token {
            Token::Plus => BinaryOperator::Arithmetic(ArithmeticOperator::Add),
            Token::Minus => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Subtract)
            }
            Token::Star => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Multiply)
            }
            Token::Slash => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Divide)
            }
            Token::Caret => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Power)
            }
            Token::And => BinaryOperator::Logical(LogicalOperator::And),
            Token::Or => BinaryOperator::Logical(LogicalOperator::Or),
            Token::Ampersand => BinaryOperator::Bitwise(BitwiseOperator::And),
            Token::Pipe => BinaryOperator::Bitwise(BitwiseOperator::Or),
            _ => unreachable!(),
        }
    }

    fn comparison_operator_from_token(
        token: &SpannedToken,
    ) -> ComparisonOperator {
        match token.token {
            Token::Is => ComparisonOperator::Is,
            Token::Equal => ComparisonOperator::Equal,
            Token::StructuralEqual => ComparisonOperator::StructuralEqual,
            Token::NotEqual => ComparisonOperator::NotEqual,
            Token::NotStructuralEqual => ComparisonOperator::NotStructuralEqual,
            Token::LeftAngle => ComparisonOperator::LessThan,
            Token::LessEqual => ComparisonOperator::LessThanOrEqual,
            Token::RightAngle => ComparisonOperator::GreaterThan,
            Token::GreaterEqual => ComparisonOperator::GreaterThanOrEqual,
            _ => unreachable!(),
        }
    }

    /// TODO: Do we need this still
    #[allow(unused)]
    fn unary_operator_from_token(
        token: &SpannedToken,
    ) -> Result<UnaryOperator, SpannedParserError> {
        match token.token {
            Token::Minus => {
                Ok(UnaryOperator::Arithmetic(ArithmeticUnaryOperator::Minus))
            }
            _ => Err(SpannedParserError {
                error: ParserError::UnexpectedToken {
                    expected: vec![Token::Minus],
                    found: token.token.clone(),
                },
                span: token.span.clone(),
            }),
        }
    }

    fn parse_prefix(&mut self) -> Result<DataExpression, SpannedParserError> {
        match self.peek()?.token {
            // minus:
            Token::Minus => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;

                Ok(match *rhs.data {
                    // special case: if the rhs is a literal integer/decimal, parse it as negative literal
                    DataExpressionData::Integer(value) => {
                        DataExpressionData::Integer(-value)
                    }
                    DataExpressionData::Decimal(value) => {
                        DataExpressionData::Decimal(-value)
                    }
                    DataExpressionData::TypedInteger(value) => match -value {
                        Ok(neg_value) => {
                            DataExpressionData::TypedInteger(neg_value)
                        }
                        Err(_e) => {
                            return Err(SpannedParserError {
                                error: ParserError::NumberParseError(
                                    NumberParseError::OutOfRange,
                                ),
                                span: rhs.span.clone(),
                            });
                        }
                    },
                    DataExpressionData::TypedDecimal(value) => {
                        DataExpressionData::TypedDecimal(-value)
                    }
                    // default case: unary minus operation
                    _ => DataExpressionData::UnaryOperation(UnaryOperation {
                        operator: UnaryOperator::Arithmetic(
                            ArithmeticUnaryOperator::Minus,
                        ),
                        expression: (rhs),
                    }),
                }
                .with_span(span))
            }
            // plus
            Token::Plus => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(match *rhs.data {
                    // special case: unary plus is a no-op for integer and decimal literals
                    DataExpressionData::Integer(value) => {
                        DataExpressionData::Integer(value)
                    }
                    DataExpressionData::Decimal(value) => {
                        DataExpressionData::Decimal(value)
                    }
                    DataExpressionData::TypedInteger(value) => {
                        DataExpressionData::TypedInteger(value)
                    }
                    DataExpressionData::TypedDecimal(value) => {
                        DataExpressionData::TypedDecimal(value)
                    }
                    // default case: unary plus operation
                    _ => DataExpressionData::UnaryOperation(UnaryOperation {
                        operator: UnaryOperator::Arithmetic(
                            ArithmeticUnaryOperator::Plus,
                        ),
                        expression: (rhs),
                    }),
                }
                .with_span(span))
            }
            // negation (!)
            Token::Exclamation => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::UnaryOperation(UnaryOperation {
                    operator: UnaryOperator::Logical(LogicalUnaryOperator::Not),
                    expression: (rhs),
                })
                .with_span(span))
            }
            // shared
            Token::Shared => {
                let op = self.advance()?;

                // check if next token is 'mut' to determine mutability of shared container
                let mutability = if let Ok(next) = self.peek()
                    && next.token == Token::Mutable
                {
                    let _token = self.advance()?; // consume 'mut'
                    SharedContainerMutability::Mutable
                } else {
                    SharedContainerMutability::Immutable
                };

                let rhs = self.parse_expression(UNARY_BP)?;
                let span_end = rhs.span.end;

                let span = op.span.start..span_end;
                Ok(DataExpressionData::CreateShared(CreateShared {
                    mutability,
                    expression: (rhs),
                })
                .with_span(span))
            }
            // mut
            Token::Mutable => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::CreateMut(CreateMut {
                    expression: (rhs),
                })
                .with_span(span))
            }
            // clone
            Token::Clone => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::Clone(CloneExpression {
                    expression: (rhs),
                })
                .with_span(span))
            }
            // ref (&)
            Token::Ampersand => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::DeriveRef(DeriveRef {
                    mutability: LocalReferenceMutability::Immutable,
                    expression: (rhs),
                })
                .with_span(span))
            }
            // mutable ref (&mut)
            Token::RefMut => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::DeriveRef(DeriveRef {
                    mutability: LocalReferenceMutability::Mutable,
                    expression: (rhs),
                })
                .with_span(span))
            }
            // shared ref (')
            Token::SharedRef => {
                let op = self.advance()?;

                // check if followed by pointer address literal
                if let Ok(next) = self.peek()
                    && let Token::PointerAddress(address) = &next.token
                {
                    let address =
                        PointerAddress::try_from(&address[1..]).unwrap();

                    let span = op.span.start..next.span.end;
                    self.advance()?; // consume pointer address

                    Ok(DataExpressionData::RequestSharedRef(
                        RequestSharedRef {
                            mutability: ReferenceMutability::Immutable,
                            address,
                        },
                    )
                    .with_span(span))
                }
                // normal shared ref to dynamic expression
                else {
                    let rhs = self.parse_expression(UNARY_BP)?;
                    let span = op.span.start..rhs.span.end;

                    Ok(DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: ReferenceMutability::Immutable,
                        expression: (rhs),
                    })
                    .with_span(span))
                }
            }
            // shared mutable ref ('mut)
            Token::SharedRefMut => {
                let op = self.advance()?;

                // check if followed by pointer address literal
                if let Ok(next) = self.peek()
                    && let Token::PointerAddress(address) = &next.token
                {
                    let address =
                        PointerAddress::try_from(&address[1..]).unwrap();

                    let span = op.span.start..next.span.end;
                    self.advance()?; // consume pointer address

                    Ok(DataExpressionData::RequestSharedRef(
                        RequestSharedRef {
                            mutability: ReferenceMutability::Mutable,
                            address,
                        },
                    )
                    .with_span(span))
                }
                // normal shared ref to dynamic expression
                else {
                    let rhs = self.parse_expression(UNARY_BP)?;
                    let span = op.span.start..rhs.span.end;

                    Ok(DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: ReferenceMutability::Mutable,
                        expression: (rhs),
                    })
                    .with_span(span))
                }
            }
            // unbox (*)
            Token::Star => {
                let op = self.advance()?;
                let rhs = self.parse_expression(UNARY_BP)?;
                let span = op.span.start..rhs.span.end;
                Ok(DataExpressionData::Unbox(Unbox { expression: (rhs) })
                    .with_span(span))
            }

            // everything else is an atom
            _ => self.parse_atom(),
        }
    }

    /// Returns the left and right binding powers for infix operators.
    /// The left binding power is used to determine if the operator can be parsed
    /// given the current minimum binding power, while the right binding power
    /// is used to determine the minimum binding power for the right-hand side expression.
    /// Left-associative operators have a binding power of (n, n+1),
    /// while right-associative operators have a binding power of (n, n).
    fn infix_binding_power(op: &Token) -> Option<(u8, u8)> {
        match op {
            // remote execution operator
            Token::DoubleColon => Some((1, 2)),
            // assignment operators
            Token::Assign
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign => Some((3, 3)),
            // comparison operators
            Token::Equal
            | Token::NotEqual
            | Token::StructuralEqual
            | Token::NotStructuralEqual
            | Token::Is => Some((5, 6)),
            Token::LeftAngle
            | Token::LessEqual
            | Token::RightAngle
            | Token::GreaterEqual => Some((7, 8)),
            // logical operators
            Token::Or => Some((9, 10)),
            Token::And => Some((11, 12)),
            // arithmetic operators
            Token::Plus | Token::Minus => Some((13, 14)),
            Token::Star | Token::Slash => Some((15, 16)),
            Token::Caret => Some((17, 17)), // right associative
            // bitwise operators
            Token::Pipe => Some((18, 19)),
            Token::Ampersand => Some((20, 21)),
            Token::Range => Some((22, 23)),
            // property access
            Token::Dot => Some((30, 31)),
            Token::Arrow => Some((32, 33)),
            // apply (function call, type cast), which has same binding power as member access
            Token::LeftParen
            | Token::LeftCurly
            | Token::LeftBracket
            | Token::True
            | Token::False
            | Token::Null
            | Token::Identifier(_)
            | Token::StringLiteral(_)
            | Token::Infinity
            | Token::Nan
            | Token::HexadecimalIntegerLiteral(_)
            | Token::BinaryIntegerLiteral(_)
            | Token::OctalIntegerLiteral(_)
            | Token::IntegerLiteral(_)
            | Token::DecimalLiteral(_)
            | Token::PointerAddress(_)
            | Token::StackIndex(_)
            | Token::Endpoint(_) => Some((33, 34)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                Apply, BinaryOperation, ComparisonOperation, CreateMut,
                CreateShared, DataExpressionData, DeriveRef, DeriveSharedRef,
                GenericInstantiation, InterfaceMethodCall, PropertyAccess,
                PropertyAssignment, RemoteExecution, RequestSharedRef,
                StackAssignment, Statements, TypeExpressionData,
                UnaryOperation, Unbox, UnboxAssignment, VariableAssignment,
                expression::Expression,
            },
            spanned::Spanned,
        },
        global::{
            operators::{
                ArithmeticUnaryOperator, BinaryOperator, ComparisonOperator,
                LogicalUnaryOperator, ModificationOperator, UnaryOperator,
                binary::{
                    ArithmeticOperator, BitwiseOperator, LogicalOperator,
                },
            },
            stack_index::StackIndex,
        },
        parser::{
            errors::ParserError,
            tests::{parse, try_parse_and_return_on_first_error},
        },
        prelude::*,
        shared_values::{
            PointerAddress, ReferenceMutability, SharedContainerMutability,
        },
        types::type_definition_with_metadata::LocalReferenceMutability,
    };

    use core::assert_matches;

    #[test]
    fn parse_simple_binary_expression() {
        let expr = parse("true + false");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Add),
                right: (DataExpressionData::Boolean(false.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_binary_expression_with_precedence() {
        let expr = parse("true + false * null");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Add),
                right: (DataExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (DataExpressionData::Boolean(false.into())
                            .with_default_span()),
                        operator: BinaryOperator::Arithmetic(
                            ArithmeticOperator::Multiply
                        ),
                        right: (DataExpressionData::Null.with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_unary_expression() {
        let expr = parse("-true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnaryOperation(UnaryOperation {
                operator: UnaryOperator::Arithmetic(
                    ArithmeticUnaryOperator::Minus
                ),
                expression: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_property_access() {
        let expr = parse("myObject.myProperty");
        assert_eq!(
            expr.data(),
            &DataExpressionData::PropertyAccess(PropertyAccess {
                base: (DataExpressionData::Identifier("myObject".to_string())
                    .with_default_span()),
                property: (DataExpressionData::Text("myProperty".into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_property_access_reserved_keywords() {
        let expr = parse("myObject.if");
        assert_eq!(
            expr.data(),
            &DataExpressionData::PropertyAccess(PropertyAccess {
                base: (DataExpressionData::Identifier("myObject".to_string())
                    .with_default_span()),
                property: (DataExpressionData::Text("if".into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_nested_property_access() {
        let expr = parse("myObject.innerObject.myProperty");
        assert_eq!(
            expr.data(),
            &DataExpressionData::PropertyAccess(PropertyAccess {
                base: (DataExpressionData::PropertyAccess(PropertyAccess {
                    base: (DataExpressionData::Identifier(
                        "myObject".to_string()
                    )
                    .with_default_span()),
                    property: (DataExpressionData::Text("innerObject".into())
                        .with_default_span()),
                })
                .with_default_span()),
                property: (DataExpressionData::Text("myProperty".into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_complex_expression() {
        let expr = parse("-myObject.value1 + myObject.value2 * true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::UnaryOperation(UnaryOperation {
                    operator: UnaryOperator::Arithmetic(
                        ArithmeticUnaryOperator::Minus
                    ),
                    expression: (DataExpressionData::PropertyAccess(
                        PropertyAccess {
                            base: (DataExpressionData::Identifier(
                                "myObject".to_string()
                            )
                            .with_default_span()),
                            property: (DataExpressionData::Text(
                                "value1".into()
                            )
                            .with_default_span()),
                        }
                    )
                    .with_default_span()),
                })
                .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Add),
                right: (DataExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (DataExpressionData::PropertyAccess(
                            PropertyAccess {
                                base: (DataExpressionData::Identifier(
                                    "myObject".to_string()
                                )
                                .with_default_span()),
                                property: (DataExpressionData::Text(
                                    "value2".into()
                                )
                                .with_default_span()),
                            }
                        )
                        .with_default_span()),
                        operator: BinaryOperator::Arithmetic(
                            ArithmeticOperator::Multiply
                        ),
                        right: (DataExpressionData::Boolean(true.into())
                            .with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_apply() {
        let expr = parse("myFunction(arg1, arg2)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::Identifier(
                    "myFunction".to_string()
                )
                .with_default_span()),
                arguments: vec![
                    DataExpressionData::Identifier("arg1".to_string())
                        .with_default_span(),
                    DataExpressionData::Identifier("arg2".to_string())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_apply_empty_arguments() {
        let expr = parse("myFunction()");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::Identifier(
                    "myFunction".to_string()
                )
                .with_default_span()),
                arguments: vec![],
            })
        );
    }

    #[test]
    fn parse_apply_without_parentheses() {
        let expr = parse("myFunction arg1");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::Identifier(
                    "myFunction".to_string()
                )
                .with_default_span()),
                arguments: vec![
                    DataExpressionData::Identifier("arg1".to_string())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_apply_with_property_access() {
        let expr = parse("myObject.myFunction(arg1)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::PropertyAccess(PropertyAccess {
                    base: (DataExpressionData::Identifier(
                        "myObject".to_string()
                    )
                    .with_default_span()),
                    property: (DataExpressionData::Text("myFunction".into())
                        .with_default_span()),
                })
                .with_default_span()),
                arguments: vec![
                    DataExpressionData::Identifier("arg1".into())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_multiple_applies() {
        let expr = parse("myFunction(arg1)(arg2)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::Apply(Apply {
                    base: (DataExpressionData::Identifier(
                        "myFunction".into()
                    )
                    .with_default_span()),
                    arguments: vec![
                        DataExpressionData::Identifier("arg1".into())
                            .with_default_span(),
                    ],
                })
                .with_default_span()),
                arguments: vec![
                    DataExpressionData::Identifier("arg2".into())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_remote_execution() {
        let expr = parse("endpoint::xy");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RemoteExecution(RemoteExecution {
                left: (DataExpressionData::Identifier("endpoint".to_string())
                    .with_default_span()),
                right: (DataExpressionData::Identifier("xy".to_string())
                    .with_default_span()),
                injected_variable_count: None,
            })
        );
    }

    #[test]
    fn parse_remote_execution_with_apply() {
        let expr = parse("endpoint::remoteFunction(arg1)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RemoteExecution(RemoteExecution {
                left: (DataExpressionData::Identifier("endpoint".to_string())
                    .with_default_span()),
                right: (DataExpressionData::Apply(Apply {
                    base: (DataExpressionData::Identifier(
                        "remoteFunction".to_string()
                    )
                    .with_default_span()),
                    arguments: vec![
                        DataExpressionData::Identifier("arg1".to_string())
                            .with_default_span(),
                    ],
                })
                .with_default_span()),
                injected_variable_count: None,
            })
        );
    }

    #[test]
    fn parse_remote_execution_multiple_statements() {
        let expr = parse("endpoint::(statement1; statement2)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RemoteExecution(RemoteExecution {
                left: (DataExpressionData::Identifier("endpoint".to_string())
                    .with_default_span()),
                right: (DataExpressionData::Statements(Statements {
                    statements: vec![
                        DataExpressionData::Identifier(
                            "statement1".to_string()
                        )
                        .with_default_span(),
                        DataExpressionData::Identifier(
                            "statement2".to_string()
                        )
                        .with_default_span(),
                    ],
                    is_terminated: false,
                    unbounded: None,
                })
                .with_default_span()),
                injected_variable_count: None,
            })
        );
    }

    #[test]
    fn parse_and() {
        let expr = parse("true and false");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                operator: BinaryOperator::Logical(LogicalOperator::And),
                right: (DataExpressionData::Boolean(false.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_or() {
        let expr = parse("true or false");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                operator: BinaryOperator::Logical(LogicalOperator::Or),
                right: (DataExpressionData::Boolean(false.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_bitwise_and() {
        let expr = parse("1 & 2");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Integer(1.into())
                    .with_default_span()),
                operator: BinaryOperator::Bitwise(BitwiseOperator::And),
                right: (DataExpressionData::Integer(2.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_bitwise_or() {
        let expr = parse("1 | 2");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Integer(1.into())
                    .with_default_span()),
                operator: BinaryOperator::Bitwise(BitwiseOperator::Or),
                right: (DataExpressionData::Integer(2.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_logical_expression_precedence() {
        let expr = parse("true or false and true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
                operator: BinaryOperator::Logical(LogicalOperator::Or),
                right: (DataExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (DataExpressionData::Boolean(false.into())
                            .with_default_span()),
                        operator: BinaryOperator::Logical(LogicalOperator::And),
                        right: (DataExpressionData::Boolean(true.into())
                            .with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_shared() {
        let expr = parse("shared myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::CreateShared(CreateShared {
                mutability: SharedContainerMutability::Immutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_mut() {
        let expr = parse("mut myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::CreateMut(CreateMut {
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_shared_mut() {
        let expr = parse("shared mut myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::CreateShared(CreateShared {
                mutability: SharedContainerMutability::Mutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_immutable_reference() {
        let expr = parse("&myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveRef(DeriveRef {
                mutability: LocalReferenceMutability::Immutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_immutable_shared_reference() {
        let expr = parse("'myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                mutability: ReferenceMutability::Immutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_mutable_reference() {
        let expr = parse("&mut myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveRef(DeriveRef {
                mutability: LocalReferenceMutability::Mutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_mutable_shared_reference() {
        let expr = parse("'mut myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                mutability: ReferenceMutability::Mutable,
                expression: (DataExpressionData::Identifier(
                    "myVar".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_ref_of_property_access() {
        let expr = parse("&myObject.myProperty");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveRef(DeriveRef {
                mutability: LocalReferenceMutability::Immutable,
                expression: (DataExpressionData::PropertyAccess(
                    PropertyAccess {
                        base: (DataExpressionData::Identifier(
                            "myObject".to_string()
                        )
                        .with_default_span()),
                        property: (DataExpressionData::Text(
                            "myProperty".into()
                        )
                        .with_default_span()),
                    }
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_shared_ref_of_property_access() {
        let expr = parse("'myObject.myProperty");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                mutability: ReferenceMutability::Immutable,
                expression: (DataExpressionData::PropertyAccess(
                    PropertyAccess {
                        base: (DataExpressionData::Identifier(
                            "myObject".to_string()
                        )
                        .with_default_span()),
                        property: (DataExpressionData::Text(
                            "myProperty".into()
                        )
                        .with_default_span()),
                    }
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_multiple_refs() {
        let expr = parse("&mut &myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveRef(DeriveRef {
                mutability: LocalReferenceMutability::Mutable,
                expression: (DataExpressionData::DeriveRef(DeriveRef {
                    mutability: LocalReferenceMutability::Immutable,
                    expression: (DataExpressionData::Identifier(
                        "myVar".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_multiple_shared_refs() {
        let expr = parse("'mut 'myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::DeriveSharedRef(DeriveSharedRef {
                mutability: ReferenceMutability::Mutable,
                expression: (DataExpressionData::DeriveSharedRef(
                    DeriveSharedRef {
                        mutability: ReferenceMutability::Immutable,
                        expression: (DataExpressionData::Identifier(
                            "myVar".to_string()
                        )
                        .with_default_span()),
                    }
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unbox() {
        let expr = parse("*myRef");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Unbox(Unbox {
                expression: (DataExpressionData::Identifier(
                    "myRef".to_string()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unbox_of_reference() {
        let expr = parse("*&myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Unbox(Unbox {
                expression: (DataExpressionData::DeriveRef(DeriveRef {
                    mutability: LocalReferenceMutability::Immutable,
                    expression: (DataExpressionData::Identifier(
                        "myVar".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unbox_of_shared_reference() {
        let expr = parse("*'myVar");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Unbox(Unbox {
                expression: (DataExpressionData::DeriveSharedRef(
                    DeriveSharedRef {
                        mutability: ReferenceMutability::Immutable,
                        expression: (DataExpressionData::Identifier(
                            "myVar".to_string()
                        )
                        .with_default_span()),
                    }
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_multiple_unboxes() {
        let expr = parse("**myRef");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Unbox(Unbox {
                expression: (DataExpressionData::Unbox(Unbox {
                    expression: (DataExpressionData::Identifier(
                        "myRef".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unbox_addition() {
        let expr = parse("*myRef + *myRef2");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Unbox(Unbox {
                    expression: (DataExpressionData::Identifier(
                        "myRef".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Add),
                right: (DataExpressionData::Unbox(Unbox {
                    expression: (DataExpressionData::Identifier(
                        "myRef2".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_variable_assignment() {
        let expr = parse("x = true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::VariableAssignment(VariableAssignment {
                id: None,
                name: "x".to_string(),
                operator: None,
                expression: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_invalid_variable_assignment() {
        let result = try_parse_and_return_on_first_error("42 = true");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().error,
            ParserError::InvalidAssignmentTarget
        );
    }

    #[test]
    fn parse_invalid_assignment_with_lhs_expression() {
        let result = try_parse_and_return_on_first_error("x + y = true");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().error,
            ParserError::InvalidAssignmentTarget
        );
    }

    #[test]
    fn parse_property_assignment() {
        let expr = parse("obj.prop = true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::PropertyAssignment(PropertyAssignment {
                operator: None,
                base: (DataExpressionData::Identifier("obj".to_string())
                    .with_default_span()),
                property: (DataExpressionData::Text("prop".into())
                    .with_default_span()),
                assigned_expression: (DataExpressionData::Boolean(
                    true.into()
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_variable_add_assignment() {
        let expr = parse("x += 42");
        assert_eq!(
            expr.data(),
            &DataExpressionData::VariableAssignment(VariableAssignment {
                id: None,
                name: "x".to_string(),
                operator: Some(ModificationOperator::AddAssign),
                expression: (DataExpressionData::Integer(42.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_stack_assignment() {
        let expr = parse("\\2 = 100");
        assert_eq!(
            expr.data(),
            &DataExpressionData::StackAssignment(StackAssignment {
                index: StackIndex(2),
                expression: (DataExpressionData::Integer(100.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unbox_assignment() {
        let expr = parse("*myRef = 200");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnboxAssignment(UnboxAssignment {
                operator: None,
                unbox_expression: (DataExpressionData::Identifier(
                    "myRef".to_string()
                )
                .with_default_span()),
                assigned_expression: (DataExpressionData::Integer(200.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_nested_unbox_assignment() {
        let expr = parse("**myRef = 300");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnboxAssignment(UnboxAssignment {
                operator: None,
                unbox_expression: (DataExpressionData::Unbox(Unbox {
                    expression: (DataExpressionData::Identifier(
                        "myRef".to_string()
                    )
                    .with_default_span()),
                })
                .with_default_span()),
                assigned_expression: (DataExpressionData::Integer(300.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_structural_equality_comparison() {
        let expr = parse("a == b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::StructuralEqual,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_is_comparison() {
        let expr = parse("a is b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::Is,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_equality_comparison() {
        let expr = parse("a === b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::Equal,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_structural_inequality_comparison() {
        let expr = parse("a != b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::NotStructuralEqual,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_inequality_comparison() {
        let expr = parse("a !== b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::NotEqual,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_less_than_comparison() {
        let expr = parse("a < b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::LessThan,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_generic_instantiation() {
        let expr = parse("MyType<Arg1>");
        assert_eq!(
            expr.data(),
            &DataExpressionData::GenericInstantiation(GenericInstantiation {
                base: (DataExpressionData::Identifier("MyType".to_string())
                    .with_default_span()),
                generic_arguments: vec![
                    TypeExpressionData::Identifier("Arg1".to_string())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_generic_instantiation_multiple_arguments() {
        let expr = parse("MyType<Arg1, Arg2>");
        assert_eq!(
            expr.data(),
            &DataExpressionData::GenericInstantiation(GenericInstantiation {
                base: (DataExpressionData::Identifier("MyType".to_string())
                    .with_default_span()),
                generic_arguments: vec![
                    TypeExpressionData::Identifier("Arg1".to_string())
                        .with_default_span(),
                    TypeExpressionData::Identifier("Arg2".to_string())
                        .with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_generic_instantiation_with_apply() {
        let expr = parse("MyType<Arg1>(42)");
        assert_eq!(
            expr.data(),
            &DataExpressionData::Apply(Apply {
                base: (DataExpressionData::GenericInstantiation(
                    GenericInstantiation {
                        base: (DataExpressionData::Identifier(
                            "MyType".to_string()
                        )
                        .with_default_span()),
                        generic_arguments: vec![
                            TypeExpressionData::Identifier("Arg1".to_string())
                                .with_default_span(),
                        ],
                    }
                )
                .with_default_span()),
                arguments: vec![
                    DataExpressionData::Integer(42.into()).with_default_span(),
                ],
            })
        );
    }

    #[test]
    fn parse_greater_than_comparison() {
        let expr = parse("a > b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::GreaterThan,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_less_equal_comparison() {
        let expr = parse("a <= b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::LessThanOrEqual,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_greater_equal_comparison() {
        let expr = parse("a >= b");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::GreaterThanOrEqual,
                right: (DataExpressionData::Identifier("b".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_comparison_precedence() {
        let expr = parse("a == b + c");
        assert_eq!(
            expr.data(),
            &DataExpressionData::ComparisonOperation(ComparisonOperation {
                left: (DataExpressionData::Identifier("a".to_string())
                    .with_default_span()),
                operator: ComparisonOperator::StructuralEqual,
                right: (DataExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (DataExpressionData::Identifier(
                            "b".to_string()
                        )
                        .with_default_span()),
                        operator: BinaryOperator::Arithmetic(
                            ArithmeticOperator::Add
                        ),
                        right: (DataExpressionData::Identifier(
                            "c".to_string()
                        )
                        .with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_negation() {
        let expr = parse("!true");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnaryOperation(UnaryOperation {
                operator: UnaryOperator::Logical(LogicalUnaryOperator::Not),
                expression: (DataExpressionData::Boolean(true.into())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unary_plus_integer() {
        let expr = parse("+42");
        assert_eq!(expr.data(), &DataExpressionData::Integer(42.into()));
    }

    #[test]
    fn parse_unary_plus_variable() {
        let expr = parse("+x");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnaryOperation(UnaryOperation {
                operator: UnaryOperator::Arithmetic(
                    ArithmeticUnaryOperator::Plus
                ),
                expression: (DataExpressionData::Identifier("x".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_unary_minus_integer() {
        let expr = parse("-42");
        assert_eq!(expr.data(), &DataExpressionData::Integer((-42).into()));
    }

    #[test]
    fn parse_unary_minus_variable() {
        let expr = parse("-x");
        assert_eq!(
            expr.data(),
            &DataExpressionData::UnaryOperation(UnaryOperation {
                operator: UnaryOperator::Arithmetic(
                    ArithmeticUnaryOperator::Minus
                ),
                expression: (DataExpressionData::Identifier("x".to_string())
                    .with_default_span()),
            })
        );
    }

    #[test]
    fn parse_power_expression() {
        let expr = parse("2 ^ 3");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Integer(2.into())
                    .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Power),
                right: (DataExpressionData::Integer(3.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_power_expression_right_associative() {
        let expr = parse("2 ^ 3 ^ 4");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::Integer(2.into())
                    .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Power),
                right: (DataExpressionData::BinaryOperation(
                    BinaryOperation {
                        left: (DataExpressionData::Integer(3.into())
                            .with_default_span()),
                        operator: BinaryOperator::Arithmetic(
                            ArithmeticOperator::Power
                        ),
                        right: (DataExpressionData::Integer(4.into())
                            .with_default_span()),
                        ty: None,
                    }
                )
                .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_power_expression_with_parentheses() {
        let expr = parse("(2 ^ 3) ^ 4");
        assert_eq!(
            expr.data(),
            &DataExpressionData::BinaryOperation(BinaryOperation {
                left: (DataExpressionData::BinaryOperation(BinaryOperation {
                    left: (DataExpressionData::Integer(2.into())
                        .with_default_span()),
                    operator: BinaryOperator::Arithmetic(
                        ArithmeticOperator::Power
                    ),
                    right: (DataExpressionData::Integer(3.into())
                        .with_default_span()),
                    ty: None,
                })
                .with_default_span()),
                operator: BinaryOperator::Arithmetic(ArithmeticOperator::Power),
                right: (DataExpressionData::Integer(4.into())
                    .with_default_span()),
                ty: None,
            })
        );
    }

    #[test]
    fn parse_pointer_address_ref() {
        let expr = parse("'$ABCDEFABCE");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RequestSharedRef(RequestSharedRef {
                address: PointerAddress::try_from("ABCDEFABCE").unwrap(),
                mutability: ReferenceMutability::Immutable,
            })
        );
    }

    #[test]
    fn parse_mutable_pointer_address_ref() {
        let expr = parse("'mut $ABCDEFABCE");
        assert_eq!(
            expr.data(),
            &DataExpressionData::RequestSharedRef(RequestSharedRef {
                address: PointerAddress::try_from("ABCDEFABCE").unwrap(),
                mutability: ReferenceMutability::Mutable,
            })
        );
    }

    #[test]
    fn excessive_nesting() {
        let input = "(".repeat(1_000) + "3" + &")".repeat(1_000);
        let result = try_parse_and_return_on_first_error(&input).unwrap_err();
        assert_matches!(result.error, ParserError::ExpressionNestingTooDeep);
    }

    #[test]
    fn interface_function_call() {
        let input = parse("hello->struct 4");
        assert_eq!(
            input.data(),
            &DataExpressionData::InterfaceMethodCall(InterfaceMethodCall {
                target: (DataExpressionData::Identifier("hello".to_string())
                    .with_default_span()),
                method_name: "struct".to_string(),
                arguments: vec![
                    DataExpressionData::Integer(4.into()).with_default_span()
                ],
            })
        );

        let input = parse("hello->struct(5, 2)");
        assert_eq!(
            input.data(),
            &DataExpressionData::InterfaceMethodCall(InterfaceMethodCall {
                target: (DataExpressionData::Identifier("hello".to_string())
                    .with_default_span()),
                method_name: "struct".to_string(),
                arguments: vec![
                    DataExpressionData::Integer(5.into()).with_default_span(),
                    DataExpressionData::Integer(2.into()).with_default_span()
                ],
            })
        );
        let input = parse("hello->struct()");
        assert_eq!(
            input.data(),
            &DataExpressionData::InterfaceMethodCall(InterfaceMethodCall {
                target: (DataExpressionData::Identifier("hello".to_string())
                    .with_default_span()),
                method_name: "struct".to_string(),
                arguments: vec![],
            })
        );
    }
}
