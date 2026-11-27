use crate::{
    network::com_hub::InterfacePriority,
    prelude::*,
    random::RandomState,
    traits::datex_native_structural::DatexNativeStructural,
    values::{core_values::endpoint::Endpoint, value::Value},
};
use datex_macros_internal::Datex;
use indexmap::IndexMap;
use datex_facades::vault::Vault;
use crate::network::com_hub::key_manager::own_endpoint_keys::OwnEndpointKeys;
use crate::vault::VaultImpl;

pub fn is_priority_none(v: &InterfacePriority) -> bool {
    matches!(v, InterfacePriority::None)
}

#[derive(Datex, Debug, Clone, PartialEq, Eq)]
#[datex(structural)]
/// A generic interface configuration to setup a runtime interface.
pub struct RuntimeConfigInterface {
    #[datex(rename = "type")]
    pub interface_type: String,
    pub config: Value,
    pub priority: InterfacePriority,
}

impl RuntimeConfigInterface {
    pub fn new<T: DatexNativeStructural>(
        interface_type: &str,
        setup_data: T,
    ) -> Result<RuntimeConfigInterface, String> {
        Ok(RuntimeConfigInterface {
            interface_type: interface_type.to_string(),
            priority: InterfacePriority::default(),
            config: Value::native(setup_data),
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

#[derive(Datex, Debug, Default, Clone, PartialEq, Eq)]
#[datex(structural)]
pub struct RuntimeConfig {
    /// An optional endpoint identifier for the runtime.
    /// When none is set, the endpoint is loaded from the vault if available, or a new anonymous endpoint identifier is generated.
    pub endpoint: Option<Endpoint>,
    /// An optional human-readable name for the endpoint which is publicly visible to other endpoints.
    pub name: Option<String>,
    /// Optional public+private keys for the endpoint. If not provided, a new key pair will be generated or retrieved from the vault if available.
    /// When these keys are set, they will always be used, even if a vault already contains keys for the endpoint. This allows for overriding the vault's keys.
    #[datex(serde)]
    pub keys: Option<OwnEndpointKeys>,
    /// Optional list of interfaces to be created for the runtime.
    pub interfaces: Option<Vec<RuntimeConfigInterface>>,
    /// Optional environment variables to be set for the runtime.
    pub env: Option<IndexMap<String, String, RandomState>>,
    /// The location of the vault for storing keys and other sensitive data.
    /// If not provided, the vault will not be used and keys will not be persisted.
    pub vault_location: Option<String>,
}

impl RuntimeConfig {
    pub fn new_with_endpoint(endpoint: Endpoint) -> Self {
        RuntimeConfig {
            endpoint: Some(endpoint),
            name: None,
            keys: None,
            interfaces: None,
            env: None,
            vault_location: None,
        }
    }

    pub fn add_interface<T: DatexNativeStructural>(
        &mut self,
        interface_type: String,
        config: T,
        priority: InterfacePriority,
    ) {
        let config = Value::native(config);
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
        self.env
            .get_or_insert_with(IndexMap::default)
            .insert(key, value);
    }

    /// Adds multiple environment variables to the runtime's custom environment variables.
    pub fn add_env_vars(
        &mut self,
        vars: IndexMap<String, String, RandomState>,
    ) {
        self.env.get_or_insert_with(IndexMap::default).extend(vars);
    }

    #[cfg(feature = "target_native")]
    /// Adds all host environment variables to the runtime's custom environment variables.
    pub fn load_host_env_vars(&mut self) {
        // add all host environment variables to the runtime's custom environment variables
        for (key, value) in std::env::vars() {
            self.env
                .get_or_insert_with(IndexMap::new)
                .insert(key, value);
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
            self.env.get_or_insert_with(IndexMap::new).insert(key, val);
        }
        Ok(())
    }

    /// Provides or creates the endpoint and own keys.
    /// First checks if the endpoint and keys are provided in the config.
    /// If not, it checks if the vault contains the endpoint and keys.
    /// If not, it generates a new anonymous endpoint and keys, and stores them in the vault if available.
    pub async fn get_or_create_endpoint_and_keys(
        &mut self,
    ) -> (Endpoint, OwnEndpointKeys) {
        let vault = self.vault_location.as_ref().map(|vault_location| VaultImpl::new(vault_location));
        
        let endpoint = match &self.endpoint {
            Some(endpoint) => endpoint.clone(),
            None => {
                // check if the vault contains the endpoint
                let stored_endpoint = match &vault {
                    Some(vault) => vault.get(&Endpoint::ANY.to_string(), "active_endpoint"),
                    None => None,
                };

                match stored_endpoint {
                    Some(stored_endpoint) => Endpoint::new(&stored_endpoint),
                    None => {
                        // generate a new anonymous endpoint
                        let new_endpoint = Endpoint::random();
                        // store the new endpoint in the vault if available
                        if let Some(vault) = &vault {
                            vault.store(&Endpoint::ANY.to_string(), "active_endpoint", &new_endpoint.to_string());
                        }
                        new_endpoint
                    }
                }
            }
        };

        let keys = match &self.keys {
            Some(keys) => {
                // update the vault with the provided keys
                if let Some(vault) = &vault {
                    let serialized_keys = keys.serialize_to_base58().expect("Failed to serialize own keys");
                    vault.store( & endpoint.to_string(), "own_keys", & serialized_keys);
                }
                keys.clone()
            },
            _ => {
                let stored_keys = match &vault {
                    Some(vault) => vault.get(&endpoint.to_string(), "own_keys"),
                    None => None,
                };

                // If keys are stored in the vault, use them. Otherwise, generate new keys and store them in the vault.
                match stored_keys {
                    Some(stored_keys) => {
                        let keys: OwnEndpointKeys = OwnEndpointKeys::deserialize_from_base58(&stored_keys).expect("Failed to deserialize own keys");
                        keys
                    }
                    None => {
                        let keys = OwnEndpointKeys::new().await;
                        let serialized_keys = keys
                            .serialize_to_base58()
                            .expect("Failed to serialize own keys");
                        if let Some(vault) = &vault {
                            vault.store(&endpoint.to_string(), "own_keys", &serialized_keys);
                        }
                        keys
                    }
                }
            }
        };
        
        // also store the endpoint and keys in the config for future use
        self.endpoint = Some(endpoint.clone());
        self.keys = Some(keys.clone());

        (endpoint, keys)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        network::com_hub::InterfacePriority,
        prelude::*,
        runtime::{Runtime, RuntimeConfig, RuntimeConfigInterface},
        traits::{
            convert_value_container::ConvertValueContainer, try_clone::TryClone,
        },
        values::{
            core_values::{endpoint::Endpoint, map::Map},
            value::Value,
            value_container::ValueContainer,
        },
    };
    use datex_macros_internal::Datex;
    use crate::network::com_hub::key_manager::own_endpoint_keys::OwnEndpointKeys;

    #[derive(Datex, Clone)]
    #[datex(structural)]
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
        let setup_data = config_interface.config.try_clone().unwrap();
        let map: Map = setup_data.try_convert_into_value().unwrap();
        assert_eq!(
            map.try_get(&ValueContainer::from("field1"))
                .unwrap()
                .clone()
                .try_into_value::<String>()
                .unwrap(),
            "value".to_string()
        );
        assert_eq!(
            map.try_get(&ValueContainer::from("field2"))
                .unwrap()
                .clone()
                .try_into_value::<i32>()
                .unwrap(),
            42
        );

        let value_container = Value::native(config_interface);
        let parsed_config_interface: RuntimeConfigInterface =
            value_container.try_into_value().unwrap();
        assert_eq!(parsed_config_interface.interface_type, "test");
    }

    #[test]
    fn datex_proxy_runtime_config() {
        let config = RuntimeConfig::new_with_endpoint(Endpoint::new("@test"));
        let value_container = Value::native(config);
        let parsed_config: RuntimeConfig =
            value_container.try_into_value().unwrap();
        assert_eq!(parsed_config.endpoint, Some(Endpoint::new("@test")));
    }

    #[test]
    #[cfg(feature = "compiler")]
    fn read_interface_priority_from_datex_source() {
        let priority_source = "#Priority(5u16)";

        let priority = InterfacePriority::try_from_script(
            priority_source,
            &Runtime::stub(),
        );
        assert_eq!(priority, Ok(InterfacePriority::Priority(5)));
    }

    #[test]
    #[cfg(feature = "compiler")]
    fn read_interface_configs_from_datex_source() {
        let interface_config_source = r#"[
            {
                type: "websocket-client",
                config: {url:"wss://example.unyt.land"},
                priority: #Priority(0u16)
            }
        ]"#;

        let config = <Vec<RuntimeConfigInterface>>::try_from_script(
            interface_config_source,
            &Runtime::stub(),
        );

        assert_eq!(
            config,
            Ok(vec![RuntimeConfigInterface {
                interface_type: "websocket-client".to_string(),
                config: Map::from(vec![(
                    "url",
                    "wss://example.unyt.land".into()
                )])
                .into(),
                priority: InterfacePriority::Priority(0),
            }])
        );
    }

    #[test]
    #[cfg(feature = "compiler")]
    fn read_runtime_config_from_datex_source() {
        let keys = OwnEndpointKeys {
            pub_key: [1u8; 32],
            pri_key: [2u8; 32],
        };

        let config_source = format!(r#"{{
            endpoint: @test,
            name: "Example",
            keys: {},
            interfaces: [
                {{
                    type: "websocket-client",
                    config: {{url:"wss://example.unyt.land"}},
                    priority: #Priority(0u16)
                }}
            ],
            env: null,
            vault_location: "./vault"
        }}"#, serde_json::to_string(&keys).unwrap());

        let config =
            RuntimeConfig::try_from_script(&config_source, &Runtime::stub());

        assert_eq!(
            config,
            Ok(RuntimeConfig {
                endpoint: Some(Endpoint::new("@test")),
                name: Some("Example".to_string()),
                keys: Some(keys),
                interfaces: Some(vec![RuntimeConfigInterface {
                    interface_type: "websocket-client".to_string(),
                    config: Map::from(vec![(
                        "url",
                        "wss://example.unyt.land".into()
                    )])
                    .into(),
                    priority: InterfacePriority::Priority(0),
                }]),
                env: None,
                vault_location: Some("./vault".to_string()),
            })
        );
    }
}
