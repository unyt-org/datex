use crate::prelude::*;
use crate::runtime::execution::ExecutionError;
use crate::values::core_value::CoreValue;
use crate::values::core_values::callable::*;
use crate::values::value_container::ValueContainer;
use crate::values::core_values::list::List;

pub fn get_list_method(name: &str) -> Option<Callable> {
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
            body: CallableBody::Native(list_len_impl),
            bound_this: None,
        }),
        "sort" => Some(Callable {
            name: Some("sort".to_string()),
            signature: CallableSignature {
                kind: CallableKind::Procedure,
                parameter_types: vec![],
                rest_parameter_type: None,
                return_type: None,
                yeet_type: None,
            },
            body: CallableBody::Native(list_sort_impl),
            bound_this: None,
        }),
        _ => None,
    }
}

fn list_len_impl(args: &[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() { return Err(ExecutionError::InvalidApply); }

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

fn list_sort_impl(args: &[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError> {
    if args.is_empty() { return Err(ExecutionError::InvalidApply); }

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
