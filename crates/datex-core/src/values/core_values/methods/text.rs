use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    values::{
        core_value::CoreValue,
        core_values::{callable::*, text::Text, r#type::Type},
        value_container::ValueContainer,
    },
};

pub fn get_text_method(name: &str) -> Option<Callable> {
    match name {
        "reverse" => Some(Callable::method_with_return(
            "reverse",
            CallableKind::Procedure,
            text_reverse_impl,
            Type::text(),
        )),
        _ => None,
    }
}

fn text_reverse_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }

    let target = &args[0];
    if let Some(shared) = target.maybe_shared() {
        shared.with_value_unchecked(|val| {
            if let CoreValue::Text(l) = &mut val.inner {
                l.reverse();
            }
        });
    } else {
        return Err(ExecutionError::InvalidApply);
    }

    Ok(None)
}
