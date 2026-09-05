//! This module contains the implementation of the [Map] struct, which represents a map of key-value pairs in the type system.
//! The [Map] struct supports different internal representations for different use cases, such as a dynamic map that allows any keys and values, a structural map with fixed keys and values, and a record-like map with string keys and value containers as values.

pub mod update_handler;

use crate::{
    collections::HashMap,
    prelude::*,
    random::RandomState,
    values::{
        core_value::CoreValue,
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};
pub mod equality;
use crate::shared_values::errors::KeyNotFoundError;
use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    result::Result,
};
mod child_iterator;
pub mod classification;
mod convert_parts;
mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
mod get_datex_type;
pub mod local_child_path_resolver;
pub mod serde_dif;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
pub mod updates;
mod value_access;
use crate::{
    shared_values::base_shared_value_container::observers::TransceiverId,
    utils::impl_display_for_datex_value::impl_display_for_datex_value,
    value_updates::update_handler::{
        InternalMutabilityUpdateHandler, UpdateCallbackData,
    },
    values::value_container::value_key::ValueKey,
};
use indexmap::{IndexMap, map::MutableKeys};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapEntries {
    // most general case, allows all types of keys and values, and dynamic size
    Dynamic(IndexMap<ValueContainer, ValueContainer, RandomState>),
    // for fixed-size maps with known keys and values on construction
    Structural(Vec<(ValueContainer, ValueContainer)>),
    // for maps with string keys
    StructuralWithStringKeys(Vec<(String, ValueContainer)>), // for structural maps with string keys
}

impl From<MapEntries> for Map {
    fn from(entries: MapEntries) -> Self {
        Map {
            entries,
            update_callback_data: None,
        }
    }
}

#[derive(Debug)]
pub struct Map {
    entries: MapEntries,
    update_callback_data: Option<UpdateCallbackData>,
}

impl Clone for Map {
    fn clone(&self) -> Self {
        Map {
            entries: self.entries.clone(),
            update_callback_data: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapAccessError {
    KeyNotFound(KeyNotFoundError),
    Immutable,
}

#[derive(Debug, Clone)]
pub struct UnexpectedPropertyError {
    pub key: String,
}

impl core::fmt::Display for UnexpectedPropertyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unexpected property: {:?}", self.key)
    }
}

impl Display for MapAccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapAccessError::KeyNotFound(err) => {
                core::write!(f, "{}", err)
            }
            MapAccessError::Immutable => {
                core::write!(f, "Map is immutable")
            }
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        MapEntries::Dynamic(IndexMap::default()).into()
    }
}

impl Map {
    pub fn structural_with_string_keys(
        entries: Vec<(String, ValueContainer)>,
    ) -> Self {
        MapEntries::StructuralWithStringKeys(entries).into()
    }

    pub fn structural(entries: Vec<(ValueContainer, ValueContainer)>) -> Self {
        MapEntries::Structural(entries).into()
    }

    pub fn dynamic(
        entries: IndexMap<ValueContainer, ValueContainer, RandomState>,
    ) -> Self {
        MapEntries::Dynamic(entries).into()
    }

    pub fn new(
        entries: IndexMap<ValueContainer, ValueContainer, RandomState>,
    ) -> Self {
        Self::dynamic(entries)
    }

    pub fn new_structural_with_string_keys(
        entries: Vec<(String, ValueContainer)>,
    ) -> Self {
        MapEntries::StructuralWithStringKeys(entries).into()
    }

    pub fn is_structural(&self) -> bool {
        core::matches!(
            &self.entries,
            MapEntries::StructuralWithStringKeys(_) | MapEntries::Structural(_)
        )
    }

