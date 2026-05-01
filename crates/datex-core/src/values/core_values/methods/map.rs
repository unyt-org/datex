use crate::prelude::*;
use crate::runtime::execution::ExecutionError;
use crate::values::core_value::CoreValue;
use crate::values::core_values::callable::*;
use crate::values::value_container::ValueContainer;
use crate::values::core_values::list::List;

pub fn get_map_method(name: &str) -> Option<Callable> {
    match name {
        "len" => Some(Callable {
            name: Some("len".to_string()),
            signature: CallableSignature {
                kind: CallableKind::Function,
                parameter_types: vec![],
                rest_parameter_type: None,
                return_type: None,
                yeet_type: None,
            },
            body: CallableBody::Native(map_len_impl),
            bound_this: None,
        }),
        "keys" => Some(Callable {
            name: Some("keys".to_string()),
            signature: CallableSignature {
                kind: CallableKind::Function,
                parameter_types: vec![],
                rest_parameter_type: None,
                return_type: None,
                yeet_type: None,
            },
            body: CallableBody::Native(map_keys_impl),
            bound_this: None,
        }),
        "values" => Some(Callable {
            name: Some("values".to_string()),
            signature: CallableSignature {
                kind: CallableKind::Function,
                parameter_types: vec![],
                rest_parameter_type: None,
                return_type: None,
                yeet_type: None,
            },
            body: CallableBody::Native(map_values_impl),
            bound_this: None,
        }),
        _ => None,
    }
}

fn map_len_impl(args: &[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() { return Err(ExecutionError::InvalidApply); }
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

fn map_keys_impl(args: &[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() { return Err(ExecutionError::InvalidApply); }
    let target = &args[0];
    let keys: Vec<ValueContainer> = if let Some(shared) = target.maybe_shared() {
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

fn map_values_impl(args: &[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() { return Err(ExecutionError::InvalidApply); }
    let target = &args[0];
    let values: Vec<ValueContainer> = if let Some(shared) = target.maybe_shared() {
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
