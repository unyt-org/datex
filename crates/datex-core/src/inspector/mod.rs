use crate::{
    datex_proxy::DatexProxyTypes,
    prelude::*,
    runtime::Runtime,
    types::type_definition::callable::CallableKind,
    values::{
        core_values::callable::native_sync_callable,
        value_container::ValueContainer,
    },
};
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug)]
pub struct Inspector {
    name: String, // TODO: We must make private properties to be ignore by the type definition and only use public ones, otherwise the prop and methods would colide in DATEX
}

#[datex]
impl Inspector {
    /// Creates a new Inspector instance.
    pub fn create(name: String) -> Self {
        Inspector { name }
    }
    pub fn name_getter(&self) -> String {
        // FIXME allow &str (lifetime issues)
        // TODO self arg
        self.name.clone()
    }
}

/// Registers the `inspector` namespace in the runtime, allowing users to create Inspector instances.
pub fn register_inspector_namespace(runtime: &Runtime) {
    let mut memory = runtime.memory().borrow_mut();
    let inspector_create_callable = ValueContainer::from(native_sync_callable(
        Inspector::create,
        Some("create".to_string()),
        CallableKind::Procedure,
        &mut memory,
    ));

    runtime
        .endpoint_properties_mut()
        .insert("inspector".to_string(), inspector_create_callable);
}

#[cfg(test)]
mod tests {
    use crate::{
        inspector,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        traits::apply::Apply, values::core_values::map::Map,
    };

    #[test]
    fn ty() {
        let inspector_type = inspector::Inspector::datex_type(
            &mut SharedReferencesCache::default(),
        );
        let r = native_sync_callable(
            Inspector::create,
            Some("create".to_string()),
            CallableKind::Procedure,
            &mut SharedReferencesCache::default(),
        );

        let inspector = Inspector::create("Test Inspector".to_string());
        let vc = ValueContainer::from(inspector);

        // TODO: methods bound to value container / even better, methods bound to the type definition
        // We must the store type in value container (this will require passing cache to the ValueContainer::from function somehow)
        // Then we can access methods on the type definition here
        let runtime = crate::runtime::Runtime::stub();
        if let Some(method) = vc
            .try_as::<Map>()
            .and_then(|map| map.try_get("name_getter").ok())
        {
            let result = method
                .try_apply_sync(&runtime, vec![])
                .unwrap()
                .expect("Method should return a value");
            assert_eq!(
                result.try_into_value::<String>().unwrap(),
                "Test Inspector"
            );
        } else {
            panic!("Method not found");
        }
    }

    use super::*;
    #[test]
    fn test_function() {
        let runtime = Runtime::stub();
        let mut memory = SharedReferencesCache::default();
        // 1 arg
        let func = |x: u8| x + 1;
        let dx_func_1 = ValueContainer::from(native_sync_callable(
            func,
            None,
            CallableKind::Function,
            &mut memory,
        ));
        let res = dx_func_1
            .try_apply_sync(&runtime, vec![4u8.into()])
            .unwrap()
            .expect("Function should return a value");
        assert_eq!(res, 5u8.into());

        // 2 args
        let func_2 = |x: u8, y: u8| x + y;
        let dx_func_2 = ValueContainer::from(native_sync_callable(
            func_2,
            None,
            CallableKind::Function,
            &mut memory,
        ));
        let res_2 = dx_func_2
            .try_apply_sync(&runtime, vec![3u8.into(), 4u8.into()])
            .unwrap()
            .expect("Function should return a value");
        assert_eq!(res_2, 7u8.into());
    }
}
