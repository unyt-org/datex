use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::values::core_values::list::List;
use crate::prelude::*;

impl IntoParts for List {
    fn into_parts<'a>(self, _cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>> where Self: 'a, {
        Some(Parts::List(Box::new(self.into_iter().map(move |item| item))))
    }

    fn as_parts<'a>(&'a self, _cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        Some(BorrowedParts::List(Box::new(self.iter().map(|item| item.into()))))
    }
}

impl FromParts for List {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized
    {
        match parts {
            Parts::List(iter) => {
                let mut list = List::default();
                for item in iter {
                    list.push(item);
                }
                Ok(list)
            }
            _ => Err(()),
        }
    }
}