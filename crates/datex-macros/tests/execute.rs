use datex_core::values::core_values::integer::{
    Integer, typed_integer::TypedInteger,
};
use datex_macros::{
    execute, execute_sync, execute_sync_unchecked, execute_unchecked,
};

#[test]
fn execute_sync() {
    let x = 42;
    let result = execute_sync_unchecked!("1 + ?", x).unwrap();
    assert_eq!(result, Integer::new(43).into());

    let result = execute_sync_unchecked!("1 + ?", 42).unwrap();
    assert_eq!(result, Integer::new(43).into());

    let result = execute_sync_unchecked!("? + ?", x, 42).unwrap();
    assert_eq!(result, 84.into());

    let result =
        execute_sync_unchecked!("? + ' ' + ? + '!'", "Hello,", "DATEX")
            .unwrap();
    assert_eq!(result, "Hello, DATEX!".into());
}

#[tokio::test]
async fn execute_async() {
    let x = 42;
    let result = execute_unchecked!("1 + ?", x).await.unwrap();
    assert_eq!(result, Integer::new(43).into());
}
