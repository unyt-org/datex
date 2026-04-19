#[cfg(feature = "compiler")]
use crate::ast::expressions::DatexExpressionData;
use crate::{
    prelude::*,
    traits::structural_eq::StructuralEq,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        shared_container_containing_type::SharedContainerContainingType,
        type_definition::TypeDefinition,
        type_definition_with_metadata::TypeDefinitionWithMetadata, type_match::TypeMatch,
    },
};
use core::{
    fmt::{Display, write},
    hash::Hash,
    ops::Deref,
};
use serde::{Deserialize, Serialize};
use crate::runtime::memory::Memory;
use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
use crate::values::core_value::CoreValue;
use crate::values::value_container::ValueContainer;

// {x: &integer}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    Alias(TypeDefinitionWithMetadata),
    Nominal(SharedContainerContainingNominalType),
}

impl Type {

    pub fn nominal(
        definition: NominalTypeDefinition,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        memory: &Memory
    ) -> Type {
        Type::Nominal(SharedContainerContainingNominalType::new_from_definition(definition, address_provider, memory))
    }

    /// Collapses nominal type definitions to their underlying type definitions with metadata
    pub fn with_collapsed_definition_with_metadata<R>(
        &self,
        f: impl FnOnce(&TypeDefinitionWithMetadata) -> R,
    ) -> R {
        match self {
            Type::Alias(type_def) => f(type_def),
            Type::Nominal(nominal_def) => {
                nominal_def.with_collapsed_definition(|def| f(def.definition()))
            }
        }
    }

    /// Collapses nominal type definitions to their underlying structural type definitions
    pub fn with_collapsed_type_definition<R>(
        &self,
        f: impl FnOnce(&TypeDefinition) -> R,
    ) -> R {
        self.with_collapsed_definition_with_metadata(|def| f(&def.definition))
    }

    pub fn base_core_lib_type(
        &self,
        memory: &Memory
    ) -> SharedContainerContainingNominalType {
        match self {
            Type::Alias(type_def) => type_def.definition.base_core_lib_type(memory),
            Type::Nominal(nominal_def) => {
                todo!()
            }
        }
    }
}

impl<T: Into<TypeDefinitionWithMetadata>> From<T> for Type {
    fn from(definition: T) -> Self {
        Type::Alias(definition.into())
    }
}

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
                    Type::Alias(self_definition) => false,
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
        todo!()
    }
}

impl Type {
    // / 1 matches 1 -> true
    // / 1 matches 2 -> false
    // / 1 matches 1 | 2 -> true
    // / 1 matches "x" | 2 -> false
    // / integer matches 1 | 2 -> false
    // pub fn value_matches(&self, value: &ValueContainer) -> bool {
    //     Type::value_matches_type(value, self)
    // }

    // / Checks if an atomic type matches another type
    // / An atomic type can be any type variant besides union or intersection
    // pub fn atomic_matches_type(atomic_type: &Type, other: &Type) -> bool {
    //     // FIXME #768: match rules for prefixes are more nuanced than just equality, e.g. &mut T should match &T, ...
    //     if atomic_type.metadata != other.metadata {
    //         return false;
    //     }

    //     match &other.type_definition {
    //         TypeDefinition::Shared(reference) => {
    //             // compare base type of atomic_type with the referenced type
    //             if let Some(atomic_base_type_reference) =
    //                 atomic_type.base_type_reference()
    //             {
    //                 *atomic_base_type_reference.borrow() == *reference.borrow()
    //             } else {
    //                 false
    //             }
    //         }
    //         TypeDefinition::Union(members) => {
    //             // atomic type must match at least one member of the union
    //             for member in members {
    //                 if Type::atomic_matches_type(atomic_type, member) {
    //                     return true;
    //                 }
    //             }
    //             false
    //         }
    //         TypeDefinition::Intersection(members) => {
    //             // atomic type must match all members of the intersection
    //             for member in members {
    //                 if !Type::atomic_matches_type(atomic_type, member) {
    //                     return false;
    //                 }
    //             }
    //             true
    //         }
    //         _ => {
    //             // compare type definitions directly
    //             atomic_type.type_definition == other.type_definition
    //         }
    //     }
    // }

    // /// Matches a value against a type
    // pub fn value_matches_type(
    //     value: &ValueContainer,
    //     match_type: &Type,
    // ) -> bool {
    //     // if match_type == &value.actual_type().as_type() {
    //     //     return true;
    //     // }

