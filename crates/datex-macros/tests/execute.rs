use datex_macros::execute;

#[test]
fn test_execute() {
    let x = 42;
    let tokens = execute!("1 + ?", x);
	println!("{:?}", tokens);
}
