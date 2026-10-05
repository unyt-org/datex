use serde::de::Visitor;
use serde::ser::SerializeSeq;
use serde::Serializer;
use crate::dif::deserialize_serde_context::DeserializeSerdeContext;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContext;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::values::core_values::native::DatexNativeBase;

impl<'ctx, T: DatexNativeBase> SerializeWithSerdeContext for Vec<T> {

    fn serialize_with_ctx<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.len()))?;
        for value in self.iter() {
            state.serialize_element(&ValueWithSerdeContext::new(
                value,
                ctx,
            ))?;
        }
        state.end()
    }
}

impl<'de, 'ctx, T: DatexNativeBase> DeserializeWithSerdeContext<'de> for Vec<T> {
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::<Vec<T>>::new(ctx))
    }
}

impl<'de, 'a, 'ctx, T: DatexNativeBase> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, Vec<T>> {
    type Value = Vec<T>;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a sequence of values for Vec<T>")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut list = Vec::new();

        while let Some(value) =
            seq.next_element_seed(self.cast::<T>())?
        {
            list.push(value);
        }

        Ok(list)
    }
}
