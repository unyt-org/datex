use crate::values::core_value::CoreValue;
use crate::values::core_values::callable::Callable;

pub mod list;
pub mod map;

pub fn get_method(value: &CoreValue, name: &str) -> Option<Callable> {
    match value {
        CoreValue::List(_) => list::get_list_method(name),
        CoreValue::Map(_) => map::get_map_method(name),
        _ => None,
    }
}
