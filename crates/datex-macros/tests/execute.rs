use datex_macros::execute_sync;

#[test]
fn execute_sync() {
    let x = 42;
    let tokens = execute_sync!("1 + ?", x);
    println!("{:?}", tokens);
}

#[tokio::test]
async fn execute_async() {
    let x = 42;
    let tokens = datex_macros::execute!("1 + ?", x).await;
    println!("{:?}", tokens);
}
