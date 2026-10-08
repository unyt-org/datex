use erased_serde::__private::serde::Serializer;
use serde::ser::SerializeMap;
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContext;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::values::core_value_with_classification::CoreValueWithClassification;

impl SerializeWithSerdeContext for CoreValueWithClassification {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        let classification = &self.classification;
        // if default classification, just serialize the core value + tag
        if classification.is_unclassified() {
            (&self.inner, &classification.tag)
                .serialize_with_ctx(ctx, serializer)
        }
        // else serialize as a map with "c" and "v" keys
        else {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("c", &ValueWithSerdeContext::new(&classification, ctx))?;
            map.serialize_entry("v", &ValueWithSerdeContext::new(&(&self.inner, &classification.tag), ctx))?;
            map.end()
        }
    }
}