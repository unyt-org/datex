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
    values::core_values::sender::Sender,
};
use erased_serde::__private::serde::ser::SerializeTuple;
use num_traits::ToPrimitive;
use serde::de::{SeqAccess, Visitor};

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext for Sender<T> {
    fn serialize_with_ctx<S: serde::Serializer>(
        &self,
        _ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // [CoreLibBaseTypeId::Sender, null]
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(
            &CoreLibIdIndex::from(CoreLibBaseTypeId::Sender).to_u16(),
        )?;
        tuple.serialize_element(&())?;
        tuple.end()
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de>
    for Sender<T>
{
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Sender<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(DeserializeSerdeContext::<Sender<T>>::new(ctx))
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> Visitor<'de>
    for DeserializeSerdeContext<'_, '_, Sender<T>>
{
    type Value = Sender<T>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("a sequence of values for Sender<T>")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        _seq: A,
    ) -> Result<Sender<T>, A::Error> {
        todo!()
    }
}
