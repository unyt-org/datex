// TODO: move tests
#[cfg(test)]
mod tests {
    use crate::{
        values::{
            core_value::CoreValue, core_values::endpoint::Endpoint,
            value::Value,
        },
    };

    #[test]
    fn to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_boxed_to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_from_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        let result: Endpoint = value.try_into().unwrap();
        assert_eq!(result, endpoint);
    }
}
