use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    values::{
        core_value::CoreValue,
        core_values::{callable::*, list::List, r#type::Type},
        value_container::ValueContainer,
    },
};

pub fn get_list_method(name: &str) -> Option<Callable> {
    match name {
        "len" => Some(Callable::method_with_return(
            "len",
            CallableKind::Function,
            list_len_impl,
            Type::integer(),
        )),
        "sort" => Some(Callable::method(
            "sort",
            CallableKind::Procedure,
            list_sort_impl,
        )),
        _ => None,
    }
}

fn list_len_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }

    let target = &args[0];
    let len = if let Some(shared) = target.maybe_shared() {
        shared.with_value_unchecked(|val| {
            if let CoreValue::List(l) = &val.inner {
                l.len() as f64
            } else {
                0.0
            }
        })
    } else if let ValueContainer::Local(l) = target {
        if let CoreValue::List(l) = &l.inner {
            l.len() as f64
        } else {
            return Err(ExecutionError::InvalidApply);
        }
    } else {
        return Err(ExecutionError::InvalidApply);
    };

    Ok(Some(ValueContainer::from(len)))
}

fn list_sort_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }

    let target = &args[0];
    if let Some(shared) = target.maybe_shared() {
        shared.with_value_unchecked(|val| {
            if let CoreValue::List(l) = &mut val.inner {
                l.sort_by_to_string();
            }
        });
    } else {
        return Err(ExecutionError::InvalidApply);
    }

    Ok(None)
}
