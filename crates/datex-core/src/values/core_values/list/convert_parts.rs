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
    fn try_into_list_parts<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        Ok(*self)
    }
}

impl FromParts for List {
    fn try_from_list_parts(parts: List) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(parts)
    }
}
