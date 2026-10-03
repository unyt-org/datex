use crate::{
    dif::serde_context::SerdeContext, shared_values::SharedContainer,
    types::entity_type::EntityType, utils::serde_serialize_seed::SerializeSeed,
};
use serde::{Deserializer, Serialize, Serializer, de::DeserializeSeed};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for EntityType {
    fn deserialize_with_ctx<D: Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        Ok(unsafe {
            EntityType::new_unchecked(
                DeserializeSerdeContext::new(ctx).cast::<SharedContainer>().deserialize(deserializer)?,
            )
        })
    }
}
impl<'ctx> SerializeSeed for EntityType {
    /// SAFETY:
    /// The caller of the `serialize` method must either
    /// * guarantee that no direct value (accessible without borrow) is an owned shared value
    ///   (this can be guaranteed by calling clone on the top level value before passing it to [SerializeSeed])
    /// * or guarantee that the value is dropped after calling `serialize`, so that the owned shared value
    ///   is not leaked after serialization.
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .serialize_seed(ctx, serializer)
    }
}
