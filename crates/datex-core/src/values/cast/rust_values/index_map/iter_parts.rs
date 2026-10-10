use crate::{
    random::RandomState,
    traits::{
        iter_parts::IterParts, convert_value::ConvertValue,
        convert_value_container::ConvertValueContainer,
    },
    values::{
    },
};
use indexmap::IndexMap;
use crate::values::value_container::borrowed_value_container::BorrowedValueContainer;

impl<K: ConvertValueContainer, V: ConvertValueContainer> IterParts
    for IndexMap<K, V, RandomState>
{
    fn iter_list_parts<'a>(
		&'a self,
	) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>>
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
