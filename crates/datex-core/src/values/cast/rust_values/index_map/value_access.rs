use crate::{
    random::RandomState,
    shared_values::errors::{AccessError, KeyNotFoundError},
    traits::{
        convert_value_container::ConvertValueContainer,
        value_access::ValueAccess,
    },
    values::{
        core_values::map::MapAccessError,
        value_container::{
            ValueContainer,
            borrowed_value_container::{
                BorrowedValueContainer, BorrowedValueContainerMut,
            },
        },
    },
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K, V> ValueAccess for IndexMap<K, V, RandomState>
where
    K: ConvertValueContainer + Eq + Hash,
    V: ConvertValueContainer,
{
    fn try_get_property(
        &self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        let property =
            K::try_borrow_from_value_container(key).map_err(|_| {
                AccessError::MapAccessError(MapAccessError::KeyNotFound(
                    KeyNotFoundError::new(key.clone()),
                ))
            })?;

        let value = self
            .get(property)
            .ok_or_else(|| {
                AccessError::MapAccessError(MapAccessError::KeyNotFound(
                    KeyNotFoundError::new(key.clone()),
                ))
            })?
            .as_borrowed_value_container();
        Ok(value)
    }

    fn try_get_property_mut(
        &mut self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        let property =
            K::try_borrow_from_value_container(key).map_err(|_| {
                AccessError::MapAccessError(MapAccessError::KeyNotFound(
                    KeyNotFoundError::new(key.clone()),
                ))
            })?;

        let value = self
            .get_mut(property)
            .ok_or_else(|| {
                AccessError::MapAccessError(MapAccessError::KeyNotFound(
                    KeyNotFoundError::new(key.clone()),
                ))
            })?
            .as_borrowed_value_container_mut();
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::values::core_values::text::Text;

    use super::*;
    use crate::prelude::*;
    #[test]
    fn test_try_get_property() {
        let mut map: IndexMap<Text, ValueContainer, RandomState> =
            IndexMap::default();
        let key = ValueContainer::from(Text::String("key".to_string()));
        let value = ValueContainer::from("value");
        map.insert(Text::String("key".to_string()), value.clone());
        let result = map.try_get_property(&key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value.as_borrowed_value_container());
    }

    #[test]
    fn test_try_get_property_mut() {
        let mut map: IndexMap<Text, ValueContainer, RandomState> =
            IndexMap::default();
        let key = ValueContainer::from(Text::String("key".to_string()));
        let mut value = ValueContainer::from(Text::String("value".to_string()));
        map.insert(Text::String("key".to_string()), value.clone());
        let result = map.try_get_property_mut(&key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value.as_borrowed_value_container_mut());
    }
}