    pub fn size(&self) -> usize {
        match &self.entries {
            MapEntries::Dynamic(map) => map.len(),
            MapEntries::Structural(vec) => vec.len(),
            MapEntries::StructuralWithStringKeys(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Gets a value in the map by reference.
    /// Returns None if the key is not found.
    pub fn try_get<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<&ValueContainer, KeyNotFoundError> {
        let key = key.into();
        match &self.entries {
            MapEntries::Dynamic(map) => {
                key.with_value_container(|key| map.get(key))
            }
            MapEntries::Structural(vec) => key.with_value_container(|key| {
                vec.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }),
            MapEntries::StructuralWithStringKeys(vec) => {
                // only works if key is a string
                if let Some(string) = key.try_as_text() {
                    vec.iter().find(|(k, _)| k == string).map(|(_, v)| v)
                } else {
                    None
                }
            }
        }
        .ok_or_else(|| KeyNotFoundError::new(key.into()))
    }

    pub fn try_get_mut<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<&mut ValueContainer, KeyNotFoundError> {
        let key = key.into();
        match &mut self.entries {
            MapEntries::Dynamic(map) => {
                key.with_value_container(|key| map.get_mut(key))
            }
            MapEntries::Structural(vec) => key.with_value_container(|key| {
                vec.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
            }),
            MapEntries::StructuralWithStringKeys(vec) => {
                // only works if key is a string
                if let Some(string) = key.try_as_text() {
                    vec.iter_mut().find(|(k, _)| k == string).map(|(_, v)| v)
                } else {
                    None
                }
            }
        }
        .ok_or_else(|| KeyNotFoundError::new(key.into()))
    }

    /// Checks if the map contains the given key.
    pub fn has<'a>(&self, key: impl Into<BorrowedValueKey<'a>>) -> bool {
        match &self.entries {
            MapEntries::Dynamic(map) => {
                key.into().with_value_container(|key| map.contains_key(key))
            }
            MapEntries::Structural(vec) => key
                .into()
                .with_value_container(|key| vec.iter().any(|(k, _)| k == key)),
            MapEntries::StructuralWithStringKeys(vec) => {
                // only works if key is a string
                if let Some(string) = key.into().try_as_text() {
                    vec.iter().any(|(k, _)| k == string)
                } else {
                    false
                }
            }
        }
    }

    /// Ensures that the map only contains the given allowed keys, returning an error if any unexpected keys are found.
    /// This should only be called on structural maps, as it relies on the assumption that the map will not be modified after construction.
    pub fn ensure_only_properties(
        &self,
        allowed: &[&str],
    ) -> Result<(), UnexpectedPropertyError> {
        match &self.entries {
            MapEntries::Structural(_) => {
                for (key, _) in self.iter() {
                    if let BorrowedMapKey::Text(text) = key {
                        if !allowed.contains(&text) {
                            return Err(UnexpectedPropertyError {
                                key: text.to_string(),
                            });
                        }
                    } else {
                        return Err(UnexpectedPropertyError {
                            key: format!("{key}"),
                        });
                    }
                }
            }
            MapEntries::Dynamic(entries) => {
                for (key, _) in entries {
                    if let ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) = key
                    {
                        if !allowed.contains(&text.0.as_str()) {
                            return Err(UnexpectedPropertyError {
                                key: text.0.clone(),
                            });
                        }
                    } else {
                        return Err(UnexpectedPropertyError {
                            key: format!("{key}"),
                        });
                    }
                }
            }
            MapEntries::StructuralWithStringKeys(vec) => {
                for (key, _) in vec {
                    if !allowed.contains(&key.as_str()) {
                        return Err(UnexpectedPropertyError {
                            key: key.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn iter(&self) -> MapIterator<'_> {
        MapIterator {
            map: self,
            index: 0,
        }
    }

    pub(crate) fn iter_mut(&mut self) -> MapMutIterator<'_> {
        self.into_iter()
    }

    /// Returns an iterator over the local values in the map,
    /// skipping any children that have a [ValueContainer::Shared] value
    pub fn iter_local_values_mut(
        &mut self,
    ) -> impl Iterator<Item = (BorrowedMutMapKey<'_>, &mut Value)> {
        self.iter_mut().filter_map(|(key, item)| {
            if let ValueContainer::Local(local_value) = item {
                Some((key, local_value))
            } else {
                None
            }
        })
    }

    /// Sets a value in the map, panicking if it fails.
    pub(crate) fn set_unchecked<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        value: impl Into<ValueContainer>,
    ) {
        self.try_set(key, value)
            .expect("Setting value in map failed");
    }

    /// Removes a key from the map, returning the value if it existed.
    pub fn try_delete<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, MapAccessError> {
        self.try_delete_with_source(key, Some(TransceiverId::Local))
    }

    /// Removes a key from the map, returning the value if it existed.
    /// Also works for structural maps, but creates a map that no longer matches the assumed type.
    /// # Safety
    /// The map should no longer be used after this operation.
    pub unsafe fn try_delete_unchecked<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, KeyNotFoundError> {
        unsafe {
            self.try_delete_unchecked_with_source(
                key,
                Some(TransceiverId::Local),
            )
        }
    }

    /// Clears all entries in the map, returning an error if the map is not dynamic.
    pub fn try_clear(&mut self) -> Result<ValueContainer, MapAccessError> {
        self.try_clear_with_source(Some(TransceiverId::Local))
    }

    /// Sets a value in the map, returning an error if it fails.
    /// This is the preferred way to set values in the map.
    pub(crate) fn try_set<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        value: impl Into<ValueContainer>,
    ) -> Result<Option<ValueContainer>, KeyNotFoundError> {
        self.try_set_with_source(key, value.into(), Some(TransceiverId::Local))
    }
}

#[derive(Clone)]
pub enum BorrowedMapKey<'a> {
    Text(&'a str),
    Value(&'a ValueContainer),
}

impl<'a> From<&'a MapKey> for BorrowedMapKey<'a> {
    fn from(key: &'a MapKey) -> Self {
        match key {
            MapKey::Text(text) => BorrowedMapKey::Text(text),
            MapKey::Value(value) => BorrowedMapKey::Value(value),
        }
    }
}

impl<'a> From<BorrowedMapKey<'a>> for ValueContainer {
    fn from(key: BorrowedMapKey) -> Self {
        match key {
            BorrowedMapKey::Text(text) => {
                ValueContainer::Local(Value::from(text))
            }
            BorrowedMapKey::Value(value) => value.clone(),
        }
    }
}

impl Hash for BorrowedMapKey<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BorrowedMapKey::Text(text) => text.hash(state),
            BorrowedMapKey::Value(value) => value.hash(state),
        }
    }
}

impl Display for BorrowedMapKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // TODO #331: escape string
            BorrowedMapKey::Text(string) => core::write!(f, "\"{}\"", string),
            BorrowedMapKey::Value(value) => core::write!(f, "{value}"),
        }
    }
}

