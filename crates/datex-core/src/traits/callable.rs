use crate::{
    datex_proxy::DatexProxyTypes,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::r#type::Type,
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
    prelude::*,
};

pub trait IntoDatexCallable<Args, R> {
    /// Returns a vector of tuples containing the parameter names (if any) and their corresponding [Type]s.
    fn parameters(
        memory: &mut SharedReferencesCache,
    ) -> Vec<(Option<String>, Type)>;
    /// Invokes the callable with the provided arguments and returns a [Result] containing either the return value or a [CallableError].
    fn invoke(&self, args: Vec<ValueContainer>) -> Result<R, CallableError>;
}

impl<F, R> IntoDatexCallable<(), R> for F
where
    F: Fn() -> R,
{
    fn parameters(
        _memory: &mut SharedReferencesCache,
    ) -> Vec<(Option<String>, Type)> {
        vec![]
    }

    fn invoke(&self, args: Vec<ValueContainer>) -> Result<R, CallableError> {
        if !args.is_empty() {
            return Err(CallableError::InvalidSignature);
        }
        Ok(self())
    }
}

impl<F, A, R> IntoDatexCallable<(A,), R> for F
where
    F: Fn(A) -> R,
    A: DatexProxyTypes + TryFrom<ValueContainer>,
{
    fn parameters(
        memory: &mut SharedReferencesCache,
    ) -> Vec<(Option<String>, Type)> {
        vec![(None, A::datex_type(memory))]
    }

    fn invoke(
        &self,
        mut args: Vec<ValueContainer>,
    ) -> Result<R, CallableError> {
        if args.len() != 1 {
            return Err(CallableError::InvalidSignature);
        }

        let a = args
            .pop()
            .unwrap()
            .try_into()
            .map_err(|_| CallableError::InvalidSignature)?;
        Ok(self(a))
    }
}
impl<F, A, B, R> IntoDatexCallable<(A, B), R> for F
where
    F: Fn(A, B) -> R,
    A: DatexProxyTypes + TryFrom<ValueContainer>,
    B: DatexProxyTypes + TryFrom<ValueContainer>,
{
    fn parameters(
        memory: &mut SharedReferencesCache,
    ) -> Vec<(Option<String>, Type)> {
        vec![(None, A::datex_type(memory)), (None, B::datex_type(memory))]
    }

    fn invoke(
        &self,
        mut args: Vec<ValueContainer>,
    ) -> Result<R, CallableError> {
        if args.len() != 2 {
            return Err(CallableError::InvalidSignature);
        }
        let b = args
            .pop()
            .unwrap()
            .try_into()
            .map_err(|_| CallableError::InvalidSignature)?;
        let a = args
            .pop()
            .unwrap()
            .try_into()
            .map_err(|_| CallableError::InvalidSignature)?;
        Ok(self(a, b))
    }
}

// TODO macro and support more than 2 params here
