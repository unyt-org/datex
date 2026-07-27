use crate::{
    ast::{
        expressions::{
            Apply, BinaryOperation, DatexExpression, DatexExpressionData,
            InterfaceMethodCall, List, Map, PropertyAssignment, Statements,
            UnaryOperation, UnboundedStatement, VariableAssignment,
        },
        spanned::Spanned,
        type_expressions::{TypeExpression, TypeExpressionData},
    },
    dxb_parser::{
        body::{DXBParserError, iterate_instructions},
        instruction_collector::{
            CollectedResults, CollectionResultsPopper, FullOrPartialResult,
            InstructionCollector, StatementResultCollectionStrategy,
        },
    },
    global::{
        operators::{BinaryOperator, UnaryOperator},
        protocol_structures::{
            instruction_data::{ShortStatementsData, StatementsData},
            instructions::Instruction,
        },
    },
    types::literal_type_definition::LiteralTypeDefinition,
    values::{
        core_values::{
            decimal::{Decimal, typed_decimal::TypedDecimal},
            integer::{Integer, typed_integer::TypedInteger},
        },
        value_container::ValueContainer,
    },
};

use crate::{
    ast::expressions::{
        CloneExpression, ComparisonOperation, CreateShared, DeriveSharedRef,
        RemoteExecution, RequestSharedRef, RootPropertyAccess, StackAssignment,
        StackListAssignment, TagExpression, UnboxAssignment,
    },
    global::{
        operators::{ComparisonOperator, ModificationOperator},
        protocol_structures::{
            instruction_data::{
                ShortTextData, StackIndex, TaggedValue, UnboundedStatementsData,
            },
            instructions::NestedInstructionResolutionStrategy,
            regular_instructions::RegularInstruction,
            type_instructions::TypeInstruction,
        },
    },
    prelude::*,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainerMutability,
    },
};
use alloc::format;
use core::cell::RefCell;

#[derive(Debug)]
enum CollectedAstResult {
    Expression(DatexExpression),
    TypeExpression(TypeExpression),
    KeyValuePair((DatexExpression, DatexExpression)),
}
impl From<DatexExpression> for CollectedAstResult {
    fn from(value: DatexExpression) -> Self {
        CollectedAstResult::Expression(value)
    }
}

impl From<TypeExpression> for CollectedAstResult {
    fn from(value: TypeExpression) -> Self {
        CollectedAstResult::TypeExpression(value)
    }
}

impl
    CollectionResultsPopper<
        CollectedAstResult,
        DatexExpression,
        DatexExpression,
        DatexExpression,
        TypeExpression,
        TypeExpression,
    > for CollectedResults<CollectedAstResult>
{
    fn try_extract_type_definition(
        result: CollectedAstResult,
    ) -> Option<TypeExpression> {
        match result {
            CollectedAstResult::TypeExpression(expr) => Some(expr),
            _ => None,
        }
    }

    /// Pops a DatexExpression from the collected results.
    fn try_extract_value(
        result: CollectedAstResult,
    ) -> Option<DatexExpression> {
        match result {
            CollectedAstResult::Expression(expr) => Some(expr),
            _ => None,
        }
    }

    /// Pops a TypeExpression from the collected results.
    fn try_extract_type(result: CollectedAstResult) -> Option<TypeExpression> {
        match result {
            CollectedAstResult::TypeExpression(expr) => Some(expr),
            _ => None,
        }
    }

    /// Pops a key-value pair from the collected results.
    fn try_extract_key_value_pair(
        result: CollectedAstResult,
    ) -> Option<(DatexExpression, DatexExpression)> {
        match result {
            CollectedAstResult::KeyValuePair((key, value)) => {
                Some((key, value))
            }
            _ => None,
        }
    }
}

