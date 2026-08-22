use crate::{
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
        type_definition::{
            TypeDefinition,
            callable::CallableTypeDefinition,
            collection::{
                CollectionTypeDefinition,
                type_definition::{
                    list::ListCollectionTypeDefinition,
                    list_slice::ListSliceCollectionTypeDefinition,
                    map::MapCollectionTypeDefinition,
                },
            },
            impl_type::ImplTypeDefinition,
            intersection::IntersectionTypeDefinition,
            list::ListTypeDefinition,
            map::MapTypeDefinition,
            range::RangeTypeDefinition,
            tagged_type::TaggedTypeDefinition,
            union::UnionTypeDefinition,
        },
    },
};

/// Folds a type using the provided folder.
pub fn fold_type<F>(folder: &mut F, ty: &Type) -> Result<F::Output, F::Error>
where
    F: TypeFolder,
{
    match ty {
        Type::Definition(alias) => {
            let Some(name) = alias.reference_name() else {
                return fold_definition(folder, &alias.definition);
            };
            if folder.begin_named_alias(name)? {
                let definition = fold_definition(folder, &alias.definition)?;
                folder.end_named_alias(name, definition)?;
            }
            folder.fold_named_alias_reference(name)
        }

        Type::Entity(nominal) => folder.fold_entity_reference(nominal),
    }
}

