use crate::{
    collections::HashMap,
    prelude::*,
    random::RandomState,
    traits::structural_eq::StructuralEq,
    values::{
        core_value::CoreValue,
        value::Value,
        value_container::{ValueContainer, BorrowedValueKey},
    },
};

use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    result::Result,
};
use indexmap::IndexMap;
use crate::shared_values::errors::KeyNotFoundError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Map {
    // most general case, allows all types of keys and values, and dynamic size
    Dynamic(IndexMap<ValueContainer, ValueContainer, RandomState>),
    // for fixed-size maps with known keys and values on construction
    Structural(Vec<(ValueContainer, ValueContainer)>),
    // for maps with string keys
    StructuralWithStringKeys(Vec<(String, ValueContainer)>), // for structural maps with string keys
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapAccessError {
    KeyNotFound(KeyNotFoundError),
    Immutable,
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
        Map::Dynamic(IndexMap::default())
    }
}

impl Map {
    pub fn new(
        entries: IndexMap<ValueContainer, ValueContainer, RandomState>,
    ) -> Self {
        Map::Dynamic(entries)
    }

    pub fn is_structural(&self) -> bool {
        core::matches!(
            self,
            Map::StructuralWithStringKeys(_) | Map::Structural(_)
        )
    }

    pub fn size(&self) -> usize {
        match self {
            Map::Dynamic(map) => map.len(),
            Map::Structural(vec) => vec.len(),
            Map::StructuralWithStringKeys(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Gets a value in the map by reference.
    /// Returns None if the key is not found.
    pub fn get<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<&ValueContainer, KeyNotFoundError> {
        let key = key.into();
        match self {
            Map::Dynamic(map) => key.with_value_container(|key| map.get(key)),
            Map::Structural(vec) => key.with_value_container(|key| {
                vec.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }),
            Map::StructuralWithStringKeys(vec) => {
                // only works if key is a string
                if let Some(string) = key.try_as_text() {
                    vec.iter().find(|(k, _)| k == string).map(|(_, v)| v)
                } else {
                    None
                }
            }
        }
        .ok_or_else(|| KeyNotFoundError { key: key.into() })
    }

    /// Checks if the map contains the given key.
    pub fn has<'a>(&self, key: impl Into<BorrowedValueKey<'a>>) -> bool {
        match self {
            Map::Dynamic(map) => {
                key.into().with_value_container(|key| map.contains_key(key))
            }
            Map::Structural(vec) => key
                .into()
                .with_value_container(|key| vec.iter().any(|(k, _)| k == key)),
            Map::StructuralWithStringKeys(vec) => {
                // only works if key is a string
                if let Some(string) = key.into().try_as_text() {
                    vec.iter().any(|(k, _)| k == string)
                } else {
                    false
                }
            }
        }
    }

    /// Removes a key from the map, returning the value if it existed.
    pub fn delete<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, MapAccessError> {
        let key = key.into();
        match self {
            Map::Dynamic(map) => key.with_value_container(|key| {
                map.shift_remove(key).ok_or_else(|| {
                    MapAccessError::KeyNotFound(KeyNotFoundError {
                        key: key.clone(),
                    })
                })
            }),
            Map::Structural(_) | Map::StructuralWithStringKeys(_) => {
                Err(MapAccessError::Immutable)
            }
        }
    }

    /// Clears all entries in the map, returning an error if the map is not dynamic.
    pub fn clear(&mut self) -> Result<(), MapAccessError> {
        match self {
            Map::Dynamic(map) => {
                map.clear();
                Ok(())
            }
            Map::Structural(_) | Map::StructuralWithStringKeys(_) => {
                Err(MapAccessError::Immutable)
            }
        }
    }

    /// Sets a value in the map, panicking if it fails.
    pub(crate) fn set<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        value: impl Into<ValueContainer>,
    ) {
        self.try_set(key, value)
            .expect("Setting value in map failed");
    }

    /// Sets a value in the map, returning an error if it fails.
    /// This is the preferred way to set values in the map.
    pub(crate) fn try_set<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        value: impl Into<ValueContainer>,
    ) -> Result<(), KeyNotFoundError> {
        let key = key.into();
        match self {
            Map::Dynamic(map) => {
                key.with_value_container(|key| {
                    map.insert(key.clone(), value.into());
                });
                Ok(())
            }
            Map::Structural(vec) => key.with_value_container(|key| {
                if let Some((_, v)) = vec.iter_mut().find(|(k, _)| k == key) {
                    *v = value.into();
                    Ok(())
                } else {
                    Err(KeyNotFoundError { key: key.clone() })
                }
            }),
            Map::StructuralWithStringKeys(vec) => {
                if let Some(string) = key.try_as_text() {
                    if let Some((_, v)) =
                        vec.iter_mut().find(|(k, _)| k == string)
                    {
                        *v = value.into();
                        Ok(())
                    } else {
                        Err(KeyNotFoundError { key: key.into() })
                    }
                } else {
                    Err(KeyNotFoundError { key: key.into() })
                }
            }
        }
    }

