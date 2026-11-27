pub macro yield_unwrap($expr:expr) {
    match $expr {
        Ok(val) => val,
        Err(e) => return yield Err(e),
    }
}
