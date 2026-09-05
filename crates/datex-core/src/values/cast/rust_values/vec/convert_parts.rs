use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::prelude::*;

impl<T: ConvertValueContainer> IntoParts for Vec<T> {
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>> where Self: 'a, {
        Some(Parts::List(Box::new(self.into_iter().map(move |item| item.to_value_container(cache)))))
    }

    fn as_parts<'a>(&'a self, cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        Some(BorrowedParts::List(Box::new(self.iter().map(|item| item.as_borrowed_value_container(cache)))))
    }
}

impl<T: ConvertValueContainer> FromParts for Vec<T> {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized
    {
        match parts {
            Parts::List(list) => {
                let mut vec = Vec::new();
                for item in list {
                    vec.push(T::try_from_value_container(item).map_err(|_| ())?);
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}