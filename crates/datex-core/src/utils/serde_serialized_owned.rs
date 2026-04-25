use serde::Serializer;
use serde_serialize_seed::SerializeSeed;

pub trait SerializeSeedOwned: SerializeSeed {
    fn serialize_owned<S>(
        &self,
        value: <Self as SerializeSeed>::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        <Self as SerializeSeed>::Value: Sized,
    {
        self.serialize(&value, serializer)
    }
}
