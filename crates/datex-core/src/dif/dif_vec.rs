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
    ser::SerializeSeq,
};

pub struct DIFVec<'a, T>(pub &'a [T]);

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext
    for DIFVec<'_, T>
{
    fn serialize_with_ctx<S: serde::Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            seq.serialize_element(&*value.with_ctx(ctx))?;
        }
        seq.end()
    }
}

pub struct OwnedDIFVec<T>(pub Vec<T>);

impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de>
    for OwnedDIFVec<T>
{
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<OwnedDIFVec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(
            DeserializeSerdeContext::<OwnedDIFVec<T>>::new(ctx),
        )
    }
}

impl<'de, T: DeserializeWithSerdeContextDyn> Visitor<'de>
    for DeserializeSerdeContext<'_, '_, OwnedDIFVec<T>>
{
    type Value = OwnedDIFVec<T>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("a sequence of values for OwnedDIFVec<T>")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> Result<OwnedDIFVec<T>, A::Error> {
        let seed = ErasedSeed::<T>::new(self.ctx);
        let mut list =
            Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
        while let Some(value) = seq.next_element_seed(seed)? {
            list.push(value);
        }
        Ok(OwnedDIFVec(list))
    }
}
