use core::ops::Range;

use pretty::DocAllocator;

use crate::{
    ast::expressions::{
        BinaryOperation, DataExpression, DataExpressionData, List, Map,
        VariableAccess, VariableDeclaration, expression::Expression,
    },
    fmt::{
        Assoc, Format, Formatter, Operation, ParentContext,
        options::{StatementFormatting, VariantFormatting},
    },
    prelude::*,
    shared_values::ReferenceMutability,
    types::type_definition_with_metadata::LocalReferenceMutability,
    values::core_values::{
        decimal::typed_decimal::TypedDecimal,
        integer::typed_integer::TypedInteger,
    },
};

impl<'a> Formatter<'a> {
    pub fn datex_expression_to_source_code(
        &'a self,
        expr: &'a DataExpression,
    ) -> Format<'a> {
        let a = &self.alloc;
        match expr.data() {
            DataExpressionData::Integer(i) => a.as_string(i),
            DataExpressionData::TypedInteger(ti) => {
                self.typed_integer_to_source_code(ti, &expr.span)
            }
            DataExpressionData::Decimal(d) => a.as_string(d),
            DataExpressionData::TypedDecimal(td) => {
                self.typed_decimal_to_source_code(td, &expr.span)
            }
            DataExpressionData::Boolean(b) => a.as_string(b),
            DataExpressionData::Text(t) => self.text_to_source_code(t.as_str()),
            DataExpressionData::Endpoint(e) => a.text(e.to_string()),
            DataExpressionData::Null => a.text("null"),
            DataExpressionData::Identifier(_l) => unreachable!(
                "Identifiers should have been resolved before formatting"
            ),
            DataExpressionData::Map(map) => self.map_to_source_code(map),
            DataExpressionData::List(list) => self.list_to_source_code(list),
            DataExpressionData::DeriveRef(create_ref) => {
                (match create_ref.mutability {
                    LocalReferenceMutability::Immutable => a.text("&"),
                    LocalReferenceMutability::Mutable => a.text("&mut "),
                }) + self.format_datex_expression(&create_ref.expression)
            }
            DataExpressionData::DeriveSharedRef(create_shared_ref) => {
                (match create_shared_ref.mutability {
                    ReferenceMutability::Immutable => a.text("'"),
                    ReferenceMutability::Mutable => a.text("'mut "),
                }) + self.format_datex_expression(&create_shared_ref.expression)
            }
            DataExpressionData::BinaryOperation(BinaryOperation {
                operator,
                left,
                right,
                ..
            }) => {
                let (precedence, associativity, _is_assoc) =
                    self.binary_operator_info(operator);

                // format children with parent context so they can decide about parens themselves
                let left_doc = self.format_datex_expression_with_parent(
                    left,
                    Some(ParentContext {
                        precedence,
                        associativity,
                        operation: Operation::Binary(operator),
                    }),
                    true,
                );
                let right_doc = self.format_datex_expression_with_parent(
                    right,
                    Some(ParentContext {
                        precedence,
                        associativity,
                        operation: Operation::Binary(operator),
                    }),
                    false,
                );

                let a = &self.alloc;
                (left_doc
                    + self.operator_with_spaces(a.text(operator.to_string()))
                    + right_doc)
                    .group()
            }
            DataExpressionData::Statements(statements) => {
                let is_terminated = statements.is_terminated;
                let docs: Vec<_> = statements
                    .statements
                    .iter()
                    .enumerate()
                    .map(|(i, stmt)| {
                        self.format_datex_expression_with_parent(
                            stmt,
                            Some(ParentContext {
                                precedence: 0,
                                associativity: Assoc::None,
                                operation: Operation::Statements,
                            }),
                            false,
                        ) + (if is_terminated
                            || i < statements.statements.len() - 1
                        {
                            a.text(";")
                        } else {
                            self.alloc.nil()
                        })
                    })
                    .collect();

                let joined = a.intersperse(
                    docs,
                    match self.options.statement_formatting {
                        StatementFormatting::NewlineBetween => a.hardline(),
                        StatementFormatting::SpaceBetween => a.space(),
                        StatementFormatting::Compact => a.nil(),
                    },
                );
                joined.group()
            }
            DataExpressionData::VariableDeclaration(VariableDeclaration {
                id: _,
                init_expression,
                kind,
                name,
                type_annotation,
            }) => {
                let type_annotation_doc =
                    if let Some(type_annotation) = type_annotation {
                        self.type_declaration_colon()
                            + self.format_type_expression(type_annotation)
                    } else {
                        a.nil()
                    };
                a.text(kind.to_string())
                    + a.space()
                    + a.text(name)
                    + type_annotation_doc
                    + self.operator_with_spaces(a.text("="))
                    + self.format_datex_expression(init_expression)
            }
            DataExpressionData::TypeExpression(type_expr) => {
                let a = &self.alloc;
                let inner = self.format_type_expression(type_expr);
                (a.text("type<") + a.line_() + inner + a.line_() + a.text(">"))
                    .group()
            }
            DataExpressionData::VariableAccess(VariableAccess {
                name, ..
            }) => a.text(name),
            e => panic!("Formatter not implemented for {:?}", e),
        }
    }

    /// Formats a typed integer into source code representation based on variant formatting options.
    fn typed_integer_to_source_code(
        &'a self,
        ti: &'a TypedInteger,
        span: &'a Range<usize>,
    ) -> Format<'a> {
        let a = &self.alloc;
        match self.options.variant_formatting {
            VariantFormatting::KeepAll => a.text(self.tokens_at(span)),
            VariantFormatting::WithSuffix => a.text(ti.to_string_with_suffix()),
            VariantFormatting::WithoutSuffix => a.text(ti.to_string()),
        }
    }

    /// Formats a typed decimal into source code representation based on variant formatting options.
    fn typed_decimal_to_source_code(
        &'a self,
        td: &'a TypedDecimal,
        span: &'a Range<usize>,
    ) -> Format<'a> {
        let a = &self.alloc;
        match self.options.variant_formatting {
            VariantFormatting::KeepAll => a.text(self.tokens_at(span)),
            VariantFormatting::WithSuffix => a.text(td.to_string_with_suffix()),
            VariantFormatting::WithoutSuffix => a.text(td.to_string()),
        }
    }

    /// Formats a list into source code representation.
    fn list_to_source_code(&'a self, elements: &'a List) -> Format<'a> {
        self.wrap_collection(
            elements
                .items
                .iter()
                .map(|e| self.format_datex_expression(e)),
            ("[", "]"),
            ",",
        )
    }

    /// Formats a string into source code representation.
    fn text_to_source_code(&'a self, s: &'a str) -> Format<'a> {
        self.alloc.text(format!("{:?}", s)) // quoted string
    }

    /// Formats a map into source code representation.
    fn map_to_source_code(&'a self, map: &'a Map) -> Format<'a> {
        let a = &self.alloc;
        let entries = map.entries.iter().map(|(key, value)| {
            self.format_datex_expression(key)
                + a.text(":")
                + (self.options.space_in_collection.then(|| a.space()))
                + self.format_datex_expression(value)
        });
        self.wrap_collection(entries, ("{", "}"), ",")
    }
}
