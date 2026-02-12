use datex_core::values::core_values::integer::Integer;
use datex_macros::{execute_sync_unchecked, execute_unchecked};

#[test]
fn execute_sync() {
    let x: i32 = 42;
    let result = execute_sync_unchecked!("1 + ?", x).unwrap();
    assert_eq!(result, Integer::new(43).into());

    let result = execute_sync_unchecked!("1 + ?", 42).unwrap();
    assert_eq!(result, Integer::new(43).into());

    let result = execute_sync_unchecked!("? + ?", x, 42).unwrap();
    assert_eq!(result, 84_i32.into());

    let result =
        execute_sync_unchecked!("? + ' ' + ? + '!'", "Hello,", "DATEX")
            .unwrap();
    assert_eq!(result, "Hello, DATEX!".into());
}

#[tokio::test]
async fn execute_async() {
    let x: i32 = 42;
    let result = execute_unchecked!("1 + ?", x).await.unwrap();
    assert_eq!(result, Integer::new(43).into());
}