/// Folds a type definition using the provided folder.
pub fn fold_definition<F>(
    folder: &mut F,
    definition: &TypeDefinition,
) -> Result<F::Output, F::Error>
where
    F: TypeFolder,
{
    match definition {
        TypeDefinition::Literal(literal) => folder.fold_literal(literal),
        TypeDefinition::List(list) => {
            let elements = list
                .iter()
                .map(|element| fold_type(folder, element))
                .collect::<Result<Vec<_>, _>>()?;

            folder.fold_list(list, elements)
        }
        TypeDefinition::Map(map) => {
            let entries = map
                .0
                .iter()
                .map(|(key, value)| {
                    let key = fold_type(folder, key)?;
                    let value = fold_type(folder, value)?;

                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, F::Error>>()?;

            folder.fold_map(map, entries)
        }
        TypeDefinition::Box(inner) => {
            let folded_inner = fold_type(folder, inner)?;
            folder.fold_nested(inner, folded_inner)
        }
        TypeDefinition::Union(union) => {
            let members = union
                .iter()
                .map(|member| fold_type(folder, member))
                .collect::<Result<Vec<_>, _>>()?;

            folder.fold_union(union, members)
        }
        TypeDefinition::Intersection(intersection) => {
            let members = intersection
                .iter()
                .map(|member| fold_type(folder, member))
                .collect::<Result<Vec<_>, _>>()?;

            folder.fold_intersection(intersection, members)
        }
        TypeDefinition::Callable(callable) => {
            let parameters = callable
                .parameters
                .iter()
                .map(|(name, ty)| {
                    fold_type(folder, ty).map(|ty| (name.clone(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let rest_parameter = callable
                .rest_parameter
                .as_ref()
                .map(|(name, ty)| {
                    fold_type(folder, ty).map(|ty| (name.clone(), ty))
                })
                .transpose()?;

            let return_type = callable
                .return_type
                .as_deref()
                .map(|ty| fold_type(folder, ty))
                .transpose()?;

            let yeet_type = callable
                .yeet_type
                .as_deref()
                .map(|ty| fold_type(folder, ty))
                .transpose()?;

            folder.fold_callable(
                callable,
                parameters,
                rest_parameter,
                return_type,
                yeet_type,
            )
        }
        TypeDefinition::Shared(shared) => folder.fold_shared_reference(shared),
        TypeDefinition::CoreType(core_type) => {
            folder.fold_core_type(*core_type)
        }
        TypeDefinition::Range(range) => {
            let start = fold_type(folder, &range.start)?;
            let end = fold_type(folder, &range.end)?;
            folder.fold_range(range, start, end)
        }
        TypeDefinition::Collection(collection) => match collection {
            CollectionTypeDefinition::List(list) => {
                let item = fold_type(folder, list.0.as_ref())?;
                folder.fold_list_collection(list, item)
            }
            CollectionTypeDefinition::ListSlice(
                list_slice_collection_type_definition,
            ) => {
                let item = fold_type(
                    folder,
                    &list_slice_collection_type_definition.item_type,
                )?;
                folder.fold_list_slice_collection(
                    list_slice_collection_type_definition,
                    item,
                )
            }
            CollectionTypeDefinition::Map(map_collection_type_definition) => {
                let key = fold_type(
                    folder,
                    &map_collection_type_definition.key_type,
                )?;
                let value = fold_type(
                    folder,
                    &map_collection_type_definition.value_type,
                )?;
                folder.fold_map_collection(
                    map_collection_type_definition,
                    key,
                    value,
                )
            }
            CollectionTypeDefinition::Range(range_type_definition) => {
                let start = fold_type(folder, &range_type_definition.start)?;
                let end = fold_type(folder, &range_type_definition.end)?;
                folder.fold_range(range_type_definition, start, end)
            }
        },
        TypeDefinition::ImplType(impl_type) => {
            let ty = fold_type(folder, &impl_type.inner_type)?;
            folder.fold_impl_type(impl_type, ty)
        }
        TypeDefinition::TaggedType(tagged) => {
            let payload = tagged
                .ty
                .as_deref()
                .map(|ty| fold_type(folder, ty))
                .transpose()?;
            folder.fold_tagged_type(tagged, payload)
        }
    }
}

/// A trait for folding DATEX types. This is a generalization of the visitor pattern that allows the folder to produce an output value and return a result, which can be used for error handling.
pub trait TypeFolder {
    type Output;
    type Error;

    /// Called when a named alias is encountered. If this returns true, the folder will fold the alias definition and call end_named_alias.
    /// If it returns false, the folder will skip the alias definition and directly call fold_named_alias_reference
    fn begin_named_alias(&mut self, name: &str) -> Result<bool, Self::Error>;

    /// Called after folding the definition of a named alias. The folded definition is provided as an argument.
    fn end_named_alias(
        &mut self,
        name: &str,
        definition: Self::Output,
    ) -> Result<(), Self::Error>;

    /// Called when a reference to a named alias is encountered. This will be called regardless of whether begin_named_alias returns true or false.
    fn fold_named_alias_reference(
        &mut self,
        name: &str,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a literal type definition is encountered.
    fn fold_literal(
        &mut self,
        literal: &LiteralTypeDefinition,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a list type definition is encountered. The folded elements of the list are provided as an argument.
    fn fold_list(
        &mut self,
        source: &ListTypeDefinition,
        elements: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a map type definition is encountered. The folded entries of the map are provided as an argument.
    fn fold_map(
        &mut self,
        source: &MapTypeDefinition,
        entries: Vec<(Self::Output, Self::Output)>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a nested type definition is encountered. The folded inner type is provided as an argument.
    fn fold_nested(
        &mut self,
        source: &Type,
        inner: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a union type definition is encountered. The folded members of the union are provided as an argument.
    fn fold_union(
        &mut self,
        source: &UnionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when an intersection type definition is encountered. The folded members of the intersection are provided as an argument.
    fn fold_intersection(
        &mut self,
        source: &IntersectionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a callable type definition is encountered. The folded parameter types, rest parameter type, return type and yeet type are provided as arguments.
    fn fold_callable(
        &mut self,
        source: &CallableTypeDefinition,
        parameters: Vec<(Option<String>, Self::Output)>,
        rest_parameter: Option<(Option<String>, Self::Output)>,
        return_type: Option<Self::Output>,
        yeet_type: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a reference to a shared container is encountered. The shared container is provided as an argument.
    fn fold_shared_reference(
        &mut self,
        shared: &SharedContainerContainingType,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a reference to a shared container containing a entity type is encountered. The shared container is provided as an argument.
    fn fold_entity_reference(
        &mut self,
        entity: &SharedContainerContainingEntityType,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a reference to a core type is encountered. The core type ID is provided as an argument.
    fn fold_core_type(
        &mut self,
        core_type: CoreLibTypeId,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a tagged type definition is encountered. The folded payload type is provided as an argument, or None if the tagged type has no payload.
    fn fold_tagged_type(
        &mut self,
        source: &TaggedTypeDefinition,
        payload: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a range type definition is encountered. The folded start and end types are provided as arguments.
    fn fold_range(
        &mut self,
        source: &RangeTypeDefinition,
        start: Self::Output,
        end: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when an impl type definition is encountered. The folded inner type is provided as an argument.
    fn fold_impl_type(
        &mut self,
        source: &ImplTypeDefinition,
        ty: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a list collection type definition is encountered. The folded item type is provided as an argument.
    fn fold_list_collection(
        &mut self,
        source: &ListCollectionTypeDefinition,
        item: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a list slice collection type definition is encountered. The folded item type is provided as an argument.
    fn fold_list_slice_collection(
        &mut self,
        source: &ListSliceCollectionTypeDefinition,
        item: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when a map collection type definition is encountered. The folded key and value types are provided as arguments.
    fn fold_map_collection(
        &mut self,
        source: &MapCollectionTypeDefinition,
        key: Self::Output,
        value: Self::Output,
    ) -> Result<Self::Output, Self::Error>;
}
