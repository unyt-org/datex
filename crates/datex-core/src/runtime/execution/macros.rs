/// Yield an interrupt and get the next input
macro_rules! interrupt {
    ($input:expr, $arg:expr) => {{
        yield Ok($arg);
        $input.take_result()
    }};
}
pub(crate) use interrupt;

/// Yield an interrupt and get the next resolved value or None
/// expecting the next input to be a ResolvedValue variant
macro_rules! interrupt_with_maybe_value {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt;

        let res = interrupt!($input, $arg).unwrap();
        match res {
            crate::runtime::execution::execution_loop::interrupts::InterruptResult::ResolvedValue(value) => value,
            _ => unreachable!(),
        }
    }};
}
pub(crate) use interrupt_with_maybe_value;

/// Yield an interrupt and get the next resolved value
/// expecting the next input to be a ResolvedValue variant with Some value
macro_rules! interrupt_with_value {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt_with_maybe_value;
        let maybe_value = interrupt_with_maybe_value!($input, $arg);
        if let Some(value) = maybe_value {
            value
        } else {
            unreachable!();
        }
    }};
}
pub(crate) use interrupt_with_value;

/// Yield an interrupt and get the next resolved values
/// expecting the next input to be a ResolvedValues variant
macro_rules! interrupt_with_values {
    ($input:expr, $arg:expr) => {{
        use crate::runtime::execution::macros::interrupt;
        let res = interrupt!($input, $arg).unwrap();
        match res {
            crate::runtime::execution::execution_loop::interrupts::InterruptResult::ResolvedValues(values) => values,
            _ => unreachable!(),
        }
    }};
}
