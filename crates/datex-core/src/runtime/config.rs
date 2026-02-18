use crate::prelude::*;
use crate::runtime::is_none_variant;
use crate::collections::HashMap;
use serde::Serialize;
use crate::network::com_hub::InterfacePriority;
use crate::serde::Deserialize;
use crate::serde::error::SerializationError;
use crate::serde::serializer::to_value_container;
use crate::values::core_values::endpoint::Endpoint;
use crate::values::value_container::ValueContainer;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct RuntimeConfigInterface {
    #[serde(rename = "type")]
    pub interface_type: String,
    #[serde(rename = "config")]
    #[cfg_attr(feature = "wasm_runtime", tsify(type = "unknown"))]
    pub setup_data: ValueContainer,

    #[serde(default, skip_serializing_if = "is_none_variant")]
    pub priority: InterfacePriority,
}

impl RuntimeConfigInterface {
    pub fn new<T: Serialize>(
        interface_type: &str,
        setup_data: T,
    ) -> Result<RuntimeConfigInterface, SerializationError> {
        Ok(RuntimeConfigInterface {
            interface_type: interface_type.to_string(),
            priority: InterfacePriority::default(),
            setup_data: to_value_container(&setup_data)?,
        })
    }

    pub fn new_from_value_container(
        interface_type: &str,
        config: ValueContainer,
    ) -> RuntimeConfigInterface {
        RuntimeConfigInterface {
            priority: InterfacePriority::default(),
            interface_type: interface_type.to_string(),
            setup_data: config,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
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

    pub fn add_interface<T: Serialize>(
        &mut self,
        interface_type: String,
        config: T,
        priority: InterfacePriority,
    ) -> Result<(), SerializationError> {
        let config = to_value_container(&config)?;
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

        Ok(())
    }

    /// Adds a single environment variable to the runtime's custom environment variables.
    pub fn add_env_var(
        &mut self,
        key: String,
        value: String,
    ) {
        self.env.get_or_insert_with(HashMap::new).insert(key, value);
    }

    /// Adds multiple environment variables to the runtime's custom environment variables.
    pub fn add_env_vars(
        &mut self,
        vars: HashMap<String, String>,
    ) {
        self.env.get_or_insert_with(HashMap::new).extend(vars);
    }

    #[cfg(feature = "native")]
    /// Adds all host environment variables to the runtime's custom environment variables.
    pub fn load_host_env_vars(&mut self) {
        // add all host environment variables to the runtime's custom environment variables
        for (key, value) in std::env::vars() {
            self.env.get_or_insert_with(HashMap::new).insert(key, value);
        }
    }

    #[cfg(feature = "native")]
    /// Adds all environment variables from a .env file to the runtime's custom environment variables.
    pub fn add_env_vars_from_file(&mut self, path: &std::path::PathBuf) -> Result<(), dotenvy::Error> {
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
    use crate::prelude::*;
    use crate::runtime::RuntimeConfig;

    #[test]
    fn test_add_env_var() {
        let mut config = RuntimeConfig::default();
        config.add_env_var("KEY1".to_string(), "VALUE1".to_string());
        let env_vars = config.env.unwrap();
        assert_eq!(env_vars.get("KEY1"), Some(&"VALUE1".to_string()));
    }
}