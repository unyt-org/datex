use datex_macros::precompile;

#[test]
fn test_integer() {
    precompile!("1 + ?", 42);
    precompile!("1 + ?", 42u8);
    precompile!("1 + ?", 42u16);
    precompile!("1 + ?", 42u32);
    precompile!("1 + ?", 42u64);
    precompile!("1 + ?", 42u128);
    precompile!("1 + ?", 42i8);
    precompile!("1 + ?", 42i16);
    precompile!("1 + ?", 42i32);
    precompile!("1 + ?", 42i64);
    precompile!("1 + ?", 42i128);
}

#[test]
fn test_float() {
    precompile!("1.0 + ?", 42.0);
    precompile!("1.0 + ?", 42.0f32);
    precompile!("1.0 + ?", 42.0f64);
}

#[test]
fn test_bool() {
    precompile!("true == ?", true);
    precompile!("false == ?", false);
}

#[test]
fn test_string() {
    precompile!("\"Hello, ?!\"");
}

#[test]
fn test_mixed() {
    precompile!("? + ? == ?", 1, 2.0, true);
}
