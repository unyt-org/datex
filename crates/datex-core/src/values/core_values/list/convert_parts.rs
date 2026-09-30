use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{
        BorrowedParts, FromParts, IntoParts, Parts, PartsKind, WithPartsKind,
    },
    values::core_values::list::List,
};

impl WithPartsKind for List {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::List
    }
}

impl IntoParts for List {
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
    fn try_from_list_parts_with_tag(
        parts: List,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(parts)
    }
}
