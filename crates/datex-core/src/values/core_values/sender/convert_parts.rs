use crate::{
    traits::{
        convert_parts::{FromParts, HasPartsKind, IntoParts, PartsKind},
        convert_value_container::ConvertValueContainer,
    },
    values::core_values::sender::Sender,
};

impl<T> HasPartsKind for Sender<T> {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::None
    }
}
impl<T: ConvertValueContainer> IntoParts for Sender<T> {}

impl<T: ConvertValueContainer + FromParts> FromParts for Sender<T> {}
