use crate::{
    prelude::*,
    values::{
        core_value::CoreValue, core_values::list::List,
        value_container::ValueContainer,
    },
};
use serde::{Serialize, Serializer};

/// Serialization for [CoreValue].
impl Serialize for CoreValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CoreValue::Integer(i) => i.serialize(serializer),
            CoreValue::Boolean(b) => b.serialize(serializer),
            CoreValue::Text(s) => s.serialize(serializer),
            CoreValue::Decimal(d) => d.serialize(serializer),
            CoreValue::Endpoint(e) => e.serialize(serializer),
            CoreValue::List(l) => l.serialize(serializer),
            CoreValue::TypedInteger(ti) => ti.serialize(serializer),
            CoreValue::TypedDecimal(td) => td.serialize(serializer),
            CoreValue::Map(map) => map.serialize(serializer),
            CoreValue::Range(r) => r.serialize(serializer),
            // CoreValue::Type(t) => t.serialize(serializer),
            // CoreValue::Callable(c) => c.serialize(serializer),
            // CoreValue::NominalTypeDefinition(ntd) => ntd.serialize(serializer),
            CoreValue::Null => serializer.serialize_unit(),
            _ => unimplemented!(
                "Serialization for this CoreValue variant is not implemented yet."
            ),
        }
    }
}

use crate::dif::serde_context::SerdeContext;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
};
/// Deserialization for [CoreValue] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<CoreValue, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a CoreValue")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<CoreValue, A::Error> {
        let mut items = Vec::new();
        while let Some(item) =
            seq.next_element_seed(self.cast::<ValueContainer>())?
        {
            items.push(item);
        }
        Ok(CoreValue::List(List::from(items)))
    }
}
