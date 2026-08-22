use crate::ast::expressions::{DatexExpressionData};

pub trait ToDatexExpressionData {
    /// Converts the implementing type into a [DatexExpressionData] representation.
    fn to_datex_expression_data(&self) -> DatexExpressionData;
}


// TODO: move tests
#[cfg(test)]
mod tests {
    use crate::{
        alloc::boxed::Box,
        ast::{
            expressions::{DatexExpressionData, List, RangeDeclaration},
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

    use crate::prelude::*;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;

    #[test]
    fn integer_to_ast() {
        let value = ValueContainer::from(Integer::from(42));
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, DatexExpressionData::Integer(Integer::from(42)));
    }

    #[test]
    fn typed_integer_to_ast() {
        let value = ValueContainer::from(TypedInteger::from(42i8));
        let ast = value.to_datex_expression_data();
        assert_eq!(
            ast,
            DatexExpressionData::TypedInteger(TypedInteger::from(42i8))
        );
    }

    #[test]
    fn decimal_to_ast() {
        let value = ValueContainer::from(Decimal::from(1.23));
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, DatexExpressionData::Decimal(Decimal::from(1.23)));
    }

    #[test]
    fn typed_decimal_to_ast() {
        let value = ValueContainer::from(TypedDecimal::from(2.71f32));
        let ast = value.to_datex_expression_data();
        assert_eq!(
            ast,
            DatexExpressionData::TypedDecimal(TypedDecimal::from(2.71f32))
        );
    }

    #[test]
    fn boolean_to_ast() {
        let value = ValueContainer::from(true);
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, DatexExpressionData::Boolean(true.into()));
    }

    #[test]
    fn text_to_ast() {
        let value = ValueContainer::from("Hello, World!".to_string());
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, DatexExpressionData::Text("Hello, World!".into()));
    }

    #[test]
    fn null_to_ast() {
        let value = ValueContainer::Local(Value::null());
        let ast = value.to_datex_expression_data();
        assert_eq!(ast, DatexExpressionData::Null);
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
            DatexExpressionData::List(List::new(vec![
                DatexExpressionData::Integer(Integer::from(1))
                    .with_default_span(),
                DatexExpressionData::Integer(Integer::from(2))
                    .with_default_span(),
                DatexExpressionData::Integer(Integer::from(3))
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
            DatexExpressionData::Range(RangeDeclaration {
                start: (DatexExpressionData::Integer(Integer::from(11))
                    .with_default_span()),
                end: (DatexExpressionData::Integer(Integer::from(13))
                    .with_default_span()),
            })
        );
    }
}