    pub(crate) fn iter(&self) -> MapIterator<'_> {
        MapIterator {
            map: self,
            index: 0,
        }
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

impl StructuralEq for BorrowedMapKey<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BorrowedMapKey::Text(a), BorrowedMapKey::Text(b)) => a == b,
            (BorrowedMapKey::Value(a), BorrowedMapKey::Value(b)) => {
                a.structural_eq(b)
            }
            (BorrowedMapKey::Text(a), BorrowedMapKey::Value(b))
            | (BorrowedMapKey::Value(b), BorrowedMapKey::Text(a)) => {
                if let ValueContainer::Local(Value {
                    inner: CoreValue::Text(text),
                    ..
                }) = b
                {
                    a == &text.0
                } else {
                    false
                }
            }
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

#[derive(Debug)]
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

impl<'a> From<&'a MapKey> for BorrowedValueKey<'a> {
    fn from(key: &'a MapKey) -> Self {
        match key {
            MapKey::Text(text) => BorrowedValueKey::Text(Cow::Borrowed(text)),
            MapKey::Value(value) => BorrowedValueKey::Value(Cow::Borrowed(value)),
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
        match self.map {
            Map::Dynamic(map) => {
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
            Map::Structural(vec) => {
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
            Map::StructuralWithStringKeys(vec) => {
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
    Dynamic(indexmap::map::IterMut<'a, ValueContainer, ValueContainer>),
    Fixed(core::slice::IterMut<'a, (ValueContainer, ValueContainer)>),
    Structural(core::slice::IterMut<'a, (String, ValueContainer)>),
}

impl<'a> Iterator for MapMutIterator<'a> {
    type Item = (BorrowedMapKey<'a>, &'a mut ValueContainer);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MapMutIterator::Dynamic(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => BorrowedMapKey::Text(&text.0),
                    _ => BorrowedMapKey::Value(k),
                };
                (key, v)
            }),
            MapMutIterator::Fixed(iter) => iter.next().map(|(k, v)| {
                let key = match k {
                    ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    }) => BorrowedMapKey::Text(&text.0),
                    _ => BorrowedMapKey::Value(k),
                };
                (key, v)
            }),
            MapMutIterator::Structural(iter) => iter
                .next()
                .map(|(k, v)| (BorrowedMapKey::Text(k.as_str()), v)),
        }
    }
}

pub struct IntoMapIterator {
    map: Map,
    index: usize,
}

impl Iterator for IntoMapIterator {
    type Item = (MapKey, ValueContainer);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO #332: optimize to avoid cloning keys and values
        match &self.map {
            Map::Dynamic(map) => {
                let item = map.iter().nth(self.index);
                self.index += 1;
                item.map(|(k, v)| {
                    let key = match k {
                        ValueContainer::Local(Value {
                            inner: CoreValue::Text(text),
                            ..
                        }) => MapKey::Text(text.0.clone()),
                        _ => MapKey::Value(k.clone()),
                    };
                    (key, v.clone())
                })
            }
            Map::Structural(vec) => {
                if self.index < vec.len() {
                    let item = &vec[self.index];
                    self.index += 1;
                    let key = match &item.0 {
                        ValueContainer::Local(Value {
                            inner: CoreValue::Text(text),
                            ..
                        }) => MapKey::Text(text.0.clone()),
                        _ => MapKey::Value(item.0.clone()),
                    };
                    Some((key, item.1.clone()))
                } else {
                    None
                }
            }
            Map::StructuralWithStringKeys(vec) => {
                if self.index < vec.len() {
                    let item = &vec[self.index];
                    self.index += 1;
                    Some((MapKey::Text(item.0.clone()), item.1.clone()))
                } else {
                    None
                }
            }
        }
    }
}

impl StructuralEq for Map {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        for ((key, value), (other_key, other_value)) in
            self.iter().zip(other.iter())
        {
            if !key.structural_eq(&other_key)
                || !value.structural_eq(other_value)
            {
                return false;
            }
        }
        true
    }
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        IntoMapIterator {
            map: self,
            index: 0,
        }
    }
}

