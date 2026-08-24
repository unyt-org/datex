/// Yield an interrupt and get the next input
pub(crate) macro interrupt {
    ($input:expr, $arg:expr) => {{
        yield Ok($arg);
        $input.take_result()
    }}
}

/// Yield an interrupt and get the next resolved value or None
/// expecting the next input to be a ResolvedValue variant
pub(crate) macro interrupt_with_maybe_value {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt;

        let res = interrupt!($input, $arg).unwrap();
        match res {
            crate::runtime::execution::execution_loop::interrupts::InterruptResult::ResolvedValue(value) => value,
            _ => unreachable!(),
        }
    }}
}

/// Yield an interrupt and get the returned borrowed args and the result value
pub(crate) macro interrupt_with_borrowed_args_and_maybe_result {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt;

        let res = interrupt!($input, $arg).unwrap();
        match res {
            crate::runtime::execution::execution_loop::interrupts::InterruptResult::ResolvedValueAndBorrowedArgs((value, args)) => (value, args),
            _ => unreachable!(),
        }
    }}
}

/// Yield an interrupt and get the next resolved value
/// expecting the next input to be a ResolvedValue variant with Some value
pub(crate) macro interrupt_with_value {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt_with_maybe_value;
        let maybe_value = interrupt_with_maybe_value!($input, $arg);
        if let Some(value) = maybe_value {
            value
        } else {
            unreachable!();
        }
    }}
}

/// Yield an interrupt and get the next resolved values
/// expecting the next input to be a ResolvedValues variant
#[allow(unused_macros)]
pub(crate) macro interrupt_with_values {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt;
        let res = interrupt!($input, $arg).unwrap();
        match res {
            crate::runtime::execution::execution_loop::interrupts::InterruptResult::ResolvedValues(values) => values,
            _ => unreachable!(),
        }
    }}
}
