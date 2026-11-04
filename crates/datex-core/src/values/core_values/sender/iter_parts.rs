use crate::{
    prelude::*,
    traits::{
        convert_value_container::ConvertValueContainer, iter_parts::IterParts,
    },
};
use crate::values::core_values::sender::Sender;

impl<T: ConvertValueContainer> IterParts for Sender<T> {
}