    //     match &match_type.type_definition {
    //         // e.g. 1 matches 1 | 2
    //         TypeDefinition::Union(types) => {
    //             // value must match at least one of the union types
    //             types.iter().any(|t| Type::value_matches_type(value, t))
    //         }
    //         TypeDefinition::Intersection(types) => {
    //             // value must match all of the intersection types
    //             types.iter().all(|t| Type::value_matches_type(value, t))
    //         }
    //         TypeDefinition::Literal(structural_type) => {
    //             structural_type.value_matches(value)
    //         }
    //         TypeDefinition::Shared(_reference) => {
    //             core::todo!("#327 handle reference type matching");
    //             //reference.value_matches(value)
    //         }
    //         TypeDefinition::Type(inner_type) => {
    //             // TODO #464: also check mutability of current type?
    //             inner_type.value_matches(value)
    //         }
    //         TypeDefinition::Callable(_signature) => {
    //             core::todo!("#328 handle function type matching");
    //         }
    //         TypeDefinition::Collection(_collection_type) => {
    //             core::todo!("#329 handle collection type matching");
    //         }
    //         TypeDefinition::Unit => false, // unit type does not match any value
    //         TypeDefinition::Never => false,
    //         TypeDefinition::Unknown => false,
    //         TypeDefinition::ImplType(ty, _) => {
    //             Type::value_matches_type(value, ty)
    //         }
    //     }
    // }
}

impl StructuralEq for Type {
    // FIXME is this what we want?
    fn structural_eq(&self, other: &Self) -> bool {
        self.with_collapsed_definition_with_metadata(|own| {
            other.with_collapsed_definition_with_metadata(|other| {
                own.definition.structural_eq(&other.definition)
            })
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Type::Alias(def) => write!(f, "{}", def),
            Type::Nominal(nom) => write!(f, "{}", nom.deref()),
        }
    }
}

// impl From<&CoreValue> for Type {
//     fn from(value: &CoreValue) -> Self {
//         match value {
//             CoreValue::Null => Type::structural(
//                 LiteralTypeDefinition::Null,
//                 TypeMetadata::default(),
//             ),
//             CoreValue::Boolean(b) => Type::structural(
//                 LiteralTypeDefinition::Boolean(b.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::Text(s) => {
//                 Type::structural(s.clone(), TypeMetadata::default())
//             }
//             CoreValue::Decimal(d) => Type::structural(
//                 LiteralTypeDefinition::Decimal(d.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::TypedDecimal(td) => Type::structural(
//                 LiteralTypeDefinition::TypedDecimal(td.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::Integer(i) => Type::structural(
//                 LiteralTypeDefinition::Integer(i.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::TypedInteger(ti) => Type::structural(
//                 LiteralTypeDefinition::TypedInteger(ti.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::Endpoint(e) => Type::structural(
//                 LiteralTypeDefinition::Endpoint(e.clone()),
//                 TypeMetadata::default(),
//             ),
//             CoreValue::List(list) => {
//                 let types = list
//                     .iter()
//                     .map(|v| {
//                         Type::from(v.to_cloned_value().borrow().inner.clone())
//                     })
//                     .collect::<Vec<_>>();
//                 Type::structural(
//                     LiteralTypeDefinition::List(types),
//                     TypeMetadata::default(),
//                 )
//             }
//             CoreValue::Map(map) => {
//                 let struct_types = map
//                     .iter()
//                     .map(|(key, value)| {
//                         (
//                             Type::from(
//                                 ValueContainer::from(key)
//                                     .to_cloned_value()
//                                     .borrow()
//                                     .inner
//                                     .clone(),
//                             ),
//                             Type::from(
//                                 value.to_cloned_value().borrow().inner.clone(),
//                             ),
//                         )
//                     })
//                     .collect::<Vec<_>>();
//                 Type::structural(
//                     LiteralTypeDefinition::Map(struct_types),
//                     TypeMetadata::default(),
//                 )
//             }
//             e => unimplemented!("Type conversion not implemented for {}", e),
//         }
//     }
// }
// impl From<CoreValue> for Type {
//     fn from(value: CoreValue) -> Self {
//         Type::from(&value)
//     }
// }

#[cfg(feature = "compiler")]
impl TryFrom<&DatexExpressionData> for LiteralTypeDefinition {
    type Error = ();

    fn try_from(expr: &DatexExpressionData) -> Result<Self, Self::Error> {
        Ok(match expr {
            DatexExpressionData::Null => LiteralTypeDefinition::Null,
            DatexExpressionData::Boolean(b) => {
                LiteralTypeDefinition::Boolean(*b)
            }
            DatexExpressionData::Text(s) => {
                LiteralTypeDefinition::Text(s.clone())
            }
            DatexExpressionData::Decimal(d) => {
                LiteralTypeDefinition::Decimal(d.clone())
            }
            DatexExpressionData::Integer(i) => {
                LiteralTypeDefinition::Integer(i.clone())
            }
            DatexExpressionData::Endpoint(e) => {
                LiteralTypeDefinition::Endpoint(e.clone())
            }
            _ => return Err(()),
        })
    }
}

#[cfg(feature = "compiler")]
impl TryFrom<&DatexExpressionData> for Type {
    type Error = ();

