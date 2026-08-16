use crate::{
    datex_proxy::DatexProxyTypes,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::r#type::Type,
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
};
use seq_macro::seq;

pub trait IntoDatexCallable<Args, R, C> {
    /// Returns a vector of tuples containing the parameter names (if any) and their corresponding [Type]s.
    fn parameters(context: &mut C) -> Vec<(Option<String>, Type)>;
    /// Invokes the callable with the provided arguments and returns a [Result] containing either the return value or a [CallableError].
    fn invoke(&self, args: Vec<ValueContainer>) -> Result<R, CallableError>;
}

macro_rules! impl_datex_callable {
    ($n:literal) => {
        seq!(N in 0..$n {
            impl<F, R, C, #(A~N,)*> IntoDatexCallable<(#(A~N,)*), R, C> for F
            where
                F: Fn(#(A~N,)*) -> R,
                #(
                    A~N: DatexProxyTypes<C> + TryFrom<ValueContainer>,
                )*
            {
                fn parameters(
                    #[allow(unused_variables)]
                    context: &mut C,
                ) -> Vec<(Option<String>, Type)> {
                    vec![#((None, A~N::datex_type(context)),)*]
                }
                fn invoke(
                    &self,
                    args: Vec<ValueContainer>,
                ) -> Result<R, CallableError> {
                    // check that number of args is correct
                    if args.len() != $n {
                        return Err(CallableError::InvalidSignature);
                    }
                    #[allow(unused)]
                    let mut args = args.into_iter();
                    #(
                        let arg~N: A~N = args
                            .next()
                            .unwrap()
                            .try_into()
                            .map_err(|_| CallableError::InvalidSignature)?;
                    )*
                    Ok(self(#(arg~N,)*))
                }
            }
        });
    };
}

impl_datex_callable!(0);
impl_datex_callable!(1);
impl_datex_callable!(2);
impl_datex_callable!(3);
impl_datex_callable!(4);
impl_datex_callable!(5);
impl_datex_callable!(6);
impl_datex_callable!(7);
impl_datex_callable!(8);
impl_datex_callable!(9);
impl_datex_callable!(10);

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*, runtime::execution::ExecutionError,
        traits::callable::IntoDatexCallable,
        values::core_values::callable::error::CallableError,
    };

    // FIXME
    // #[test]
    // fn simple_call() {
    //     let func = |x: u8, y: u8| x + y;
    //     let result = func.invoke(vec![1u8.into(), 2u8.into()]).unwrap();
    //     assert_eq!(result, 3);
    // }
    //
    // #[test]
    // fn order() {
    //     let func = |a: u8, b: u8, c: u8, d: u8, e: u8| vec![a, b, c, d, e];
    //     let result = func
    //         .invoke(vec![
    //             1u8.into(),
    //             2u8.into(),
    //             3u8.into(),
    //             4u8.into(),
    //             5u8.into(),
    //         ])
    //         .unwrap();
    //     assert_eq!(result, vec![1, 2, 3, 4, 5]);
    // }
    //
    // #[test]
    // fn invalid_signature_args_count() {
    //     let func = |x: u8, y: u8| x + y;
    //     // only one instead of two args
    //     let result = func.invoke(vec![1u8.into()]);
    //     assert!(matches!(result, Err(CallableError::InvalidSignature)));
    // }
    //
    // #[test]
    // fn invalid_signature_wrong_type() {
    //     let func = |x: u8, y: u8| x + y;
    //     // second arg is a text instead of u8
    //     let result = func.invoke(vec![1u8.into(), "test".into()]);
    //     assert!(matches!(result, Err(CallableError::InvalidSignature)));
    // }
    //
    // #[test]
    // fn error_result() {
    //     let func = |should_fail: bool| -> Result<u8, ExecutionError> {
    //         if should_fail {
    //             Err(ExecutionError::RequiresAsyncExecution)
    //         } else {
    //             Ok(42)
    //         }
    //     };
    //     let result = func.invoke(vec![true.into()]).unwrap();
    //     assert!(matches!(
    //         result,
    //         Err(ExecutionError::RequiresAsyncExecution)
    //     ));
    // }
}
