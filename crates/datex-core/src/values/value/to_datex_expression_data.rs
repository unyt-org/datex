use crate::{
    ast::{
        expressions::{
            DatexExpressionData,
        },
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{
        Value,
    },
};

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        match self {
            Value::Core(core_value) => core_value.to_datex_expression_data(),
            Value::Native(native_value) => native_value.to_datex_expression_data(),
        }
    }
}