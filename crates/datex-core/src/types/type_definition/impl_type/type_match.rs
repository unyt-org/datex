use crate::types::{
    traits::type_match::TypeSuperset,
    type_definition::impl_type::ImplTypeDefinition,
};

impl TypeSuperset<ImplTypeDefinition> for ImplTypeDefinition {
    fn is_superset_of(&self, other: &ImplTypeDefinition) -> bool {
        // other must include all impls that self includes
        let all_impls_in_self_are_in_other = self
            .impl_markers
            .iter()
            .all(|self_impl| other.impl_markers.contains(self_impl));

        if !all_impls_in_self_are_in_other {
            return false;
        }

        // type self must be superset of type in other
        self.inner_type.is_superset_of(other.inner_type.as_ref())
    }
}
