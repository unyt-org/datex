use crate::{
    prelude::*,
    traits::{
        convert_parts::{FromParts, HasPartsKind, IntoParts, PartsKind},
        convert_value_container::ConvertValueContainer,
    },
};
use crate::values::core_values::receiver::Receiver;

impl<T> HasPartsKind for Receiver<T> {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::None
    }
}
impl<T: ConvertValueContainer> IntoParts for Receiver<T> {
}

impl<T: ConvertValueContainer + FromParts> FromParts for Receiver<T> {
}
