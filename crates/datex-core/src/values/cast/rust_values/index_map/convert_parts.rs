use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    random::RandomState,
    traits::{
        convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
        convert_value_container::ConvertValueContainer,
    },
    values::{core_values::map::Map},
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K: ConvertValueContainer, V: ConvertValueContainer> IntoParts
    for IndexMap<K, V, RandomState>
{
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        Ok(self
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_value_container(cache),
                    value.to_value_container(cache),
                )
            })
            .collect::<Map>()
        )
    }
}

impl<K: ConvertValueContainer + Eq + Hash, V: ConvertValueContainer> FromParts
    for IndexMap<K, V, RandomState>
{
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match parts {
            Parts::Map(iter) => {
                let mut map = IndexMap::default();
                for (key, value) in iter {
                    map.insert(
                        K::try_from_value_container(key).map_err(|_| ())?,
                        V::try_from_value_container(value).map_err(|_| ())?,
                    );
                }
                Ok(map)
            }
            _ => Err(()),
        }
    }
}
