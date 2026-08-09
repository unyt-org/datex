use core::ops::DerefMut;
use datex_core::{
    datex_proxy::DatexProxyTypes,
    decompiler::{DecompileOptions, decompile_value},
    runtime::Runtime,
};
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug, Clone, PartialEq)]
// TODO: #[structural], nominal default
struct Example {
    a: u8,
    b: u8,
}

#[datex]
impl Example {
    pub fn set_a(&mut self, a: u8) {
        self.a = a;
    }
}

#[test]
fn impl_functions() {
    let runtime = Runtime::stub();
    let mut memory = runtime.memory().borrow_mut();

    let example_type = Example::datex_type(memory.deref_mut());
    println!(
        "{}",
        decompile_value(
            &example_type.into(),
            DecompileOptions::colorized_pretty()
        )
    );

    let mut example = Example { a: 1, b: 2 };
    example.set_a(2);
}
