use erased_serde::__private::serde::ser::SerializeSeq;
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::{SerializeWithSerdeContext, SerializeWithSerdeContextDyn};

pub struct DIFVec<'a, T>(pub &'a [T]);

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext for DIFVec<'_, T> {
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