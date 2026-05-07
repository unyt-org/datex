use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    values::{
        core_value::CoreValue,
        core_values::{callable::*, list::List},
        value_container::ValueContainer,
    },
};

pub fn get_map_method(name: &str) -> Option<Callable> {
    match name {
        "len" => Some(Callable::method(
            "len",
            CallableKind::Function,
            map_len_impl,
        )),
        "keys" => Some(Callable::method(
            "keys",
            CallableKind::Function,
            map_keys_impl,
        )),
        "values" => Some(Callable::method(
            "values",
            CallableKind::Function,
            map_values_impl,
        )),
        _ => None,
    }
}

fn map_len_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }
    let target = &args[0];
    let len = if let Some(shared) = target.maybe_shared() {
        shared.with_value_unchecked(|val| {
            if let CoreValue::Map(m) = &val.inner {
                m.size() as f64
            } else {
                0.0
            }
        })
    } else if let ValueContainer::Local(l) = target {
        if let CoreValue::Map(m) = &l.inner {
            m.size() as f64
        } else {
            return Err(ExecutionError::InvalidApply);
        }
    } else {
        return Err(ExecutionError::InvalidApply);
    };
    Ok(Some(ValueContainer::from(len)))
}

fn map_keys_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }
    let target = &args[0];
    let keys: Vec<ValueContainer> = if let Some(shared) = target.maybe_shared()
    {
        shared.with_value_unchecked(|val| {
            if let CoreValue::Map(m) = &val.inner {
                m.iter().map(|(k, _)| k.into()).collect()
            } else {
                vec![]
            }
        })
    } else if let ValueContainer::Local(l) = target {
        if let CoreValue::Map(m) = &l.inner {
            m.iter().map(|(k, _)| k.into()).collect()
        } else {
            return Err(ExecutionError::InvalidApply);
        }
    } else {
        return Err(ExecutionError::InvalidApply);
    };
    Ok(Some(ValueContainer::from(List::new(keys))))
}

fn map_values_impl(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() {
        return Err(ExecutionError::InvalidApply);
    }
    let target = &args[0];
    let values: Vec<ValueContainer> =
        if let Some(shared) = target.maybe_shared() {
            shared.with_value_unchecked(|val| {
                if let CoreValue::Map(m) = &val.inner {
                    m.iter().map(|(_, v)| v.clone()).collect()
                } else {
                    vec![]
                }
            })
        } else if let ValueContainer::Local(l) = target {
            if let CoreValue::Map(m) = &l.inner {
                m.iter().map(|(_, v)| v.clone()).collect()
            } else {
                return Err(ExecutionError::InvalidApply);
            }
        } else {
            return Err(ExecutionError::InvalidApply);
        };
    Ok(Some(ValueContainer::from(List::new(values))))
}
