use serde::de::DeserializeSeed;
use serde::{Deserializer, Serialize, Serializer};
use crate::dif::serde_context::SerdeContext;
use crate::shared_values::{SharedContainer};
use crate::types::entity_type::EntityType;
use crate::utils::serde_serialize_seed::SerializeSeed;

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, EntityType> {
    type Value = EntityType;
    fn deserialize<D: Deserializer<'de>>(
        mut self,
        d: D,
    ) -> Result<EntityType, D::Error> {
        Ok(
            unsafe {
                EntityType::new_unchecked(self.cast::<SharedContainer>().deserialize(d)?)
            }
        )
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
        self.cast::<SharedContainer>().serialize(&value.0, serializer)
    }
}