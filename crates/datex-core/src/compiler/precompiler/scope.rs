use crate::collections::HashMap;

use crate::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct PrecompilerScope {
    pub realm_index: usize,
    pub variable_ids_by_name: HashMap<String, usize>,
    pub external_variables: HashSet<usize>,
}

impl PrecompilerScope {
    pub fn new_with_realm_index(realm_index: usize) -> Self {
        PrecompilerScope {
            realm_index,
            variable_ids_by_name: HashMap::new(),
            external_variables: HashSet::new(),
        }
    }

    /// Registers the use of an external variable in the current scope
    pub fn register_external_variable(&mut self, variable_id: usize) {
        self.external_variables.insert(variable_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewScopeType {
    // no new scope, just continue in the current scope
    None,
    // create a new scope, but do not increment the realm index
    NewScope,
    // create a new scope and increment the realm index (e.g. for remote execution calls)
    NewScopeWithNewRealm,
}
