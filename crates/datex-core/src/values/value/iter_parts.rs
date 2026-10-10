use crate::{
    traits::iter_parts::IterParts,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        value::Value,
    },
};

impl IterParts for Value {
    fn iter_map_parts<'a>(
        &'a self,
    ) -> Option<
        Box<
            dyn Iterator<
                    Item = (
                        BorrowedValueContainer<'a>,
                        BorrowedValueContainer<'a>,
                    ),
                > + 'a,
        >,
    > {
        match self {
            Value::Core(core_value) => core_value.iter_map_parts(),
            Value::Native(native_value) => native_value.iter_map_parts(),
        }
    }
    fn iter_map_parts_mut<'a>(
        &'a mut self,
    ) -> Option<
        Box<
            dyn Iterator<
                    Item = (
                        BorrowedValueContainer<'a>,
                        BorrowedValueContainerMut<'a>,
                    ),
                > + 'a,
        >,
    > {
        match self {
            Value::Core(core_value) => core_value.iter_map_parts_mut(),
            Value::Native(native_value) => native_value.iter_map_parts_mut(),
        }
    }
    fn iter_list_parts<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        match self {
            Value::Core(core_value) => core_value.iter_list_parts(),
            Value::Native(native_value) => native_value.iter_list_parts(),
        }
    }

    fn iter_list_parts_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        match self {
            Value::Core(core_value) => core_value.iter_list_parts_mut(),
            Value::Native(native_value) => native_value.iter_list_parts_mut(),
        }
    }
}