    fn try_from(expr: &DatexExpressionData) -> Result<Self, Self::Error> {
        Ok(Type::from(
            LiteralTypeDefinition::try_from(expr)?,
        ))
    }
}


impl TryFrom<ValueContainer> for Type {
    type Error = ();

    fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
        match value {
            ValueContainer::Shared(shared) => {
                SharedContainerContainingNominalType::try_from(shared)
                    .map(Type::Nominal)
            }
            ValueContainer::Local(value) => {
                match value.inner {
                    CoreValue::Type(ty) => Ok(ty),
                    _ => Err(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use crate::{
        types::r#type::Type,
        values::{
            core_values::{
                integer::{Integer, typed_integer::TypedInteger},
                text::Text,
            },
            value_container::ValueContainer,
        },
    };
    use crate::libs::core::type_id::CoreLibBaseTypeId;
    use crate::runtime::memory::Memory;
    use crate::types::literal_type_definition::LiteralTypeDefinition;
    use crate::types::type_definition::TypeDefinition;
    use crate::types::type_match::TypeMatch;

    #[test]
    fn test_match_equal_values() {
        // 1u8 matches 1u8
        assert!(
            Type::from(LiteralTypeDefinition::TypedInteger(1u8.into()))
                .matched_by_value(&TypedInteger::from(1u8).into())
        );

        // 1u16 matches 1u16
        assert!(
            Type::from(LiteralTypeDefinition::TypedInteger(1u16.into()))
                .matched_by_value(&TypedInteger::from(1u16).into())
        );

        // 1 matches 1
        assert!(
            Type::from(LiteralTypeDefinition::Integer(1.into()))
                .matched_by_value(&Integer::from(1).into())
        );

        // "test" matches "test"
        assert!(
            Type::from(LiteralTypeDefinition::Text("test".into()))
                .matched_by_value(&Text::from("test").into())
        );
    }

    #[test]
    fn test_match_union() {
        // 1 matches (1 | 2 | 3)
        assert!(
            Type::from(TypeDefinition::Union(
                vec![
                    LiteralTypeDefinition::Integer(Integer::from(1)).into(),
                    LiteralTypeDefinition::Integer(Integer::from(2)).into(),
                    LiteralTypeDefinition::Integer(Integer::from(3)).into()
                ]
            )).matched_by_value(&Integer::from(1).into())
        );
    }

    #[test]
    fn type_matches_union_type() {
        let memory = &Memory::new();

        // 1 matches (1 | 2 | 3)
        assert!(
            Type::from(LiteralTypeDefinition::Integer(Integer::from(1)))
                .matches(&Type::from(TypeDefinition::Union(
                    vec![
                        LiteralTypeDefinition::Integer(Integer::from(1)).into(),
                        LiteralTypeDefinition::Integer(Integer::from(2)).into(),
                        LiteralTypeDefinition::Integer(Integer::from(3)).into()
                    ]
                )))
        );

        // 1 matches integer | text
        assert!(
            Type::from(LiteralTypeDefinition::Integer(Integer::from(1)))
                .matches(&Type::from(TypeDefinition::Union(
                    vec![
                        memory.get_core_type(CoreLibBaseTypeId::Integer),
                        memory.get_core_type(CoreLibBaseTypeId::Text),
                    ]
                )))
        );
    }

    // TODO #330
    // #[test]
    // fn test_match_combined_type() {
    //     // [1, 1] matches List<1>
    //     assert!(Type::value_matches_type(
    //         &ValueContainer::from(List::from(vec![1, 1])),
    //         &Type::list(Type::structural(1))
    //     ));
    //
    //     // [1, 2] matches List<(1 | 2)>
    //     assert!(Type::value_matches_type(
    //         &ValueContainer::from(List::from(vec![1, 2])),
    //         &Type::list(Type::union(vec![
    //             Type::structural(1).as_type_container(),
    //             Type::structural(2).as_type_container(),
    //         ])),
    //     ));
    //
    //     // [1, 2] does not match List<1>
    //     assert!(!Type::value_matches_type(
    //         &ValueContainer::from(List::from(vec![1, 2])),
    //         &Type::list(Type::structural(1))
    //     ));
    //
    //     // ["test", "jonas"] matches List<("jonas" | "test" | 3)>
    //     assert!(Type::value_matches_type(
    //         &ValueContainer::from(List::from(vec!["test", "jonas"])),
    //         &Type::list(Type::union(vec![
    //             Type::structural("jonas"),
    //             Type::structural("test"),
    //             Type::structural(3),
    //         ])),
    //     ));
    // }
}
