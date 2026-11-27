use crate::ast::expressions::NormalExpressionData;

/// This trait defines a method for converting any value into a [NormalExpressionData] representation.
/// It is implemented for all layers of values, e.g. [ValueContainer], [Value], [CoreValue], and native values.
pub trait ToDatexExpressionData {
    /// Converts the implementing type into a [NormalExpressionData] representation.
    fn to_datex_expression_data(&self) -> NormalExpressionData;
}

// TODO: move tests
#[cfg(test)]
mod tests {
    use crate::{
        alloc::boxed::Box,
        ast::{
            expressions::{NormalExpressionData, List, RangeDeclaration},
            spanned::Spanned,
        },
        values::{
            core_values::{
                decimal::{Decimal, typed_decimal::TypedDecimal},
                integer::{Integer, typed_integer::TypedInteger},
                range::Range,
            },
            value::Value,
            value_container::ValueContainer,
        },
    };

    use crate::{
        prelude::*, traits::to_datex_expression_data::ToDatexExpressionData,
    };

    #[test]
    fn integer_to_ast() {
        let value = ValueContainer::from(Integer::from(42));
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, NormalExpressionData::Integer(Integer::from(42)));
    }

    #[test]
    fn typed_integer_to_ast() {
        let value = ValueContainer::from(TypedInteger::from(42i8));
        let ast = value.to_datex_expression_data();
        assert_eq!(
            ast,
            NormalExpressionData::TypedInteger(TypedInteger::from(42i8))
        );
    }

    #[test]
    fn decimal_to_ast() {
        let value = ValueContainer::from(Decimal::from(1.23));
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, NormalExpressionData::Decimal(Decimal::from(1.23)));
    }

    #[test]
    fn typed_decimal_to_ast() {
        let value = ValueContainer::from(TypedDecimal::from(2.71f32));
        let ast = value.to_datex_expression_data();
        assert_eq!(
            ast,
            NormalExpressionData::TypedDecimal(TypedDecimal::from(2.71f32))
        );
    }

    #[test]
    fn boolean_to_ast() {
        let value = ValueContainer::from(true);
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, NormalExpressionData::Boolean(true.into()));
    }

    #[test]
    fn text_to_ast() {
        let value = ValueContainer::from("Hello, World!".to_string());
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, NormalExpressionData::Text("Hello, World!".into()));
    }

    #[test]
    fn null_to_ast() {
        let value = ValueContainer::Local(Value::null());
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, NormalExpressionData::Null);
    }

    #[test]
    fn list_to_ast() {
        let value = ValueContainer::from(vec![
            Integer::from(1),
            Integer::from(2),
            Integer::from(3),
        ]);
        let ast = value.to_datex_expression_data();
        assert_eq!(
            ast,
            NormalExpressionData::List(List::new(vec![
                NormalExpressionData::Integer(Integer::from(1))
                    .with_default_span(),
                NormalExpressionData::Integer(Integer::from(2))
                    .with_default_span(),
                NormalExpressionData::Integer(Integer::from(3))
                    .with_default_span(),
            ]))
        );
    }

    #[test]
    fn range_to_ast() {
        let range = ValueContainer::from(Range {
            start: Box::new(Integer::from(11).into()),
            end: Box::new(Integer::from(13).into()),
        });
        let ast = range.to_datex_expression_data();
        assert_eq!(
            ast,
            NormalExpressionData::Range(RangeDeclaration {
                start: (NormalExpressionData::Integer(Integer::from(11))
                    .with_default_span()),
                end: (NormalExpressionData::Integer(Integer::from(13))
                    .with_default_span()),
            })
        );
    }
}
