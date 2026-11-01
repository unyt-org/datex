use erased_serde::__private::serde::ser::SerializeTuple;
use num_traits::ToPrimitive;
use crate::{
    dif::{
        deserialize_serde_context::{DeserializeSerdeContext, ErasedSeed},
        deserialize_with_serde_context::{
            DeserializeWithSerdeContext, DeserializeWithSerdeContextDyn,
        },
        serde_context::SerdeContext,
        serialize_with_serde_context::{
            SerializeWithSerdeContext, SerializeWithSerdeContextDyn,
        },
    },
    prelude::*,
};
use serde::{
    de::{SeqAccess, Visitor},
};
use crate::dif::dif_vec::DIFVec;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::libs::core::core_lib_id::CoreLibIdIndex;
use crate::libs::core::type_id::CoreLibBaseTypeId;

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext for Vec<T> {
    fn serialize_with_ctx<S: serde::Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // [CoreLibBaseTypeId::List, list]
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&CoreLibIdIndex::from(CoreLibBaseTypeId::List).to_u16())?;
        tuple.serialize_element(&ValueWithSerdeContext::new(&DIFVec(self), ctx))?;
        tuple.end()
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de>
    for Vec<T>
{
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(DeserializeSerdeContext::<Vec<T>>::new(ctx))
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> Visitor<'de>
    for DeserializeSerdeContext<'_, '_, Vec<T>>
{
    type Value = Vec<T>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("a sequence of values for Vec<T>")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> Result<Vec<T>, A::Error> {
        let seed = ErasedSeed::<T>::new(self.ctx);
        let mut list =
            Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
        while let Some(value) = seq.next_element_seed(seed)? {
            list.push(value);
        }
        Ok(list)
    }
}