pub enum BorrowedMutMapKey<'a> {
    Text(&'a mut str),
    Value(&'a mut ValueContainer),
}

impl<'a> From<&'a mut MapKey> for BorrowedMutMapKey<'a> {
    fn from(key: &'a mut MapKey) -> Self {
        match key {
            MapKey::Text(text) => BorrowedMutMapKey::Text(text),
            MapKey::Value(value) => BorrowedMutMapKey::Value(value),
        }
    }
}
impl<'a> From<BorrowedMutMapKey<'a>> for MapKey {
    fn from(key: BorrowedMutMapKey<'a>) -> Self {
        match key {
            BorrowedMutMapKey::Text(text) => MapKey::Text(text.to_string()),
            BorrowedMutMapKey::Value(value) => MapKey::Value(value.clone()),
        }
    }
}

impl<'a> From<BorrowedMutMapKey<'a>> for ValueContainer {
    fn from(key: BorrowedMutMapKey) -> Self {
        match key {
            BorrowedMutMapKey::Text(text) => {
                ValueContainer::Local(Value::from(text as &_))
            }
            BorrowedMutMapKey::Value(value) => value.clone(),
        }
    }
}

impl Hash for BorrowedMutMapKey<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BorrowedMutMapKey::Text(text) => text.hash(state),
            BorrowedMutMapKey::Value(value) => value.hash(state),
        }
    }
}

impl Display for BorrowedMutMapKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // TODO #331: escape string
            BorrowedMutMapKey::Text(string) => {
                core::write!(f, "\"{}\"", string)
            }
            BorrowedMutMapKey::Value(value) => core::write!(f, "{value}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum MapKey {
    Text(String),
    Value(ValueContainer),
}

impl From<MapKey> for ValueContainer {
    fn from(key: MapKey) -> Self {
        match key {
            MapKey::Text(text) => ValueContainer::Local(Value::from(text)),
            MapKey::Value(value) => value,
        }
    }
}

impl From<MapKey> for ValueKey {
    fn from(key: MapKey) -> Self {
        match key {
            MapKey::Text(text) => ValueKey::Text(text),
            MapKey::Value(value) => ValueKey::Value(value),
        }
    }
}

