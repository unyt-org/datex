use crate::{
    collections::HashMap,
    datex_proxy::{
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
    },
    network::com_hub::InterfacePriority,
    prelude::*,
    values::{core_values::endpoint::Endpoint, value::Value},
};
use datex_macros_internal::Datex;

pub fn is_priority_none(v: &InterfacePriority) -> bool {
    matches!(v, InterfacePriority::None)
}

#[derive(Datex, Debug, Clone, PartialEq, Eq)]
/// A generic interface configuration to setup a runtime interface.
pub struct RuntimeConfigInterface {
    #[datex(rename = "type")]
    pub interface_type: String,
    pub config: Value,
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
            config: setup_data.try_to_value().map_err(|e| {
                format!(
                    "Failed to convert setup_data to ValueContainer: {:?}",
                    e
                )
            })?,
        })
    }

    pub fn new_from_map(
        interface_type: &str,
        config: Value,
    ) -> RuntimeConfigInterface {
        RuntimeConfigInterface {
            priority: InterfacePriority::default(),
            interface_type: interface_type.to_string(),
            config,
        }
    }
}

#[derive(Datex, Debug, Default, Clone)]
pub struct RuntimeConfig {
    pub endpoint: Endpoint,
    pub interfaces: Option<Vec<RuntimeConfigInterface>>,
    pub env: Option<HashMap<String, String>>,
}

impl RuntimeConfig {
    pub fn new_with_endpoint(endpoint: Endpoint) -> Self {
        RuntimeConfig {
            endpoint,
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
            config,
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
        datex_proxy::{
            DatexValueContainerProxyDeserialize,
            DatexValueContainerProxyInfallibleSerialize,
        },
        prelude::*,
        runtime::{RuntimeConfig, RuntimeConfigInterface},
        values::core_values::{endpoint::Endpoint, map::Map},
    };

    #[derive(Datex)]
    struct MySetupData {
        field1: String,
        field2: i32,
    }

    #[test]
    fn add_env_var() {
        let mut config = RuntimeConfig::default();
        config.add_env_var("KEY1".to_string(), "VALUE1".to_string());
        let env_vars = config.env.unwrap();
        assert_eq!(env_vars.get("KEY1"), Some(&"VALUE1".to_string()));
    }

    #[test]
    fn datex_proxy_runtime_config_interface() {
        let config_interface = RuntimeConfigInterface::new(
            "test",
            MySetupData {
                field1: "value".to_string(),
                field2: 42,
            },
        )
        .unwrap();
        assert_eq!(config_interface.interface_type, "test");
        let setup_data = config_interface.config.clone();
        let map: Map = setup_data.try_into_value().unwrap();
        assert_eq!(
            map.get("field1")
                .unwrap()
                .clone()
                .try_into_value::<String>()
                .unwrap(),
            "value".to_string()
        );
        assert_eq!(
            map.get("field2")
                .unwrap()
                .clone()
                .try_into_value::<i32>()
                .unwrap(),
            42
        );

        let value_container = config_interface.to_value_container();
        let parsed_config_interface: RuntimeConfigInterface =
            value_container.try_into().unwrap();
        assert_eq!(parsed_config_interface.interface_type, "test");
    }

    #[test]
    fn datex_proxy_runtime_config() {
        let config = RuntimeConfig::new_with_endpoint(Endpoint::new("@test"));
        let value_container = config.to_value_container();
        let parsed_config: RuntimeConfig =
            RuntimeConfig::try_from_value_container(value_container).unwrap();
        assert_eq!(parsed_config.endpoint, Endpoint::new("@test"));
    }
}
