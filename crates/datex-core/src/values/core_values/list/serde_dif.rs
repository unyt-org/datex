use crate::{
    dif::{
        deserialize_serde_context::DeserializeSerdeContext,
        deserialize_with_serde_context::DeserializeWithSerdeContext,
        dif_vec::DIFVec, serde_context::SerdeContext,
        serialize_with_serde_context::SerializeWithSerdeContext,
        value_with_serde_context::ValueWithSerdeContext,
    },
    libs::core::{core_lib_id::CoreLibIdIndex, type_id::CoreLibBaseTypeId},
    values::{core_values::list::List, value_container::ValueContainer},
};
use erased_serde::__private::serde::ser::SerializeTuple;
use num_traits::ToPrimitive;
use serde::{
    Serializer,
    de::Visitor,
    ser::{SerializeMap, SerializeSeq},
};

impl SerializeWithSerdeContext for List {
    fn serialize_with_ctx<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // [CoreLibBaseTypeId::List, list]
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(
            &CoreLibIdIndex::from(CoreLibBaseTypeId::List).to_u16(),
        )?;
        tuple.serialize_element(&ValueWithSerdeContext::new(
            &DIFVec(&self.items),
            ctx,
        ))?;
        tuple.end()
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for List {
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<List, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::<List>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, List> {
    type Value = List;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a sequence of values for List")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<List, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut list = List::default();

        while let Some(value) =
            seq.next_element_seed(self.cast::<ValueContainer>())?
        {
            list.push(value);
        }

        Ok(list)
    }
}
