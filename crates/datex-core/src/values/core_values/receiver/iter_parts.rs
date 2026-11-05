use crate::{
    traits::{
        convert_value_container::ConvertValueContainer, iter_parts::IterParts,
    },
    values::core_values::receiver::Receiver,
};

impl<T: ConvertValueContainer> IterParts for Receiver<T> {}
