use crate::{
    random::RandomState,
    traits::{
        child_iterator::ChildIterator, convert_value::ConvertValue,
        convert_value_container::ConvertValueContainer,
    },
    values::{
        borrowed_value_container::BorrowedValueContainer,
        core_values::native::DatexNativeBase,
    },
};
use indexmap::IndexMap;

impl<K: ConvertValueContainer, V: ConvertValueContainer> ChildIterator
    for IndexMap<K, V, RandomState>
{
    fn iter_children<'a>(
		&'a self,
	) -> Option<Box<dyn Iterator<Item = crate::values::borrowed_value_container::BorrowedValueContainer<'a>> + 'a>>
{
        Some(Box::new(self.iter().flat_map(|(k, v)| {
            vec![
                k.as_borrowed_value_container(),
                v.as_borrowed_value_container(),
            ]
            .into_iter()
        })))
    }
}
