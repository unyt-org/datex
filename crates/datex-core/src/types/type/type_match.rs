use crate::{
    types::{r#type::Type, type_match::TypeMatch},
    values::value_container::ValueContainer,
};

impl TypeMatch for Type {
    /// 1 matches integer -> true
    /// integer matches 1 -> false
    /// integer matches integer -> true
    /// 1 matches integer | text -> true
    fn matches(&self, other_definition: &Type) -> bool {
        match &other_definition {
            Type::Alias(inner_other_definition) => self
                .with_collapsed_definition_with_metadata(|self_definition| {
                    self_definition.matches(inner_other_definition)
                }),
            Type::Nominal(other_nominal_definition) => {
                match self {
                    // FIXME is this type alias here allowed?
                    Type::Alias(_self_definition) => false,
                    Type::Nominal(self_nominal_definition) => {
                        // compare collapsed definitions of both nominal types
                        self_nominal_definition
                            .matches(other_nominal_definition)
                    }
                }
            }
        }
    }

    fn matched_by_value(&self, value: &ValueContainer) -> bool {
        match self {
            Type::Alias(definition) => definition.matched_by_value(value),
            Type::Nominal(definition) => definition.matched_by_value(value),
        }
    }
}
