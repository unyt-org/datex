use crate::traits::value_access::ValueAccess;

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use ordered_float::OrderedFloat;
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

    impl ToDatexExpressionData for f32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedDecimal(TypedDecimal::F32(OrderedFloat(*self)))
        }
    }
    
    impl ToDatexExpressionData for f64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedDecimal(TypedDecimal::F64(OrderedFloat(*self)))
        }
    }
}

impl ValueAccess for f32 {}
impl ValueAccess for f64 {}