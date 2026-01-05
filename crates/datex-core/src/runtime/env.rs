use crate::prelude::*;
use core::cell::RefCell;
use crate::random::RandomState;
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct RuntimeEnv {
    pub(crate) env_vars: RefCell<IndexMap<String, String, RandomState>>,
}

impl RuntimeEnv {
   
}

impl From<HashMap<String, String>> for RuntimeEnv {
    fn from(env_vars: HashMap<String, String>) -> Self {
        RuntimeEnv {
            env_vars: RefCell::new(IndexMap::from_iter(env_vars.into_iter())),
        }
    }
}

