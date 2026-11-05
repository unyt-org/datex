use crate::{
    traits::{
        convert_value_container::ConvertValueContainer, iter_parts::IterParts,
    },
    values::core_values::sender::Sender,
};

impl<T: ConvertValueContainer> IterParts for Sender<T> {}
