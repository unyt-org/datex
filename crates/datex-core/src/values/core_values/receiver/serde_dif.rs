use crate::{
    dif::{
        deserialize_serde_context::DeserializeSerdeContext,
        deserialize_with_serde_context::{
            DeserializeWithSerdeContext, DeserializeWithSerdeContextDyn,
        },
        serde_context::SerdeContext,
        serialize_with_serde_context::{
            SerializeWithSerdeContext, SerializeWithSerdeContextDyn,
        },
    },
    libs::core::{core_lib_id::CoreLibIdIndex, type_id::CoreLibBaseTypeId},
    values::core_values::receiver::Receiver,
};
use erased_serde::__private::serde::ser::SerializeTuple;
use num_traits::ToPrimitive;
use serde::de::{SeqAccess, Visitor};

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext
    for Receiver<T>
{
    fn serialize_with_ctx<S: serde::Serializer>(
        &self,
        _ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // [CoreLibBaseTypeId::Receiver, null]
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(
            &CoreLibIdIndex::from(CoreLibBaseTypeId::Receiver).to_u16(),
        )?;
        tuple.serialize_element(&())?;
        tuple.end()
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de>
    for Receiver<T>
{
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Receiver<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(DeserializeSerdeContext::<Receiver<T>>::new(ctx))
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> Visitor<'de>
    for DeserializeSerdeContext<'_, '_, Receiver<T>>
{
    type Value = Receiver<T>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("a sequence of values for Receiver<T>")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        _seq: A,
    ) -> Result<Receiver<T>, A::Error> {
        todo!()
    }
}
