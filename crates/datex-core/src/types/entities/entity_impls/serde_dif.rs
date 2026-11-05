use crate::{
    dif::{
        deserialize_serde_context::DeserializeSerdeContext,
        deserialize_with_serde_context::DeserializeWithSerdeContext,
        dif_vec::{DIFVec, OwnedDIFVec},
        serde_context::SerdeContext,
        serialize_with_serde_context::SerializeWithSerdeContext,
        value_with_serde_context::ValueWithSerdeContext,
    },
    prelude::*,
    types::entities::entity_impls::{EntityImpl, EntityImplMethod},
    values::core_values::callable::Callable,
};
use serde::{Serializer, de::Visitor, ser::SerializeSeq};

impl SerializeWithSerdeContext for EntityImpl {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSerdeContext::new(
            &DIFVec(
                &self
                    .methods
                    .iter()
                    .map(|method| &method.callable)
                    .collect::<Vec<_>>(),
            ),
            ctx,
        ))?;
        seq.serialize_element(&ValueWithSerdeContext::new(
            &DIFVec(&self.static_methods),
            ctx,
        ))?;
        seq.end()
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for EntityImpl {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer
            .deserialize_any(DeserializeSerdeContext::<EntityImpl>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de>
    for DeserializeSerdeContext<'a, 'ctx, EntityImpl>
{
    type Value = EntityImpl;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("an EntityImpl")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let methods: OwnedDIFVec<Callable> = seq
            .next_element_seed(self.cast())?
            .ok_or_else(|| serde::de::Error::custom("Expected methods"))?;
        let static_methods: OwnedDIFVec<Callable> =
            seq.next_element_seed(self.cast())?.ok_or_else(|| {
                serde::de::Error::custom("Expected static methods")
            })?;

        Ok(EntityImpl {
            methods: methods
                .0
                .into_iter()
                .map(|callable| EntityImplMethod::new(true, callable))
                .collect(),
            static_methods: static_methods.0,
        })
    }
}
