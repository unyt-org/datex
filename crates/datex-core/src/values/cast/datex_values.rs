// TODO: move tests
#[cfg(test)]
mod tests {
    use crate::values::{
        core_value::CoreValue, core_values::endpoint::Endpoint, value::Value,
    };

    #[test]
    fn to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native(endpoint.clone());
        assert!(matches!(
            value.try_into_core_value().unwrap(),
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_boxed_to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native(endpoint.clone());
        assert_eq!(
            *value.try_as::<Endpoint>().expect("Expected Endpoint"),
            endpoint
        );
    }

    #[test]
    fn try_from_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native(endpoint.clone());
        let result = value.try_into_value::<Endpoint>().unwrap();
        assert_eq!(result, endpoint);
    }
}
