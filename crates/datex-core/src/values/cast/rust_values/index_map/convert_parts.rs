use core::hash::Hash;
use indexmap::IndexMap;
use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::traits::convert_value_container::ConvertValueContainer;

impl<K: ConvertValueContainer, V: ConvertValueContainer> IntoParts for IndexMap<K, V> {
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>> where Self: 'a, {
        Some(Parts::Map(Box::new(
            self
                .into_iter()
                .map(move |(key, value)| (key.to_value_container(cache), value.to_value_container(cache)))
        )))
    }

    fn as_parts<'a>(&'a self, cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        Some(BorrowedParts::Map(Box::new(
            self.iter()
                .map(move |(key, value)| (key.as_borrowed_value_container(cache), value.as_borrowed_value_container(cache)))
        )))
    }
}


impl<K: ConvertValueContainer + Eq + Hash, V: ConvertValueContainer> FromParts for IndexMap<K, V> {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized
    {
        match parts {
            Parts::Map(iter) => {
                let mut map = IndexMap::new();
                for (key, value) in iter {
                    map.insert(K::try_from_value_container(key).map_err(|_| ())?, V::try_from_value_container(value).map_err(|_| ())?);
                }
                Ok(map)
            }
            _ => Err(()),
        }
    }
}