impl<'a> From<&'a MapKey> for BorrowedValueKey<'a> {
    fn from(key: &'a MapKey) -> Self {
        match key {
            MapKey::Text(text) => BorrowedValueKey::Text(Cow::Borrowed(text)),
            MapKey::Value(value) => {
                BorrowedValueKey::Value(Cow::Borrowed(value))
            }
        }
    }
}

impl Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapKey::Text(text) => core::write!(f, "{text}"),
            MapKey::Value(value) => core::write!(f, "{value}"),
        }
    }
}

pub struct MapIterator<'a> {
    map: &'a Map,
    index: usize,
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = (BorrowedMapKey<'a>, &'a ValueContainer);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.map.entries {
            MapEntries::Dynamic(map) => {
                let item = map.iter().nth(self.index);
                self.index += 1;
                item.map(|(k, v)| {
                    let key = match k {
                        ValueContainer::Local(Value {
                            inner: CoreValue::Text(text),
                            ..
                        }) => BorrowedMapKey::Text(&text.0),
                        _ => BorrowedMapKey::Value(k),
                    };
                    (key, v)
                })
            }
            MapEntries::Structural(vec) => {
                if self.index < vec.len() {
                    let item = &vec[self.index];
                    self.index += 1;
                    let key = match &item.0 {
                        ValueContainer::Local(Value {
                            inner: CoreValue::Text(text),
                            ..
                        }) => BorrowedMapKey::Text(&text.0),
                        _ => BorrowedMapKey::Value(&item.0),
                    };
                    Some((key, &item.1))
                } else {
                    None
                }
            }
            MapEntries::StructuralWithStringKeys(vec) => {
                if self.index < vec.len() {
                    let item = &vec[self.index];
                    self.index += 1;
                    Some((BorrowedMapKey::Text(&item.0), &item.1))
                } else {
                    None
                }
            }
        }
    }
}

pub enum MapMutIterator<'a> {
    Dynamic(indexmap::map::IterMut2<'a, ValueContainer, ValueContainer>),
    Fixed(core::slice::IterMut<'a, (ValueContainer, ValueContainer)>),
    Structural(core::slice::IterMut<'a, (String, ValueContainer)>),
}

impl<'a> Iterator for MapMutIterator<'a> {
    type Item = (BorrowedMutMapKey<'a>, &'a mut ValueContainer);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MapMutIterator::Dynamic(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => BorrowedMutMapKey::Text(&mut text.0),
                    _ => BorrowedMutMapKey::Value(k),
                };
                (key, v)
            }),
            MapMutIterator::Fixed(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => BorrowedMutMapKey::Text(&mut text.0),
                    _ => BorrowedMutMapKey::Value(k),
                };
                (key, v)
            }),
            MapMutIterator::Structural(iter) => {
                iter.next().map(|(k, v)| (BorrowedMutMapKey::Text(k), v))
            }
        }
    }
}

pub enum IntoMapIterator {
    Dynamic(indexmap::map::IntoIter<ValueContainer, ValueContainer>),
    Fixed(vec::IntoIter<(ValueContainer, ValueContainer)>),
    Structural(vec::IntoIter<(String, ValueContainer)>),
}

impl Iterator for IntoMapIterator {
    type Item = (MapKey, ValueContainer);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoMapIterator::Dynamic(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => MapKey::Text(text.0),
                    _ => MapKey::Value(k),
                };
                (key, v)
            }),
            IntoMapIterator::Fixed(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => MapKey::Text(text.0),
                    _ => MapKey::Value(k),
                };
                (key, v)
            }),
            IntoMapIterator::Structural(iter) => {
                iter.next().map(|(k, v)| (MapKey::Text(k), v))
            }
        }
    }
}

impl_display_for_datex_value!(
    Map,
    impl core::fmt::Display for Map {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result  {
            core::write!(f, "{{")?;
            for (i, (key, value)) in self.iter().enumerate() {
                if i > 0 {
                    core::write!(f, ", ")?;
                }
                core::write!(f, "{key}: {value}")?;
            }
            core::write!(f, "}}")
        }
    }
);

