use datex_macros::precompile;

#[test]
fn test_precompile() {
    let tokens = precompile!("1 + ?", 2);
    println!("{}", tokens);
}
