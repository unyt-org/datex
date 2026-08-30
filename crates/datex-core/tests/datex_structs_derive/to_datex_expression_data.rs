//! This mod tests the behavior of the `ToDatexExpressionData` derive macro, which is used to convert Rust structs into Datex expression data.

use datex_core::{
    ast::{
        expressions::{DatexExpressionData, Map},
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::{core_values::integer::typed_integer::TypedInteger, value::Value},
};
use datex_macros_internal::Datex;

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(only_structural)]
struct TestStruct {
    pub field1: i32,
    pub field2: String,
}

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(only_structural)]
struct TestStructWithValue {
    pub field1: i32,
    pub field2: Value,
}

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(only_structural)]
struct TestStructNested {
    pub nested: TestStruct,
    pub field3: String,
}

#[test]
fn test_to_datex_expression_data() {
    let test_instance = TestStruct {
        field1: 42,
        field2: "Hello, Datex!".to_string(),
    };

    let datex_expression_data = test_instance.to_datex_expression_data();

    assert_eq!(
        datex_expression_data,
        DatexExpressionData::Map(Map::new(vec![
            (
                DatexExpressionData::Text("field1".into()).with_default_span(),
                DatexExpressionData::TypedInteger(TypedInteger::I32(42))
                    .with_default_span()
            ),
            (
                DatexExpressionData::Text("field2".into()).with_default_span(),
                DatexExpressionData::Text("Hello, Datex!".into())
                    .with_default_span()
            ),
        ]))
    );
}

#[test]
fn test_to_datex_expression_data_with_value() {
    let test_instance = TestStructWithValue {
        field1: 42,
        field2: Value::from("Hello, Datex!".to_string()),
    };

    let datex_expression_data = test_instance.to_datex_expression_data();

    assert_eq!(
        datex_expression_data,
        DatexExpressionData::Map(Map::new(vec![
            (
                DatexExpressionData::Text("field1".into()).with_default_span(),
                DatexExpressionData::TypedInteger(TypedInteger::I32(42))
                    .with_default_span()
            ),
            (
                DatexExpressionData::Text("field2".into()).with_default_span(),
                DatexExpressionData::Text("Hello, Datex!".into())
                    .with_default_span()
            ),
        ]))
    );
}

#[test]
fn test_to_datex_expression_data_nested() {
    let test_instance = TestStructNested {
        nested: TestStruct {
            field1: 42,
            field2: "Hello, Datex!".to_string(),
        },
        field3: "Nested Test".to_string(),
    };

    let datex_expression_data = test_instance.to_datex_expression_data();

    assert_eq!(
        datex_expression_data,
        DatexExpressionData::Map(Map::new(vec![
            (
                DatexExpressionData::Text("nested".into()).with_default_span(),
                DatexExpressionData::Map(Map::new(vec![
                    (
                        DatexExpressionData::Text("field1".into())
                            .with_default_span(),
                        DatexExpressionData::TypedInteger(TypedInteger::I32(
                            42
                        ))
                        .with_default_span()
                    ),
                    (
                        DatexExpressionData::Text("field2".into())
                            .with_default_span(),
                        DatexExpressionData::Text("Hello, Datex!".into())
                            .with_default_span()
                    ),
                ]))
                .with_default_span()
            ),
            (
                DatexExpressionData::Text("field3".into()).with_default_span(),
                DatexExpressionData::Text("Nested Test".into())
                    .with_default_span()
            ),
        ]))
    );
}