impl<K, V> From<HashMap<K, V>> for Map
where
    K: Into<ValueContainer>,
    V: Into<ValueContainer>,
{
    fn from(map: HashMap<K, V>) -> Self {
        Map::new(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl IntoIterator for Map {
    type Item = (MapKey, ValueContainer);
    type IntoIter = IntoMapIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self.entries {
            MapEntries::Dynamic(map) => {
                IntoMapIterator::Dynamic(map.into_iter())
            }
            MapEntries::Structural(vec) => {
                IntoMapIterator::Fixed(vec.into_iter())
            }
            MapEntries::StructuralWithStringKeys(vec) => {
                IntoMapIterator::Structural(vec.into_iter())
            }
        }
    }
}

impl<'a> IntoIterator for &'a mut Map {
    type Item = (BorrowedMutMapKey<'a>, &'a mut ValueContainer);
    type IntoIter = MapMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &mut self.entries {
            MapEntries::Dynamic(map) => {
                MapMutIterator::Dynamic(map.iter_mut2())
            }
            MapEntries::Structural(vec) => {
                MapMutIterator::Fixed(vec.iter_mut())
            }
            MapEntries::StructuralWithStringKeys(vec) => {
                MapMutIterator::Structural(vec.iter_mut())
            }
        }
    }
}

impl From<Vec<(ValueContainer, ValueContainer)>> for Map {
    /// Create a dynamic map from a vector of value containers.
    fn from(vec: Vec<(ValueContainer, ValueContainer)>) -> Self {
        Map::new(vec.into_iter().collect())
    }
}

impl From<Vec<(String, ValueContainer)>> for Map {
    /// Create a dynamic map from a vector of string keys and value containers.
    fn from(vec: Vec<(String, ValueContainer)>) -> Self {
        Map::new(
            vec.into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect::<IndexMap<ValueContainer, ValueContainer, RandomState>>(),
        )
    }
}

impl From<Vec<(MapKey, ValueContainer)>> for Map {
    fn from(vec: Vec<(MapKey, ValueContainer)>) -> Self {
        let has_only_text_keys = vec.iter().all(|(k, _)| {
            matches!(k, MapKey::Text(_))
                || matches!(
                    k,
                    MapKey::Value(ValueContainer::Local(Value {
                        inner: CoreValue::Text(_),
                        ..
                    }))
                )
        });
        if has_only_text_keys {
            let mut entries: Vec<(String, ValueContainer)> =
                Vec::with_capacity(vec.len());
            for (k, v) in vec {
                match k {
                    MapKey::Text(text) => {
                        entries.push((text, v));
                    }
                    MapKey::Value(value) => {
                        if let ValueContainer::Local(Value {
                            inner: CoreValue::Text(text),
                            ..
                        }) = value
                        {
                            entries.push((text.0, v));
                        } else {
                            unreachable!(); // already checked above
                        }
                    }
                }
            }
            MapEntries::StructuralWithStringKeys(entries).into()
        } else {
            let mut map = Map::default();
            for (k, v) in vec {
                map.set_unchecked(&k, v);
            }
            map
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Map
where
    K: Into<ValueContainer>,
    V: Into<ValueContainer>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        MapEntries::Dynamic(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
        .into()
    }
}

impl<K, V> From<IndexMap<K, V, RandomState>> for Map
where
    K: Into<ValueContainer>,
    V: Into<ValueContainer>,
{
    fn from(map: IndexMap<K, V, RandomState>) -> Self {
        Map::new(
            map.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect::<IndexMap<ValueContainer, ValueContainer, RandomState>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::{
            cache::shared_references_cache::SharedReferencesCache,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{
            SelfOwnedPointerAddress, SelfOwnedSharedContainer, SharedContainer,
            SharedContainerMutability,
            base_shared_value_container::BaseSharedValueContainer,
        },
        values::{
            core_values::{
                decimal::{Decimal, typed_decimal::TypedDecimal},
                map::Map,
            },
            value_container::ValueContainer,
        },
    };

    #[test]
    fn map() {
        let mut map = Map::default();
        map.set_unchecked("key1", 42);
        map.set_unchecked("key2", "value2");
        assert_eq!(map.size(), 2);
        assert_eq!(map.try_get("key1").unwrap().to_string(), "42");
        assert_eq!(map.try_get("key2").unwrap().to_string(), "\"value2\"");
        assert_eq!(map.to_string(), r#"{"key1": 42, "key2": "value2"}"#);
    }

    #[test]
    fn duplicate_keys() {
        let mut map = Map::default();
        map.set_unchecked("key1", 42);
        map.set_unchecked("key1", "new_value");
        assert_eq!(map.size(), 1);
        assert_eq!(map.try_get("key1").unwrap().to_string(), "\"new_value\"");
    }

    #[test]
    fn ref_keys() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let mut map = Map::default();
        let key = ValueContainer::Shared(
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                address_provider,
            ),
        );

        map.set_unchecked(key.clone(), "value");
        // same reference should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&key));
        assert_eq!(map.try_get(&key).unwrap().to_string(), "\"value\"");

        // new reference with same value should not be found
        let new_key = ValueContainer::Shared(
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                address_provider,
            ),
        );
        assert!(!map.has(&new_key));
        assert!(map.try_get(&new_key).is_err());
    }

    #[test]
    fn decimal_nan_value_key() {
        let mut map = Map::default();
        let nan_value = ValueContainer::from(Decimal::Nan);
        map.set_unchecked(&nan_value, "value");
        // same NaN value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&nan_value));

        // new NaN value should also be found
        let new_nan_value = ValueContainer::from(Decimal::Nan);
        assert!(map.has(&new_nan_value));

        // adding new_nan_value should not increase size
        map.set_unchecked(&new_nan_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn float_nan_value_key() {
        let mut map = Map::default();
        let nan_value = ValueContainer::from(f64::NAN);
        map.set_unchecked(&nan_value, "value");
        // same NaN value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&nan_value));

        // new f64 NaN value should also be found
        let new_nan_value = ValueContainer::from(f64::NAN);
        assert!(map.has(&new_nan_value));

        // new f32 NaN should not be found
        let float32_nan_value = ValueContainer::from(f32::NAN);
        assert!(!map.has(&float32_nan_value));

        // adding new_nan_value should not increase size
        map.set_unchecked(&new_nan_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn decimal_zero_value_key() {
        let mut map = Map::default();
        let zero_value = ValueContainer::from(Decimal::Zero);
        map.set_unchecked(&zero_value, "value");
        // same Zero value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&zero_value));

        // new Zero value should also be found
        let new_zero_value = ValueContainer::from(Decimal::Zero);
        assert!(map.has(&new_zero_value));

        // new NegZero value should also be found
        let neg_zero_value = ValueContainer::from(Decimal::NegZero);
        assert!(map.has(&neg_zero_value));

        // adding neg_zero_value should not increase size
        map.set_unchecked(&neg_zero_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn float_zero_value_key() {
        let mut map = Map::default();
        let zero_value = ValueContainer::from(0.0f64);
        map.set_unchecked(&zero_value, "value");
        // same 0.0 value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&zero_value));
        // new 0.0 value should also be found
        let new_zero_value = ValueContainer::from(0.0f64);
        assert!(map.has(&new_zero_value));
        // new -0.0 value should also be found
        let neg_zero_value = ValueContainer::from(-0.0f64);
        assert!(map.has(&neg_zero_value));

        // adding neg_zero_value should not increase size
        map.set_unchecked(&neg_zero_value, "new_value");
        assert_eq!(map.size(), 1);

        // new 0.0f32 value should not be found
        let float32_zero_value = ValueContainer::from(0.0f32);
        assert!(!map.has(&float32_zero_value));
    }

    #[test]
    fn typed_big_decimal_key() {
        let mut map = Map::default();
        let zero_big_decimal =
            ValueContainer::from(TypedDecimal::Decimal(Decimal::Zero));
        map.set_unchecked(&zero_big_decimal, "value");
        // same Zero value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&zero_big_decimal));
        // new Zero value should also be found
        let new_zero_big_decimal =
            ValueContainer::from(TypedDecimal::Decimal(Decimal::Zero));
        assert!(map.has(&new_zero_big_decimal));
        // new NegZero value should also be found
        let neg_zero_big_decimal =
            ValueContainer::from(TypedDecimal::Decimal(Decimal::NegZero));
        assert!(map.has(&neg_zero_big_decimal));

        // adding neg_zero_big_decimal should not increase size
        map.set_unchecked(&neg_zero_big_decimal, "new_value");
        assert_eq!(map.size(), 1);
    }
}
