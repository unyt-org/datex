use crate::pipeline_tests::setup::{input, output, validate_pipeline};
use datex_core::{
    global::protocol_structures::{
        instruction_data::UInt8Data, regular_instructions::RegularInstruction,
    },
    values::{
        core_values::integer::typed_integer::TypedInteger,
        value_container::ValueContainer,
    },
};

#[cfg(feature = "ast")]
use datex_core::ast::expressions::DatexExpressionData;
use datex_core::parser::lexer::Token;

#[test]
fn integer_u8() {
    // source code input "42u8" produces corresponding DATEX value after execution
    validate_pipeline(
        input().source_code("42u8"),
        output().datex_value(Some(&ValueContainer::from(TypedInteger::U8(42)))),
    );

    // For any of the given input variants, all the expected outputs must be produced
    validate_pipeline(
        input()
            .source_code("42u8")
            .datex_value(TypedInteger::U8(42))
            .rust_value(42u8),
        output()
            .tokens(&[Token::IntegerLiteral("42u8".to_string())])
            .ast(DatexExpressionData::TypedInteger(TypedInteger::U8(42)))
            .instructions(RegularInstruction::UInt8(UInt8Data(42)))
            .source_code_same_as_input()
            .rust_value_same_as_input()
            .datex_value_same_as_input(),
    );

    // source code input "42u8" produces same decompiled output after execution
    validate_pipeline(
        input().source_code("42u8"),
        output().source_code_same_as_input(),
    );

    // Rust value 42u8 produces corresponding DATEX value after execution
    validate_pipeline(
        input().rust_value(42u8),
        output().datex_value(Some(&ValueContainer::from(TypedInteger::U8(42)))),
    );
}
