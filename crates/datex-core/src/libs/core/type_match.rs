use crate::libs::core::type_id::CoreLibTypeId;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_match::{TypeSatisfiesValueContainer, TypeSuperset};
use crate::values::value_container::ValueContainer;

/// Type superset of core lib types, e.g. integer >= integer/u8
impl TypeSuperset<CoreLibTypeId> for CoreLibTypeId {
    fn is_superset_of(&self, other: &CoreLibTypeId) -> bool {
        // exact match
        if self == other {
            true
        }
        // other is subvariant of self
        else if let CoreLibTypeId::Variant(variant_id) = other
            && let CoreLibTypeId::Base(base_id) = self
            && variant_id.base_type_id() == *base_id
        {
            true
        }
        else {
            false
        }
    }
}

/// Type superset of core lib types for any TypeDefinition, e.g. integer >= 1
impl TypeSuperset<TypeDefinition> for CoreLibTypeId {
    fn is_superset_of(&self, other: &TypeDefinition) -> bool {
        if let Some(other_core_lib_type_id) = other.core_lib_type_id() {
            self.is_superset_of(&other_core_lib_type_id)
        }
        else {
            false
        }
    }
}


impl TypeSatisfiesValueContainer for CoreLibTypeId {
    fn satisfies_value_container(&self, value: &ValueContainer) -> bool {
        todo!()
    }
}
