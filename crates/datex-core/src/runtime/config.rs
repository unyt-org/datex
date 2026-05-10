use crate::{
    collections::HashMap,
    datex_proxy::DatexValueContainerProxy,
    network::com_hub::InterfacePriority,
    prelude::*,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};
use crate::datex_proxy::{DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize, DatexValueContainerProxySerialize, DatexValueProxyInfallibleSerialize, DatexValueProxySerialize, TryToDatexValueError};
use crate::values::core_values::map::Map;
use crate::values::value::Value;


pub fn is_priority_none(v: &InterfacePriority) -> bool {
    matches!(v, InterfacePriority::None)
}

#[derive(Datex, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[datex(allow_serde_infallible)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct RuntimeConfigInterface {
    // #[serde(rename = "type")]
    pub interface_type: String,
    // #[serde(rename = "config")]
    #[cfg_attr(feature = "wasm_runtime", tsify(type = "unknown"))]
    pub setup_data: Value,

    // #[serde(default, skip_serializing_if = "is_priority_none")]
    pub priority: InterfacePriority,
}

impl RuntimeConfigInterface {
    pub fn new<T: DatexValueProxySerialize>(
        interface_type: &str,
        setup_data: T,
    ) -> Result<RuntimeConfigInterface, String> {
        Ok(RuntimeConfigInterface {
            interface_type: interface_type.to_string(),
            priority: InterfacePriority::default(),
            setup_data: setup_data.try_to_value().map_err(|e| {
                format!(
                    "Failed to convert setup_data to ValueContainer: {:?}",
                    e
                )
            })?.try_into().map_err(|e| {
                format!(
                    "Failed to convert ValueContainer to Map: {:?}",
                    e
                )
            })?
        })
    }

    pub fn new_from_map(
        interface_type: &str,
        config: Value,
    ) -> RuntimeConfigInterface {
        RuntimeConfigInterface {
            priority: InterfacePriority::default(),
            interface_type: interface_type.to_string(),
            setup_data: config,
        }
    }
}

#[derive(Datex, Debug, Default)]
pub struct RuntimeConfig {
    pub endpoint: Option<Endpoint>,
    pub interfaces: Option<Vec<RuntimeConfigInterface>>,
    pub env: Option<HashMap<String, String>>,
}

impl RuntimeConfig {
    pub fn new_with_endpoint(endpoint: Endpoint) -> Self {
        RuntimeConfig {
            endpoint: Some(endpoint),
            interfaces: None,
            env: None,
        }
    }

    pub fn add_interface<T: DatexValueProxyInfallibleSerialize>(
        &mut self,
        interface_type: String,
        config: T,
        priority: InterfacePriority,
    ) {
        let config = config.to_value();
        let interface = RuntimeConfigInterface {
            interface_type,
            setup_data: config,
            priority,
        };
        if let Some(interfaces) = &mut self.interfaces {
            interfaces.push(interface);
        } else {
            self.interfaces = Some(vec![interface]);
        }
    }

    /// Adds a single environment variable to the runtime's custom environment variables.
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.env.get_or_insert_with(HashMap::new).insert(key, value);
    }

    /// Adds multiple environment variables to the runtime's custom environment variables.
    pub fn add_env_vars(&mut self, vars: HashMap<String, String>) {
        self.env.get_or_insert_with(HashMap::new).extend(vars);
    }

    #[cfg(feature = "target_native")]
    /// Adds all host environment variables to the runtime's custom environment variables.
    pub fn load_host_env_vars(&mut self) {
        // add all host environment variables to the runtime's custom environment variables
        for (key, value) in std::env::vars() {
            self.env.get_or_insert_with(HashMap::new).insert(key, value);
        }
    }

    #[cfg(feature = "target_native")]
    /// Adds all environment variables from a .env file to the runtime's custom environment variables.
    pub fn add_env_vars_from_file(
        &mut self,
        path: &std::path::PathBuf,
    ) -> Result<(), dotenvy::Error> {
        let loader1 = dotenvy::from_path_iter(path)?;
        for item in loader1 {
            let (key, val) = item?;
            self.env.get_or_insert_with(HashMap::new).insert(key, val);
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use datex_macros_internal::Datex;

    use crate::{
        prelude::*,
        runtime::{RuntimeConfig, RuntimeConfigInterface},
        values::core_values::map::Map,
    };

    #[test]
    fn add_env_var() {
        let mut config = RuntimeConfig::default();
        config.add_env_var("KEY1".to_string(), "VALUE1".to_string());
        let env_vars = config.env.unwrap();
        assert_eq!(env_vars.get("KEY1"), Some(&"VALUE1".to_string()));
    }

    #[test]
    fn serde() {
        #[derive(Datex)]
        struct MySetupData {
            field1: String,
            field2: i32,
        }
        let config: RuntimeConfigInterface = RuntimeConfigInterface::new(
            "test",
            MySetupData {
                field1: "value".to_string(),
                field2: 42,
            },
        )
        .unwrap();
        assert_eq!(config.interface_type, "test");
        let setup_data = config.setup_data;
        let map = setup_data.try_as::<Map>().unwrap();
        assert_eq!(
            map.get("field1").unwrap().try_as::<String>().unwrap(),
            "value".to_string()
        );
        assert_eq!(map.get("field2").unwrap().try_as::<i32>().unwrap(), 42);
    }
}
