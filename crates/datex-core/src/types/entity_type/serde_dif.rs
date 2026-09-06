use crate::{
    dif::serde_context::SerdeContext, shared_values::SharedContainer,
    types::entity_type::EntityType, utils::serde_serialize_seed::SerializeSeed,
};
use serde::{Deserializer, Serialize, Serializer, de::DeserializeSeed};

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, EntityType> {
    type Value = EntityType;
    fn deserialize<D: Deserializer<'de>>(
        mut self,
        d: D,
    ) -> Result<EntityType, D::Error> {
        Ok(unsafe {
            EntityType::new_unchecked(
                self.cast::<SharedContainer>().deserialize(d)?,
            )
        })
    }
}
impl<'ctx> SerializeSeed for SerdeContext<'ctx, EntityType> {
    type Value = EntityType;

    /// SAFETY:
    /// The caller of the `serialize` method must either
    /// * guarantee that no direct value (accessible without borrow) is an owned shared value
    ///   (this can be guaranteed by calling clone on the top level value before passing it to [SerializeSeed])
    /// * or guarantee that the value is dropped after calling `serialize`, so that the owned shared value
    ///   is not leaked after serialization.
    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.cast::<SharedContainer>()
            .serialize(&value.0, serializer)
    }
}
