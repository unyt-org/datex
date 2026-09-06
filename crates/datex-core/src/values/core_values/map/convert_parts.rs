use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::core_values::map::Map,
};

impl IntoParts for Map {
    fn into_parts<'a>(
        self,
        _cache: &'a mut SharedReferencesCache,
    ) -> Option<Parts<'a>>
    where
        Self: 'a,
    {
        Some(Parts::Map(Box::new(
            self.into_iter().map(|(key, value)| (key.into(), value)),
        )))
    }

    fn as_parts<'a>(
        &'a self,
        _cache: &'a mut SharedReferencesCache,
    ) -> Option<BorrowedParts<'a>> {
        Some(BorrowedParts::Map(Box::new(
            self.iter().map(|(key, value)| (key.into(), value.into())),
        )))
    }
}

impl FromParts for Map {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match parts {
            Parts::Map(iter) => {
                let mut map = Map::default();
                for (key, value) in iter {
                    map.set_unchecked(key, value);
                }
                Ok(map)
            }
            _ => Err(()),
        }
    }
}