impl<'a> IntoIterator for &'a mut Map {
    type Item = (BorrowedMapKey<'a>, &'a mut ValueContainer);
    type IntoIter = MapMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Map::Dynamic(map) => MapMutIterator::Dynamic(map.iter_mut()),
            Map::Structural(vec) => MapMutIterator::Fixed(vec.iter_mut()),
            Map::StructuralWithStringKeys(vec) => {
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
            Map::StructuralWithStringKeys(entries)
        } else {
            let mut map = Map::default();
            for (k, v) in vec {
                map.set(&k, v);
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
        Map::Dynamic(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
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
        values::{
            core_values::{
                decimal::{Decimal, typed_decimal::TypedDecimal},
                map::Map,
            },
            value_container::ValueContainer,
        },
    };
    use crate::runtime::memory::Memory;
    use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
    use crate::shared_values::pointer_address::SelfOwnedPointerAddress;
    use crate::shared_values::shared_containers::SharedContainerMutability;
    use crate::shared_values::shared_containers::{SelfOwnedSharedContainer, OwnedSharedContainer, SharedContainer};
    use crate::shared_values::shared_containers::base_shared_value_container::BaseSharedValueContainer;

    #[test]
    fn test_map() {
        let mut map = Map::default();
        map.set("key1", 42);
        map.set("key2", "value2");
        assert_eq!(map.size(), 2);
        assert_eq!(map.get("key1").unwrap().to_string(), "42");
        assert_eq!(map.get("key2").unwrap().to_string(), "\"value2\"");
        assert_eq!(map.to_string(), r#"{"key1": 42, "key2": "value2"}"#);
    }

    #[test]
    fn test_duplicate_keys() {
        let mut map = Map::default();
        map.set("key1", 42);
        map.set("key1", "new_value");
        assert_eq!(map.size(), 1);
        assert_eq!(map.get("key1").unwrap().to_string(), "\"new_value\"");
    }

    #[test]
    fn test_ref_keys() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let memory = &Memory::new();

        let mut map = Map::default();
        let key = ValueContainer::Shared(
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                address_provider,
                memory
            )
        );

        map.set(key.clone(), "value");
        // same reference should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&key));
        assert_eq!(map.get(&key).unwrap().to_string(), "\"value\"");

        // new reference with same value should not be found
        let new_key = ValueContainer::Shared(
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                address_provider,
                memory
            )
        );
        assert!(!map.has(&new_key));
        assert!(map.get(&new_key).is_err());
    }

    #[test]
    fn test_decimal_nan_value_key() {
        let mut map = Map::default();
        let nan_value = ValueContainer::from(Decimal::Nan);
        map.set(&nan_value, "value");
        // same NaN value should be found
        assert_eq!(map.size(), 1);
        assert!(map.has(&nan_value));

        // new NaN value should also be found
        let new_nan_value = ValueContainer::from(Decimal::Nan);
        assert!(map.has(&new_nan_value));

        // adding new_nan_value should not increase size
        map.set(&new_nan_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn test_float_nan_value_key() {
        let mut map = Map::default();
        let nan_value = ValueContainer::from(f64::NAN);
        map.set(&nan_value, "value");
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
        map.set(&new_nan_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn test_decimal_zero_value_key() {
        let mut map = Map::default();
        let zero_value = ValueContainer::from(Decimal::Zero);
        map.set(&zero_value, "value");
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
        map.set(&neg_zero_value, "new_value");
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn test_float_zero_value_key() {
        let mut map = Map::default();
        let zero_value = ValueContainer::from(0.0f64);
        map.set(&zero_value, "value");
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
        map.set(&neg_zero_value, "new_value");
        assert_eq!(map.size(), 1);

        // new 0.0f32 value should not be found
        let float32_zero_value = ValueContainer::from(0.0f32);
        assert!(!map.has(&float32_zero_value));
    }

    #[test]
    fn test_typed_big_decimal_key() {
        let mut map = Map::default();
        let zero_big_decimal =
            ValueContainer::from(TypedDecimal::Decimal(Decimal::Zero));
        map.set(&zero_big_decimal, "value");
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
        map.set(&neg_zero_big_decimal, "new_value");
        assert_eq!(map.size(), 1);
    }
}