// TODO: don't convert to AST directly, first generate disassembler tree, then convert to AST?
pub fn ast_from_bytecode(
    dxb: &[u8],
) -> Result<DatexExpression, DXBParserError> {
    let mut collector = InstructionCollector::<CollectedAstResult>::default();

    let mut next_stack_index = StackIndex(0);

    for instruction in iterate_instructions(
        Rc::new(RefCell::new(dxb.to_vec())),
        NestedInstructionResolutionStrategy::default(),
    ) {
        let instruction = instruction?;

        let result = match instruction {
            // handle regular instructions
            Instruction::Regular(regular_instruction) => {
                let regular_instruction = collector
                    .default_regular_instruction_collection(
                        regular_instruction,
                        StatementResultCollectionStrategy::Full,
                        next_stack_index,
                    );

                let expr = regular_instruction.map(|regular_instruction|
                    Result::<DatexExpression, DXBParserError>::Ok(match regular_instruction {
                        // Handle different regular instructions here
                        RegularInstruction::Int8(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::Int16(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::Int32(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::Instant(instant_data) => {
                            DatexExpressionData::DateTime(
                                crate::values::core_values::time::Instant(
                                    instant_data.0,
                                ),
                            )
                        }
                        RegularInstruction::Int64(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::Int128(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::UInt8(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::UInt16(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::UInt32(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::UInt64(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::UInt128(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::from(integer_data.0),
                            )
                        }
                        RegularInstruction::BigInteger(integer_data) => {
                            DatexExpressionData::TypedInteger(
                                TypedInteger::IBig(integer_data),
                            )
                        }
                        RegularInstruction::Integer(integer_data) => {
                            DatexExpressionData::Integer(integer_data)
                        }
                        RegularInstruction::Range => {
                            unreachable!("decompiler ast from bytcode ranges not implemented");
                        }
                        RegularInstruction::Endpoint(endpoint) => {
                            DatexExpressionData::Endpoint(endpoint)
                        }
                        RegularInstruction::DecimalF32(f32_data) => {
                            DatexExpressionData::TypedDecimal(
                                TypedDecimal::from(f32_data.0),
                            )
                        }
                        RegularInstruction::DecimalF64(f64_data) => {
                            DatexExpressionData::TypedDecimal(
                                TypedDecimal::from(f64_data.0),
                            )
                        }
                        RegularInstruction::DecimalAsInt16(
                            decimal_i16_data,
                        ) => DatexExpressionData::Decimal(Decimal::from(
                            decimal_i16_data.0 as f64,
                        )),
                        RegularInstruction::DecimalAsInt32(
                            decimal_i32_data,
                        ) => DatexExpressionData::Decimal(Decimal::from(
                            decimal_i32_data.0 as f64,
                        )),
                        RegularInstruction::BigDecimal(decimal_data) => {
                            DatexExpressionData::TypedDecimal(
                                TypedDecimal::Decimal(decimal_data),
                            )
                        }
                        RegularInstruction::Decimal(decimal_data) => {
                            DatexExpressionData::Decimal(decimal_data)
                        }
                        RegularInstruction::ShortText(short_text_data) => {
                            DatexExpressionData::Text(short_text_data.0.into())
                        }
                        RegularInstruction::Text(text_data) => {
                            DatexExpressionData::Text(text_data.0.into())
                        }
                        RegularInstruction::True => {
                            DatexExpressionData::Boolean(true.into())
                        }
                        RegularInstruction::False => {
                            DatexExpressionData::Boolean(false.into())
                        }
                        RegularInstruction::Null => {
                            DatexExpressionData::Null
                        }

                        RegularInstruction::RequestRemoteSharedRef(raw_address) => {
                            DatexExpressionData::RequestSharedRef(RequestSharedRef {
                                address: PointerAddress::from(raw_address),
                                mutability: ReferenceMutability::Immutable,
                            })
                        }

                        RegularInstruction::RequestRemoteSharedRefMut(raw_address) => {
                            DatexExpressionData::RequestSharedRef(RequestSharedRef {
                                address: PointerAddress::from(raw_address),
                                mutability: ReferenceMutability::Mutable,
                            })
                        }

                        RegularInstruction::GetLocalSharedRef(raw_address) => {
                            DatexExpressionData::RequestSharedRef(RequestSharedRef {
                                address: PointerAddress::from(raw_address),
                                mutability: ReferenceMutability::Immutable,
                            })
                        }

                        RegularInstruction::GetCoreLibValue(id) => {
                            DatexExpressionData::ResolveCoreLibId(
                                id.try_into().map_err(|_| DXBParserError::InvalidCoreLibId(id))?,
                            )
                        }

                        RegularInstruction::SharedRef(_shared_ref) => {
                            DatexExpressionData::NativeImplementationIndicator // TODO: better ast mapping
                        }

                        RegularInstruction::SharedRefWithValue(_shared_ref) => {
                            DatexExpressionData::NativeImplementationIndicator // TODO: better ast mapping
                        }

                        RegularInstruction::ConfirmMoves(_move_data) => {
                            DatexExpressionData::NativeImplementationIndicator // TODO: better ast mapping
                        }

                        RegularInstruction::CloneStackValue(stack_index) => {
                            DatexExpressionData::Clone(CloneExpression {
                                expression: (DatexExpressionData::StackIndex(stack_index).with_default_span())
                            })
                        }

                        RegularInstruction::BorrowStackValue(stack_index) => {
                            DatexExpressionData::StackIndex(stack_index)
                        }

                        RegularInstruction::GetStackValueSharedRef(stack_index) => {
                            DatexExpressionData::DeriveSharedRef(DeriveSharedRef {
                                mutability: ReferenceMutability::Immutable,
                                expression: (DatexExpressionData::StackIndex(stack_index).with_default_span()),
                            })
                        }

                        RegularInstruction::GetStackValueSharedRefMut(stack_index) => {
                            DatexExpressionData::DeriveSharedRef(DeriveSharedRef {
                                mutability: ReferenceMutability::Mutable,
                                expression: (DatexExpressionData::StackIndex(stack_index).with_default_span()),
                            })
                        }

                        RegularInstruction::TakeStackValue(stack_index) => {
                            DatexExpressionData::StackIndex(stack_index)
                        }

                        RegularInstruction::GetRootProperty(
                            root_property,
                        ) => {
                            DatexExpressionData::RootPropertyAccess(RootPropertyAccess {
                                property_name: root_property.to_string(),
                            })
                        }

                        RegularInstruction::TaggedValue(TaggedValue { is_empty: true, tag: ShortTextData(tag) }) => {
                            DatexExpressionData::Tag(TagExpression {
                                tag,
                                expression: None,
                            })
                        }

                        // NOTE: make sure that get_next_expected_instructions does not return None for these instructions!
                        RegularInstruction::Statements(_)
                        | RegularInstruction::ShortStatements(_)
                        | RegularInstruction::UnboundedStatements
                        | RegularInstruction::UnboundedStatementsEnd(
                            _,
                        )
                        | RegularInstruction::List(_)
                        | RegularInstruction::ShortList(_)
                        | RegularInstruction::Map(_)
                        | RegularInstruction::ShortMap(_)
                        | RegularInstruction::KeyValueDynamic
                        | RegularInstruction::KeyValueShortText(_)
                        | RegularInstruction::Add
                        | RegularInstruction::Subtract
                        | RegularInstruction::Multiply
                        | RegularInstruction::Divide
                        | RegularInstruction::UnaryMinus
                        | RegularInstruction::UnaryPlus
                        | RegularInstruction::BitwiseNot
                        | RegularInstruction::TaggedValue(TaggedValue { is_empty: false, .. })
                        | RegularInstruction::Apply(_)
                        | RegularInstruction::GetPropertyText(_)
                        | RegularInstruction::GetPropertyIndex(_)
                        | RegularInstruction::GetPropertyDynamic
                        | RegularInstruction::TakeEntryText(_)
                        | RegularInstruction::TakeEntryIndex(_)
                        | RegularInstruction::TakeEntryDynamic
                        | RegularInstruction::SetEntryText(_)
                        | RegularInstruction::SetEntryIndex(_)
                        | RegularInstruction::SetEntryDynamic
                        | RegularInstruction::Is
                        | RegularInstruction::Matches
                        | RegularInstruction::StructuralEqual
                        | RegularInstruction::Equal
                        | RegularInstruction::NotStructuralEqual
                        | RegularInstruction::NotEqual
                        | RegularInstruction::DeriveSharedReference
                        | RegularInstruction::DeriveSharedReferenceMut
                        | RegularInstruction::CreateShared
                        | RegularInstruction::CreateSharedMut
                        | RegularInstruction::PushToStack
                        | RegularInstruction::PushListToStack
                        | RegularInstruction::SetStackValue(_)
                        | RegularInstruction::Splice(_)
                        | RegularInstruction::SpliceDynamic
                        | RegularInstruction::AppendEntry
                        | RegularInstruction::Clear
                        | RegularInstruction::SetSharedContainerValue
                        | RegularInstruction::Unbox
                        | RegularInstruction::TypedValue
                        | RegularInstruction::Increment
                        | RegularInstruction::Decrement
                        | RegularInstruction::MoveWithValue(_)
                        | RegularInstruction::RemoteExecution(_)
                        | RegularInstruction::TypeExpression => {
                            unreachable!()
                        }
                        #[cfg(feature = "disassembler")]
                        RegularInstruction::_RemoteExecutionDebugFlat(_) | RegularInstruction::_RemoteExecutionDebugTree(_) => {
                            todo!("also map to ast")
                        }
                    }
                        .with_default_span()))
                    .transpose()?;
                expr.map(CollectedAstResult::from)
            }
            Instruction::Type(type_instruction) => {
                let type_instruction = collector
                    .default_type_instruction_collection(type_instruction);

                let type_expression: Option<TypeExpression> = type_instruction
                    .map(|type_instruction| {
                        match type_instruction {
                            TypeInstruction::TypeDefinitionCoreType(core_lib_id) => {
                                TypeExpressionData::Identifier(core_lib_id.to_string())
                            }
                            TypeInstruction::TypeDefinitionLiteral(literal) => {
                                match literal {
                                    LiteralTypeDefinition::Integer(integer) => {
                                        TypeExpressionData::Integer(integer)
                                    }
                                    LiteralTypeDefinition::Decimal(decimal) => {
                                        TypeExpressionData::Decimal(decimal)
                                    }
                                    LiteralTypeDefinition::Text(text) => {
                                        TypeExpressionData::Text(text)
                                    }
                                    LiteralTypeDefinition::Boolean(boolean) => {
                                        TypeExpressionData::Boolean(boolean)
                                    }
                                    LiteralTypeDefinition::Endpoint(endpoint) => {
                                        TypeExpressionData::Endpoint(endpoint)
                                    }
                                    LiteralTypeDefinition::TypedDecimal(decimal) => {
                                        TypeExpressionData::TypedDecimal(decimal)
                                    }
                                    LiteralTypeDefinition::TypedInteger(integer) => {
                                        TypeExpressionData::TypedInteger(integer)
                                    }
                                }
                            }
                            TypeInstruction::TypeDefinitionSharedTypeReference(reference) => {
                                // TODO #769: handle metadata
                                TypeExpressionData::GetReference(
                                    reference.address,
                                )
                            }
                            // NOTE: make sure that get_next_expected_instructions does not return None for these instructions!
                            TypeInstruction::TypeDefinitionList(_) |
                            TypeInstruction::TypeDefinitionRange |
                            TypeInstruction::TypeDefinitionImplType(_) |
                            TypeInstruction::TypeDefinitionMap(_) |
                            TypeInstruction::TypeDefinitionWithMetadata(_) => {
                                unreachable!()
                            }
                        }
                            .with_default_span()
                    });

                type_expression.map(CollectedAstResult::from)
            }
        };

        if let Some(result) = result {
            collector.push_result(result);
        }

        // handle collecting nested expressions
        while let Some(result) = collector.try_pop_collected() {
            match result {
                FullOrPartialResult::Full {
                    instruction,
                    results: mut collected_results,
                } => {
                    let expr: CollectedAstResult = match instruction {
                        Instruction::Regular(
                            regular_instruction,
                        ) => match regular_instruction {
                            RegularInstruction::List(_)
                            | RegularInstruction::ShortList(_) => {
                                let elements =
                                    collected_results.collect_value_results();
                                DatexExpressionData::List(List::new(elements))
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::Map(_)
                            | RegularInstruction::ShortMap(_) => {
                                let entries = collected_results
                                    .collect_key_value_pair_results();
                                DatexExpressionData::Map(Map::new(entries))
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::Statements(StatementsData { terminated, .. })
                            | RegularInstruction::ShortStatements(
                                ShortStatementsData { terminated, .. },
                            ) => {
                                let statements =
                                    collected_results.collect_value_results();
                                DatexExpressionData::Statements(Statements {
                                    statements,
                                    is_terminated: terminated,
                                    unbounded: None,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::KeyValueDynamic => {
                                let value =
                                    collected_results.pop_value();
                                let key = collected_results.pop_value();
                                CollectedAstResult::KeyValuePair((key, value))
                            }

                            RegularInstruction::KeyValueShortText(
                                short_text_data,
                            ) => {
                                let value =
                                    collected_results.pop_value();
                                let key = DatexExpressionData::Text(
                                    short_text_data.0.into(),
                                )
                                    .with_default_span();
                                CollectedAstResult::KeyValuePair((key, value))
                            }

                            RegularInstruction::Add
                            | RegularInstruction::Subtract
                            | RegularInstruction::Multiply
                            | RegularInstruction::Divide
                            | RegularInstruction::Matches => {
                                let right =
                                    collected_results.pop_value();
                                let left = collected_results.pop_value();
                                DatexExpressionData::BinaryOperation(
                                    BinaryOperation {
                                        operator: BinaryOperator::from(
                                            &regular_instruction,
                                        ),
                                        left: (left),
                                        right: (right),
                                        ty: None,
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::Is
                            | RegularInstruction::StructuralEqual
                            | RegularInstruction::Equal
                            | RegularInstruction::NotStructuralEqual
                            | RegularInstruction::NotEqual => {
                                let right =
                                    collected_results.pop_value();
                                let left = collected_results.pop_value();

                                DatexExpressionData::ComparisonOperation(
                                    ComparisonOperation {
                                        operator: ComparisonOperator::from(
                                            &regular_instruction,
                                        ),
                                        left: left,
                                        right: right,
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            instruction @ (
                            RegularInstruction::CreateShared | RegularInstruction::CreateSharedMut
                            ) => {
                                let expr = collected_results.pop_value();
                                DatexExpressionData::CreateShared(
                                    CreateShared {
                                        mutability: match instruction {
                                            RegularInstruction::CreateShared => {
                                                SharedContainerMutability::Immutable
                                            }
                                            RegularInstruction::CreateSharedMut => {
                                                SharedContainerMutability::Mutable
                                            }
                                            _ => unreachable!(),
                                        },
                                        expression: (expr),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::DeriveSharedReference => {
                                let expr = collected_results.pop_value();
                                DatexExpressionData::DeriveSharedRef(
                                    DeriveSharedRef {
                                        mutability: ReferenceMutability::Immutable,
                                        expression: (expr),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::DeriveSharedReferenceMut => {
                                let expr = collected_results.pop_value();
                                DatexExpressionData::DeriveSharedRef(
                                    DeriveSharedRef {
                                        mutability: ReferenceMutability::Mutable,
                                        expression: (expr),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::SetSharedContainerValue => {
                                DatexExpressionData::UnboxAssignment(UnboxAssignment {
                                    assigned_expression: (collected_results.pop_value()),
                                    operator: None,
                                    unbox_expression: (collected_results.pop_value()),
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::UnaryMinus
                            | RegularInstruction::UnaryPlus
                            | RegularInstruction::BitwiseNot
                            | RegularInstruction::Unbox => {
                                let expr = collected_results.pop_value();
                                DatexExpressionData::UnaryOperation(
                                    UnaryOperation {
                                        operator: UnaryOperator::from(
                                            &regular_instruction,
                                        ),
                                        expression: (expr),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::TypedValue => {
                                let expr = collected_results.pop_value();
                                let expr_type =
                                    collected_results.pop_type();
                                DatexExpressionData::Apply(Apply {
                                    base: (
                                        DatexExpressionData::TypeExpression(
                                            expr_type,
                                        )
                                            .with_default_span()
                                    ),
                                    arguments: vec![expr],
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::UnboundedStatementsEnd(
                                UnboundedStatementsData { terminated },
                            ) => {
                                let result = collector.try_pop_unbounded().ok_or(DXBParserError::NotInUnboundedRegularScopeError)?;
                                if let FullOrPartialResult::Full { results, .. } =
                                    result
                                {
                                    DatexExpressionData::Statements(
                                        Statements {
                                            statements: results
                                                .collect_value_results(),
                                            is_terminated: terminated,
                                            unbounded: Some(
                                                UnboundedStatement {
                                                    is_first: true,
                                                    is_last: true,
                                                },
                                            ),
                                        },
                                    )
                                        .with_default_span()
                                        .into()
                                } else {
                                    unreachable!()
                                }
                            }

                            RegularInstruction::MoveWithValue(move_with_value) => {
                                DatexExpressionData::MoveSharedValue(move_with_value.previous_address)
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::PushToStack => {
                                let expr = collected_results.pop_value();

                                let res = DatexExpressionData::StackAssignment(
                                    StackAssignment {
                                        index: next_stack_index,
                                        expression: (expr),
                                    }
                                )
                                    .with_default_span()
                                    .into();
                                next_stack_index += 1;

                                res
                            }

                            RegularInstruction::PushListToStack => {
                                let expression = collected_results.pop_value();
                                DatexExpressionData::StackListAssignment(StackListAssignment {
                                    expression,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::SetStackValue(slot_address) => {
                                let expr = collected_results.pop_value();
                                DatexExpressionData::VariableAssignment(
                                    VariableAssignment {
                                        id: None,
                                        name: format!(
                                            "_slot_{}",
                                            slot_address.0
                                        ),
                                        operator: None,
                                        expression: (expr),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::TaggedValue(TaggedValue {
                                                                tag: ShortTextData(tag),
                                                                is_empty
                                                            }) => {
                                assert!(!is_empty);
                                let expression = Some(collected_results.pop_value());

                                DatexExpressionData::Tag(TagExpression {
                                    tag,
                                    expression,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::Apply(_) => {
                                let mut arguments =
                                    collected_results.collect_value_results();
                                // base is the last collected argument
                                let base =
                                    arguments.remove(arguments.len() - 1);
                                DatexExpressionData::Apply(Apply {
                                    base: (base),
                                    arguments,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::TakeEntryIndex(
                                index_data,
                            ) | RegularInstruction::GetPropertyIndex(
                                index_data,
                            ) => {
                                let base = collected_results.pop_value();
                                DatexExpressionData::PropertyAccess(
                                    crate::ast::expressions::PropertyAccess {
                                        base: (base),
                                        property: (
                                            DatexExpressionData::Integer(
                                                Integer::from(index_data.0),
                                            )
                                                .with_default_span()
                                        ),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::TakeEntryText(text_data) | RegularInstruction::GetPropertyText(text_data) => {
                                let base = collected_results.pop_value();
                                DatexExpressionData::PropertyAccess(
                                    crate::ast::expressions::PropertyAccess {
                                        base: (base),
                                        property: (
                                            DatexExpressionData::Text(
                                                text_data.0.into(),
                                            )
                                                .with_default_span()
                                        ),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::TakeEntryDynamic | RegularInstruction::GetPropertyDynamic => {
                                let base = collected_results.pop_value();
                                let property =
                                    collected_results.pop_value();
                                DatexExpressionData::PropertyAccess(
                                    crate::ast::expressions::PropertyAccess {
                                        base: (base),
                                        property: (property),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::SetEntryIndex(
                                index_data,
                            ) => {
                                let base = collected_results.pop_value();
                                let value =
                                    collected_results.pop_value();
                                DatexExpressionData::PropertyAssignment(
                                    PropertyAssignment {
                                        base: (base),
                                        property: (
                                            DatexExpressionData::Integer(
                                                Integer::from(index_data.0),
                                            )
                                                .with_default_span()
                                        ),
                                        operator: None,
                                        assigned_expression: (value),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::Splice(splice_data) => {
                                let target = collected_results.pop_value();
                                let values = collected_results.pop_values(
                                    splice_data.insert_count
                                );
                                let splice_args = vec![
                                    DatexExpressionData::Integer(splice_data.start_index.into()).with_default_span(),
                                    DatexExpressionData::Integer(splice_data.delete_count.into()).with_default_span(),
                                    DatexExpressionData::List(List { items: values }).with_default_span()
                                ];
                                DatexExpressionData::InterfaceMethodCall(InterfaceMethodCall::new(
                                    target,
                                    "splice".to_string(),
                                    splice_args,
                                )).with_default_span().into()
                            }
                            RegularInstruction::SpliceDynamic => {
                                let target = collected_results.pop_value(); 
                                let values = collected_results.pop_value(); 

                                let delete_count = collected_results.pop_value();
                                let start_index = collected_results.pop_value();

                                DatexExpressionData::InterfaceMethodCall(InterfaceMethodCall::new(
                                    target,
                                    "splice".to_string(),
                                    vec![
                                        start_index,
                                        delete_count,
                                        values,
                                    ],
                                )).with_default_span().into()
                            }
                            RegularInstruction::AppendEntry => {
                                let target = collected_results.pop_value();
                                let value = collected_results.pop_value();

                                DatexExpressionData::InterfaceMethodCall(InterfaceMethodCall::new(
                                    target,
                                    "append".to_string(),
                                    vec![value],
                                )).with_default_span().into()
                            }
                            RegularInstruction::Clear => {
                                let target = collected_results.pop_value();
                                DatexExpressionData::InterfaceMethodCall(InterfaceMethodCall::new(
                                    target,
                                    "clear".to_string(),
                                    vec![],
                                )).with_default_span().into()
                            }
                            RegularInstruction::Increment => {
                                let base = collected_results.pop_value();
                                let value =
                                    collected_results.pop_value();
                                DatexExpressionData::UnboxAssignment(UnboxAssignment {
                                    operator: Some(ModificationOperator::AddAssign),
                                    unbox_expression: base,
                                    assigned_expression: value,
                                })
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::Decrement => {
                                let base = collected_results.pop_value();
                                let value =
                                    collected_results.pop_value();
                                DatexExpressionData::UnboxAssignment(UnboxAssignment {
                                    operator: Some(ModificationOperator::SubtractAssign),
                                    unbox_expression: base,
                                    assigned_expression: value,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::SetEntryText(text_data) => {
                                let base = collected_results.pop_value();
                                let value =
                                    collected_results.pop_value();
                                DatexExpressionData::PropertyAssignment(
                                    PropertyAssignment {
                                        base: (base),
                                        property: (
                                            DatexExpressionData::Text(
                                                text_data.0.into(),
                                            )
                                                .with_default_span()
                                        ),
                                        operator: None,
                                        assigned_expression: (value),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }

                            RegularInstruction::SetEntryDynamic => {
                                let base = collected_results.pop_value();
                                let value =
                                    collected_results.pop_value();
                                let property =
                                    collected_results.pop_value();
                                DatexExpressionData::PropertyAssignment(
                                    PropertyAssignment {
                                        base: (base),
                                        property: (property),
                                        operator: None,
                                        assigned_expression: (value),
                                    },
                                )
                                    .with_default_span()
                                    .into()
                            }
                            RegularInstruction::RemoteExecution(remote_execution_data) => {
                                let receivers = collected_results.pop_value();

                                let body = DatexExpressionData::Statements(Statements {
                                    statements: vec![ast_from_bytecode(&remote_execution_data.body)?],
                                    is_terminated: false,
                                    unbounded: None,
                                }).with_default_span();

                                DatexExpressionData::RemoteExecution(RemoteExecution {
                                    left: (receivers),
                                    right: (body),
                                    injected_variable_count: None,
                                })
                                    .with_default_span()
                                    .into()
                            }

                            e => {
                                todo!(
                                    "Unhandled collected regular instruction: {:?}",
                                    e
                                );
                            }
                        },

                        Instruction::Type(_data) => {
                            todo!("#656 Undescribed by author.")
                        }
                    };
                    collector.push_result(expr);
                }
                _ => unreachable!(),
            }
        }
    }

    if let Some(result) = collector.take_root_result() {
        match result {
            CollectedAstResult::Expression(expr) => Ok(expr),
            _ => unreachable!("Expected root result"),
        }
    } else {
        panic!("Execution finished without root result");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{
            expressions::{
                DatexExpressionData, PropertyAccess, PropertyAssignment,
            },
            spanned::Spanned,
        },
        global::{
            instruction_codes::InstructionCode,
            operators::{ModificationOperator, binary::ArithmeticOperator},
        },
        prelude::*,
        values::core_values::integer::{Integer, typed_integer::TypedInteger},
    };

    #[test]
    fn ast_from_bytecode_simple_integer() {
        let bytecode: Vec<u8> = vec![InstructionCode::UINT_8 as u8, 0x2A];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
                .with_default_span()
        );
    }

    #[test]
    fn ast_from_bytecode_null() {
        let bytecode: Vec<u8> = vec![InstructionCode::NULL as u8];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(ast, DatexExpressionData::Null.with_default_span());
    }

    #[test]
    fn ast_from_bytecode_simple_boolean() {
        let bytecode: Vec<u8> = vec![InstructionCode::TRUE as u8];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::Boolean(true.into()).with_default_span()
        );
    }

    #[test]
    fn ast_from_bytecode_simple_text() {
        let bytecode: Vec<u8> = vec![
            InstructionCode::SHORT_TEXT as u8,
            0x05, // length 5
            b'H',
            b'e',
            b'l',
            b'l',
            b'o',
        ];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::Text("Hello".into()).with_default_span()
        );
    }

    #[test]
    fn ast_from_bytecode_simple_list() {
        let bytecode: Vec<u8> = vec![
            InstructionCode::SHORT_LIST as u8,
            0x02, // 2 elements
            InstructionCode::UINT_8 as u8,
            0x2A, // 42
            InstructionCode::UINT_8 as u8,
            0x15, // 21
        ];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::List(List::new(vec![
                DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
                    .with_default_span(),
                DatexExpressionData::TypedInteger(TypedInteger::from(21u8))
                    .with_default_span(),
            ]))
            .with_default_span()
        );
    }

    #[test]
    fn ast_from_bytecode_nested_list() {
        let bytecode: Vec<u8> = vec![
            InstructionCode::SHORT_LIST as u8,
            0x02, // 2 elements
            InstructionCode::SHORT_LIST as u8,
            0x02, // 2 elements
            InstructionCode::UINT_8 as u8,
            0x01, // 1
            InstructionCode::UINT_8 as u8,
            0x02, // 2
            InstructionCode::UINT_8 as u8,
            0x03, // 3
        ];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::List(List::new(vec![
                DatexExpressionData::List(List::new(vec![
                    DatexExpressionData::TypedInteger(TypedInteger::from(1u8))
                        .with_default_span(),
                    DatexExpressionData::TypedInteger(TypedInteger::from(2u8))
                        .with_default_span(),
                ]))
                .with_default_span(),
                DatexExpressionData::TypedInteger(TypedInteger::from(3u8))
                    .with_default_span(),
            ]))
            .with_default_span()
        );
    }

    #[test]
    fn ast_from_bytecode_statements() {
        let bytecode: Vec<u8> = vec![
            InstructionCode::SHORT_STATEMENTS as u8,
            0x02, // 2 statements
            0x01, // terminated
            InstructionCode::UINT_8 as u8,
            42,
            InstructionCode::UINT_8 as u8,
            21,
        ];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::Statements(Statements {
                statements: vec![
                    DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
                        .with_default_span(),
                    DatexExpressionData::TypedInteger(TypedInteger::from(21u8))
                        .with_default_span(),
                ],
                is_terminated: true,
                unbounded: None,
            })
            .with_default_span()
        );
    }

    #[test]
    fn ast_from_nested_expressions() {
        let bytecode: Vec<u8> = vec![
            InstructionCode::SHORT_LIST as u8,
            0x03, // 3 elements
            InstructionCode::UINT_8 as u8,
            0x01, // 1
            InstructionCode::UINT_8 as u8,
            0x02, // 2
            InstructionCode::ADD as u8,
            InstructionCode::UINT_8 as u8,
            0x03, // 3
            InstructionCode::UINT_8 as u8,
            0x04, // 4
        ];
        let ast = ast_from_bytecode(&bytecode).unwrap();
        assert_eq!(
            ast,
            DatexExpressionData::List(List::new(vec![
                DatexExpressionData::TypedInteger(TypedInteger::from(1u8))
                    .with_default_span(),
                DatexExpressionData::TypedInteger(TypedInteger::from(2u8))
                    .with_default_span(),
                DatexExpressionData::BinaryOperation(BinaryOperation {
                    operator: BinaryOperator::Arithmetic(
                        ArithmeticOperator::Add
                    ),
                    left: (DatexExpressionData::TypedInteger(
                        TypedInteger::from(3u8)
                    )
                    .with_default_span()),
                    right: (DatexExpressionData::TypedInteger(
                        TypedInteger::from(4u8)
                    )
                    .with_default_span()),
                    ty: None
                })
                .with_default_span(),
            ]))
            .with_default_span()
        );
    }

    // FIXME @Vasyl-Trefilov: reenable and migrate to latest macro matching

    // #[test]
    // fn typed_value() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::TYPED_VALUE as u8,
    //         TypeInstructionCode::TYPE_LITERAL_SHORT_TEXT as u8,
    //         2,
    //         b'O',
    //         b'K',
    //         InstructionCode::UINT_8 as u8,
    //         43,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast,
    //         DatexExpressionData::Apply(Apply {
    //             base: (
    //                 DatexExpressionData::TypeExpression(
    //                     TypeExpressionData::Text("OK".to_string())
    //                         .with_default_span()
    //                 )
    //                 .with_default_span()
    //             ),
    //             arguments: vec![
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(43u8))
    //                     .with_default_span()
    //             ],
    //         })
    //         .with_default_span()
    //     );
    // }

    // #[test]
    // fn unbounded_statements() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::UNBOUNDED_STATEMENTS as u8,
    //         InstructionCode::UINT_8 as u8,
    //         10,
    //         InstructionCode::UINT_8 as u8,
    //         20,
    //         InstructionCode::UNBOUNDED_STATEMENTS_END as u8,
    //         1, // terminated
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast,
    //         DatexExpressionData::Statements(Statements {
    //             statements: vec![
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(10u8))
    //                     .with_default_span(),
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(20u8))
    //                     .with_default_span(),
    //             ],
    //             is_terminated: true,
    //             unbounded: Some(UnboundedStatement {
    //                 is_first: true,
    //                 is_last: true
    //             }),
    //         })
    //         .with_default_span()
    //     );
    // }

    // #[test]
    // fn apply_zero_arguments() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::APPLY_ZERO as u8,
    //         InstructionCode::SHORT_TEXT as u8,
    //         4, // length 4
    //         b't',
    //         b'e',
    //         b's',
    //         b't',
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast,
    //         DatexExpressionData::Apply(Apply {
    //             base: (
    //                 DatexExpressionData::Text("test".to_string())
    //                     .with_default_span()
    //             ),
    //             arguments: vec![],
    //         })
    //         .with_default_span()
    //     );
    // }

    // #[test]
    // fn apply_single_argument() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::APPLY_SINGLE as u8,
    //         InstructionCode::UINT_8 as u8,
    //         0, // argument 0
    //         InstructionCode::SHORT_TEXT as u8,
    //         3, // length 3
    //         b's',
    //         b'i',
    //         b'n',
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast,
    //         DatexExpressionData::Apply(Apply {
    //             base: (
    //                 DatexExpressionData::Text("sin".to_string())
    //                     .with_default_span()
    //             ),
    //             arguments: vec![
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(0u8))
    //                     .with_default_span()
    //             ],
    //         })
    //         .with_default_span()
    //     );
    // }

    // #[test]
    // fn apply_multiple_arguments() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::APPLY as u8,
    //         2, // 2 arguments
    //         0,
    //         InstructionCode::UINT_8 as u8,
    //         1, // argument 1
    //         InstructionCode::UINT_8 as u8,
    //         2, // argument 2
    //         InstructionCode::SHORT_TEXT as u8,
    //         3, // length 3
    //         b'a',
    //         b'd',
    //         b'd',
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast,
    //         DatexExpressionData::Apply(Apply {
    //             base: (
    //                 DatexExpressionData::Text("add".to_string())
    //                     .with_default_span()
    //             ),
    //             arguments: vec![
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(1u8))
    //                     .with_default_span(),
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(2u8))
    //                     .with_default_span(),
    //             ],
    //         })
    //         .with_default_span()
    //     );
    // }

    // #[test]
    // fn get_text_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::GET_PROPERTY_TEXT as u8,
    //         3, // length 3
    //         b'a',
    //         b'b',
    //         b'c',
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         42,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAccess(PropertyAccess {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
    //                     .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Text("abc".to_string())
    //                     .with_default_span()
    //             ),
    //         })
    //     );
    // }

    // #[test]
    // fn set_text_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::SET_PROPERTY_TEXT as u8,
    //         3, // length 3
    //         b'x',
    //         b'y',
    //         b'z',
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         100,
    //         // base
    //         InstructionCode::UINT_8 as u8,
    //         200,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAssignment(PropertyAssignment {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(
    //                     200u8
    //                 ))
    //                 .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Text("xyz".to_string())
    //                     .with_default_span()
    //             ),
    //             operator: None,
    //             assigned_expression: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(
    //                     100u8
    //                 ))
    //                 .with_default_span(),
    //             ),
    //         })
    //     );
    // }

    // #[test]
    // fn get_index_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::GET_PROPERTY_INDEX as u8,
    //         5, // index 5
    //         0,
    //         0,
    //         0,
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         42,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAccess(PropertyAccess {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
    //                     .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Integer(Integer::from(5u8))
    //                     .with_default_span()
    //             ),
    //         })
    //     );
    // }

    // #[test]
    // fn set_index_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::SET_PROPERTY_INDEX as u8,
    //         10, // index 10
    //         0,
    //         0,
    //         0,
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         150,
    //         // base
    //         InstructionCode::UINT_8 as u8,
    //         250,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAssignment(PropertyAssignment {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(
    //                     250u8
    //                 ))
    //                 .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Integer(Integer::from(10u8))
    //                     .with_default_span()
    //             ),
    //             operator: None,
    //             assigned_expression: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(
    //                     150u8
    //                 ))
    //                 .with_default_span(),
    //             ),
    //         })
    //     );
    // }

    // #[test]
    // fn get_dynamic_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::GET_PROPERTY_DYNAMIC as u8,
    //         // property
    //         InstructionCode::SHORT_TEXT as u8,
    //         4, // length 4
    //         b'n',
    //         b'a',
    //         b'm',
    //         b'e',
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         42,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAccess(PropertyAccess {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(42u8))
    //                     .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Text("name".to_string())
    //                     .with_default_span()
    //             ),
    //         })
    //     );
    // }

    // #[test]
    // fn set_dynamic_property() {
    //     let bytecode: Vec<u8> = vec![
    //         InstructionCode::SET_PROPERTY_DYNAMIC as u8,
    //         // property
    //         InstructionCode::SHORT_TEXT as u8,
    //         3, // length 3
    //         b'a',
    //         b'g',
    //         b'e',
    //         // value
    //         InstructionCode::UINT_8 as u8,
    //         30,
    //         // base
    //         InstructionCode::UINT_8 as u8,
    //         100,
    //     ];
    //     let ast = ast_from_bytecode(&bytecode).unwrap();
    //     assert_eq!(
    //         ast.data,
    //         DatexExpressionData::PropertyAssignment(PropertyAssignment {
    //             base: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(
    //                     100u8
    //                 ))
    //                 .with_default_span()
    //             ),
    //             property: (
    //                 DatexExpressionData::Text("age".to_string())
    //                     .with_default_span()
    //             ),
    //             operator: None,
    //             assigned_expression: (
    //                 DatexExpressionData::TypedInteger(TypedInteger::from(30u8))
    //                     .with_default_span(),
    //             ),
    //         })
    //     );
    // }
}
