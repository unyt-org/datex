use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::core_values::list::List,
};
use crate::traits::convert_parts::PartsKind;

impl IntoParts for List {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::List
    }
    fn try_into_parts<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<Parts<'a>, ()>
    where
        Self: 'a,
    {
        Ok(Parts::List(Box::new(self.into_iter())))
    }

    fn try_as_parts<'a>(
        &'a self,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<BorrowedParts<'a>, ()> {
        Ok(BorrowedParts::List(Box::new(
            self.iter().map(|item| item.into()),
        )))
    }
}

impl FromParts for List {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
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
