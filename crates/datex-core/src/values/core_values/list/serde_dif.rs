use serde::{Serializer, ser::SerializeSeq};
use serde::ser::SerializeMap;
use crate::dif::serde_context::SerdeContext;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};
use crate::values::core_values::list::List;
use crate::values::core_values::map::{BorrowedMapKey};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, List> {
    type Value = List;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(value.size()))?;
        for value in value.iter() {
            state.serialize_element(
                &ValueWithSeed::new(&value, self.cast::<BorrowedMapKey>()),
            )?;
        }
        state.end()
    }
}

