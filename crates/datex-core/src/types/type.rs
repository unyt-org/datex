#[cfg(feature = "compiler")]
use crate::ast::expressions::DatexExpressionData;
use crate::{
    libs::core::{CoreLibTypeId, core_lib_type, get_core_lib_type_reference},
    prelude::*,
    shared_values::{
        pointer_address::PointerAddress,
        shared_containers::{
            ReferenceMutability, SharedContainerMutability,
            SharedContainerOwnership,
        },
    },
    traits::structural_eq::StructuralEq,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        structural_type_definition::TypeDefinition,
        type_definition::TypeDefinitionWithMetadata,
    },
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean, callable::CallableSignature,
            decimal::typed_decimal::DecimalTypeVariant,
            integer::typed_integer::IntegerTypeVariant, text::Text,
        },
        value_container::ValueContainer,
    },
};
use core::{fmt::Display, hash::Hash, unimplemented};

// {x: &integer}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    Alias(TypeDefinitionWithMetadata),
    Nominal(NominalTypeDefinition),
}

impl Type {
    /// Get the inner [TypeDefinition]
    pub fn definition(&self) -> &TypeDefinitionWithMetadata {
        match self {
            Type::Alias(type_def) => type_def,
            Type::Nominal(nominal_def) => nominal_def.definition(),
        }
    }

    /// Convert to the inner [TypeDefinition]
    pub fn into_definition(self) -> TypeDefinitionWithMetadata {
        match self {
            Type::Alias(type_def) => type_def,
            Type::Nominal(nominal_def) => nominal_def.into_definition(),
        }
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner [StructuralTypeDefinition] value
    pub fn with_collapsed_structural_type_definition<R>(
        &self,
        f: impl FnOnce(&TypeDefinition) -> R,
    ) -> R {
        self.definition()
            .structural_definition
            .with_collapsed_structural_type_definition(f)
    }
}

impl Type {
    /// 1 matches 1 -> true
    /// 1 matches 2 -> false
    /// 1 matches 1 | 2 -> true
    /// 1 matches "x" | 2 -> false
    /// integer matches 1 | 2 -> false
    pub fn value_matches(&self, value: &ValueContainer) -> bool {
        Type::value_matches_type(value, self)
    }

    /// 1 matches integer -> true
    /// integer matches 1 -> false
    /// integer matches integer -> true
    /// 1 matches integer | text -> true
    pub fn matches_type(&self, other: &Type) -> bool {
        match &self.type_definition {
            TypeDefinition::Union(members) => {
                // If self is a union, check if any member matches the other type
                for member in members {
                    if member.matches_type(other) {
                        return true;
                    }
                }
                false
            }
            TypeDefinition::Intersection(members) => {
                // If self is an intersection, all members must match the other type
                for member in members {
                    if !member.matches_type(other) {
                        return false;
                    }
                }
                true
            }
            _ => {
                // atomic type match
                Type::atomic_matches_type(self, other)
            }
        }
    }

    /// Checks if an atomic type matches another type
    /// An atomic type can be any type variant besides union or intersection
    pub fn atomic_matches_type(atomic_type: &Type, other: &Type) -> bool {
        // FIXME #768: match rules for prefixes are more nuanced than just equality, e.g. &mut T should match &T, ...
        if atomic_type.metadata != other.metadata {
            return false;
        }

        match &other.type_definition {
            TypeDefinition::Shared(reference) => {
                // compare base type of atomic_type with the referenced type
                if let Some(atomic_base_type_reference) =
                    atomic_type.base_type_reference()
                {
                    *atomic_base_type_reference.borrow() == *reference.borrow()
                } else {
                    false
                }
            }
            TypeDefinition::Union(members) => {
                // atomic type must match at least one member of the union
                for member in members {
                    if Type::atomic_matches_type(atomic_type, member) {
                        return true;
                    }
                }
                false
            }
            TypeDefinition::Intersection(members) => {
                // atomic type must match all members of the intersection
                for member in members {
                    if !Type::atomic_matches_type(atomic_type, member) {
                        return false;
                    }
                }
                true
            }
            _ => {
                // compare type definitions directly
                atomic_type.type_definition == other.type_definition
            }
        }
    }

