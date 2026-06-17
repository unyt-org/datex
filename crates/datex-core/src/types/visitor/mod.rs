use crate::{
    libs::core::type_id::CoreLibTypeId,
    prelude::*,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
        type_definition::{
            TypeDefinition,
            callable::CallableTypeDefinition,
            collection::{
                CollectionTypeDefinition,
                type_definition::list::ListCollectionTypeDefinition,
            },
            intersection::IntersectionTypeDefinition,
            list::ListTypeDefinition,
            map::MapTypeDefinition,
            tagged_type::TaggedTypeDefinition,
            union::UnionTypeDefinition,
        },
    },
};
pub fn fold_type<F>(folder: &mut F, ty: &Type) -> Result<F::Output, F::Error>
where
    F: TypeFolder,
{
    match ty {
        Type::Alias(alias) => {
            let Some(name) = alias.reference_name.as_deref() else {
                return fold_definition(folder, &alias.definition);
            };
            if folder.begin_named_alias(name)? {
                let definition = fold_definition(folder, &alias.definition)?;
                folder.end_named_alias(name, definition)?;
            }
            folder.fold_named_alias_reference(name)
        }

        Type::Nominal(nominal) => folder.fold_nominal_reference(nominal),
    }
}

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

        TypeDefinition::Nested(inner) => {
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
                .parameter_types
                .iter()
                .map(|(name, ty)| {
                    fold_type(folder, ty).map(|ty| (name.clone(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let rest_parameter = callable
                .rest_parameter_type
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

        TypeDefinition::Range(_range) => {
            todo!("Add fold_range()")
        }

        TypeDefinition::Collection(collection) => match collection {
            CollectionTypeDefinition::List(list) => {
                let item = fold_type(folder, list.0.as_ref())?;
                folder.fold_list_collection(list, item)
            }
            _ => todo!("Add fold_collection() for other collection types"),
        },

        TypeDefinition::ImplType(_impl_type) => {
            todo!("Add fold_impl_type()")
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

pub trait TypeFolder {
    type Output;
    type Error;

    fn begin_named_alias(&mut self, name: &str) -> Result<bool, Self::Error>;

    fn end_named_alias(
        &mut self,
        name: &str,
        definition: Self::Output,
    ) -> Result<(), Self::Error>;

    fn fold_named_alias_reference(
        &mut self,
        name: &str,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_literal(
        &mut self,
        literal: &LiteralTypeDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_list(
        &mut self,
        source: &ListTypeDefinition,
        elements: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_map(
        &mut self,
        source: &MapTypeDefinition,
        entries: Vec<(Self::Output, Self::Output)>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_nested(
        &mut self,
        source: &Type,
        inner: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_union(
        &mut self,
        source: &UnionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_intersection(
        &mut self,
        source: &IntersectionTypeDefinition,
        members: Vec<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_callable(
        &mut self,
        source: &CallableTypeDefinition,
        parameters: Vec<(Option<String>, Self::Output)>,
        rest_parameter: Option<(Option<String>, Self::Output)>,
        return_type: Option<Self::Output>,
        yeet_type: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_shared_reference(
        &mut self,
        shared: &SharedContainerContainingType,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_nominal_reference(
        &mut self,
        nominal: &SharedContainerContainingNominalType,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_core_type(
        &mut self,
        core_type: CoreLibTypeId,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_tagged_type(
        &mut self,
        source: &TaggedTypeDefinition,
        payload: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;

    fn fold_list_collection(
        &mut self,
        source: &ListCollectionTypeDefinition,
        item: Self::Output,
    ) -> Result<Self::Output, Self::Error>;
}
