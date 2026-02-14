use crate::prelude::*;
use core::cell::RefCell;
use crate::random::RandomState;
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct RuntimeEnv {
    pub(crate) env_vars: RefCell<IndexMap<String, String, RandomState>>,
}

impl RuntimeEnv {
    /// Adds a single environment variable to the runtime's custom environment variables.
    pub fn add_env_var(
        &self,
        key: String,
        value: String,
    ) {
        self.env_vars.borrow_mut().insert(key, value);
    }

    /// Adds multiple environment variables to the runtime's custom environment variables.
    pub fn add_env_vars(
        &self,
        vars: IndexMap<String, String, RandomState>,
    ) {
        self.env_vars.borrow_mut().extend(vars);
    }

    #[cfg(feature = "std")]
    /// Adds all host environment variables to the runtime's custom environment variables.
    pub fn add_host_env_vars(&self) {
        // add all host environment variables to the runtime's custom environment variables
        for (key, value) in std::env::vars() {
            self.env_vars.borrow_mut().insert(key, value);
        }
    }

    #[cfg(feature = "std")]
    /// Adds all environment variables from a .env file to the runtime's custom environment variables.
    pub fn add_env_vars_from_file(&self, path: &std::path::PathBuf) -> Result<(), dotenvy::Error> {
        let loader1 = dotenvy::from_path_iter(path)?;
        for item in loader1 {
            let (key, val) = item?;
            self.env_vars.borrow_mut().insert(key, val);
        }
        Ok(())
    }

    /// Returns the current environment variables as a HashMap
    pub fn get_env(&self) -> IndexMap<String, String, RandomState> {
        self.env_vars.borrow().clone()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;
    use super::RuntimeEnv;

    #[test]
    fn test_add_env_var() {
        let env = RuntimeEnv::default();
        env.add_env_var("KEY1".to_string(), "VALUE1".to_string());
        let env_vars = env.get_env();
        assert_eq!(env_vars.get("KEY1"), Some(&"VALUE1".to_string()));
    }
}