    /// Matches a value against a type
    pub fn value_matches_type(
        value: &ValueContainer,
        match_type: &Type,
    ) -> bool {
        // if match_type == &value.actual_type().as_type() {
        //     return true;
        // }

        match &match_type.type_definition {
            // e.g. 1 matches 1 | 2
            TypeDefinition::Union(types) => {
                // value must match at least one of the union types
                types.iter().any(|t| Type::value_matches_type(value, t))
            }
            TypeDefinition::Intersection(types) => {
                // value must match all of the intersection types
                types.iter().all(|t| Type::value_matches_type(value, t))
            }
            TypeDefinition::Literal(structural_type) => {
                structural_type.value_matches(value)
            }
            TypeDefinition::Shared(_reference) => {
                core::todo!("#327 handle reference type matching");
                //reference.value_matches(value)
            }
            TypeDefinition::Type(inner_type) => {
                // TODO #464: also check mutability of current type?
                inner_type.value_matches(value)
            }
            TypeDefinition::Callable(_signature) => {
                core::todo!("#328 handle function type matching");
            }
            TypeDefinition::Collection(_collection_type) => {
                core::todo!("#329 handle collection type matching");
            }
            TypeDefinition::Unit => false, // unit type does not match any value
            TypeDefinition::Never => false,
            TypeDefinition::Unknown => false,
            TypeDefinition::ImplType(ty, _) => {
                Type::value_matches_type(value, ty)
            }
        }
    }
}

impl StructuralEq for Type {
    fn structural_eq(&self, other: &Self) -> bool {
        self.type_definition.structural_eq(&other.type_definition)
            && self.metadata == other.metadata
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let prefix = match &self.metadata {
            TypeMetadata::Local {
                mutability,
                reference_mutability,
            } => {
                let ref_prefix = match reference_mutability {
                    Some(LocalReferenceMutability::Immutable) => "&",
                    Some(LocalReferenceMutability::Mutable) => "&mut ",
                    None => "",
                };
                let mut_prefix = match mutability {
                    LocalMutability::Immutable => "",
                    LocalMutability::Mutable => "mut ",
                };
                format!("{}{}", ref_prefix, mut_prefix)
            }
            TypeMetadata::Shared {
                mutability,
                reference_mutability,
            } => {
                let ref_prefix = match reference_mutability {
                    Some(ReferenceMutability::Immutable) => "'",
                    Some(ReferenceMutability::Mutable) => "'mut ",
                    None => "",
                };
                let shared_prefix = match mutability {
                    SharedContainerMutability::Immutable => "shared ",
                    SharedContainerMutability::Mutable => "shared mut ",
                };
                format!("{}{}", ref_prefix, shared_prefix)
            }
        };
        let base = self
            .base_type
            .as_ref()
            .map_or("".to_string(), |b| format!(": {}", b.borrow()));
        core::write!(f, "{}{}{}", prefix, self.type_definition, base)
    }
}

impl From<&CoreValue> for Type {
    fn from(value: &CoreValue) -> Self {
        match value {
            CoreValue::Null => Type::structural(
                LiteralTypeDefinition::Null,
                TypeMetadata::default(),
            ),
            CoreValue::Boolean(b) => Type::structural(
                LiteralTypeDefinition::Boolean(b.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::Text(s) => {
                Type::structural(s.clone(), TypeMetadata::default())
            }
            CoreValue::Decimal(d) => Type::structural(
                LiteralTypeDefinition::Decimal(d.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::TypedDecimal(td) => Type::structural(
                LiteralTypeDefinition::TypedDecimal(td.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::Integer(i) => Type::structural(
                LiteralTypeDefinition::Integer(i.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::TypedInteger(ti) => Type::structural(
                LiteralTypeDefinition::TypedInteger(ti.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::Endpoint(e) => Type::structural(
                LiteralTypeDefinition::Endpoint(e.clone()),
                TypeMetadata::default(),
            ),
            CoreValue::List(list) => {
                let types = list
                    .iter()
                    .map(|v| {
                        Type::from(v.to_cloned_value().borrow().inner.clone())
                    })
                    .collect::<Vec<_>>();
                Type::structural(
                    LiteralTypeDefinition::List(types),
                    TypeMetadata::default(),
                )
            }
            CoreValue::Map(map) => {
                let struct_types = map
                    .iter()
                    .map(|(key, value)| {
                        (
                            Type::from(
                                ValueContainer::from(key)
                                    .to_cloned_value()
                                    .borrow()
                                    .inner
                                    .clone(),
                            ),
                            Type::from(
                                value.to_cloned_value().borrow().inner.clone(),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                Type::structural(
                    LiteralTypeDefinition::Map(struct_types),
                    TypeMetadata::default(),
                )
            }
            e => unimplemented!("Type conversion not implemented for {}", e),
        }
    }
}
impl From<CoreValue> for Type {
    fn from(value: CoreValue) -> Self {
        Type::from(&value)
    }
}

#[cfg(feature = "compiler")]
impl TryFrom<&DatexExpressionData> for LiteralTypeDefinition {
    type Error = ();

    fn try_from(expr: &DatexExpressionData) -> Result<Self, Self::Error> {
        Ok(match expr {
            DatexExpressionData::Null => LiteralTypeDefinition::Null,
            DatexExpressionData::Boolean(b) => {
                LiteralTypeDefinition::Boolean(Boolean::from(*b))
            }
            DatexExpressionData::Text(s) => {
                LiteralTypeDefinition::Text(Text::from(s.clone()))
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
        Ok(Type::structural(
            LiteralTypeDefinition::try_from(expr)?,
            TypeMetadata::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use crate::{
        libs::core::{CoreLibTypeId, core_lib_type},
        types::r#type::{Type, TypeMetadata},
        values::{
            core_values::{
                integer::{Integer, typed_integer::TypedInteger},
                text::Text,
            },
            value_container::ValueContainer,
        },
    };

    #[test]
    fn test_match_equal_values() {
        // 1u8 matches 1u8
        assert!(Type::value_matches_type(
            &TypedInteger::from(1u8).into(),
            &Type::structural(1u8, TypeMetadata::default())
        ));

        // 1u16 matches 1u16
        assert!(Type::value_matches_type(
            &TypedInteger::from(1u16).into(),
            &Type::structural(1u16, TypeMetadata::default())
        ));

        // 1 matches 1
        assert!(Type::value_matches_type(
            &ValueContainer::from(Integer::from(1)),
            &Type::structural(Integer::from(1), TypeMetadata::default())
        ));

        // "test" matches "test"
        assert!(Type::value_matches_type(
            &ValueContainer::from(Text::from("test")),
            &Type::structural(Text::from("test"), TypeMetadata::default())
        ));
    }

    #[test]
    fn test_match_union() {
        // 1 matches (1 | 2 | 3)
        assert!(Type::value_matches_type(
            &ValueContainer::from(Integer::from(1)),
            &Type::union(
                vec![
                    Type::structural(Integer::from(1), TypeMetadata::default()),
                    Type::structural(Integer::from(2), TypeMetadata::default()),
                    Type::structural(Integer::from(3), TypeMetadata::default()),
                ],
                TypeMetadata::default()
            ),
        ))
    }

    #[test]
    fn type_matches_union_type() {
        // 1 matches (1 | 2 | 3)
        assert!(
            Type::structural(Integer::from(1), TypeMetadata::default())
                .matches_type(&Type::union(
                    vec![
                        Type::structural(
                            Integer::from(1),
                            TypeMetadata::default()
                        ),
                        Type::structural(
                            Integer::from(2),
                            TypeMetadata::default()
                        ),
                        Type::structural(
                            Integer::from(3),
                            TypeMetadata::default()
                        ),
                    ],
                    TypeMetadata::default()
                ))
        );

        // 1 matches integer | text
        assert!(
            Type::structural(Integer::from(1), TypeMetadata::default())
                .matches_type(&Type::union(
                    vec![
                        core_lib_type(CoreLibTypeId::Integer(None)),
                        core_lib_type(CoreLibTypeId::Text),
                    ],
                    TypeMetadata::default()
                ))
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
