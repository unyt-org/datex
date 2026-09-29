use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::{
        convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
        convert_value_container::ConvertValueContainer,
    },
    values::{core_values::list::List},
};
use crate::preludes::derive::PartsKind;

impl<T: ConvertValueContainer> IntoParts for Vec<T> {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::List
    }
    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        let mut list = List::default();
        for item in *self {
            list.push(item.to_value_container(cache));
        }
        Ok(list)
    }
}

impl<T: ConvertValueContainer> FromParts for Vec<T> {
    fn try_from_list_parts(parts: List) -> Result<Self, ()>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();
        for item in parts {
            vec.push(T::try_from_value_container(item).map_err(|_| ())?);
        }
        Ok(vec)
    }
}
