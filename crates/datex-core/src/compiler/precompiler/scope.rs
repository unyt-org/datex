use crate::{ast::expressions::ValueAccessType, collections::HashMap};

use crate::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ExternalVariable {
    Registered(usize),
    UnresolvedPlaceholder(usize, ValueAccessType),
}

#[derive(Default, Debug, Clone)]
pub struct PrecompilerScope {
    pub realm_index: usize,
    pub variable_ids_by_name: HashMap<String, usize>,
    pub external_variables: HashSet<ExternalVariable>,
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
    pub fn register_external_variable(
        &mut self,
        external_variable: ExternalVariable,
    ) {
        self.external_variables.insert(external_variable);
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
