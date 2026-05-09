use serde_json::{Number, Value};

use crate::values::{
    core_values::{list::List, map::Map},
    value::Value as DatexValue,
    value_container::ValueContainer,
};

impl From<Value> for ValueContainer {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => ValueContainer::Local(DatexValue::null()),
            Value::Bool(v) => ValueContainer::from(v),
            Value::Number(n) => {
                if let Some(v) = n.as_u64() {
                    ValueContainer::from(v)
                } else if let Some(v) = n.as_i64() {
                    ValueContainer::from(v)
                } else if let Some(v) = n.as_f64() {
                    ValueContainer::from(v)
                } else {
                    unreachable!()
                }
            }
            Value::String(v) => ValueContainer::from(v),
            Value::Array(values) => {
                let list = List::from(
                    values
                        .into_iter()
                        .map(ValueContainer::from)
                        .collect::<Vec<_>>(),
                );
                ValueContainer::from(list)
            }
            Value::Object(values) => {
                let map = Map::from(
                    values
                        .into_iter()
                        .map(|(key, value)| (key, ValueContainer::from(value)))
                        .collect::<Vec<_>>(),
                );
                ValueContainer::from(map)
            }
        }
    }
}

impl TryFrom<ValueContainer> for Value {
    type Error = ();

    fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
        if let Some(v) = value.try_as::<bool>() {
            return Ok(Value::Bool(v));
        }

        if let Some(v) = value.try_as::<u8>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<u16>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<u32>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<u64>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<i8>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<i16>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<i32>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<i64>() {
            return Ok(Value::Number(Number::from(v)));
        }

        if let Some(v) = value.try_as::<f32>() {
            let number = Number::from_f64(v as f64).ok_or(())?;
            return Ok(Value::Number(number));
        }

        if let Some(v) = value.try_as::<f64>() {
            let number = Number::from_f64(v).ok_or(())?;
            return Ok(Value::Number(number));
        }

        if let Some(v) = value.try_as::<String>() {
            return Ok(Value::String(v));
        }

        if let Some(list) = value.try_as::<List>() {
            let values = list
                .into_iter()
                .map(Value::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            return Ok(Value::Array(values));
        }

        if let Some(map) = value.try_as::<Map>() {
            let mut object = serde_json::Map::new();

            for (key, value) in map.into_iter() {
                // FIXME not converting to string
                object.insert(key.to_string(), Value::try_from(value)?);
            }

            return Ok(Value::Object(object));
        }

        // FIXME TBD

        // if let Some(v) = value.try_as::<char>() {
        //     return Ok(Value::String(v.to_string()));
        // }
        // if let Some(v) = value.try_as::<usize>() {
        //     return Ok(Value::Number(Number::from(v as u64)));
        // }
        // if let Some(v) = value.try_as::<isize>() {
        //     return Ok(Value::Number(Number::from(v as i64)));
        // }
        Err(())
    }
}
