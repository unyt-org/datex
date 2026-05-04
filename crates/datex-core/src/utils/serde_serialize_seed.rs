/// Implementation of https://docs.rs/serde-serialize-seed/latest/serde_serialize_seed/, but with &mut self
use core::fmt;
use core::fmt::Formatter;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use serde::{de, Deserializer, Serialize, Serializer};
use serde::de::{DeserializeSeed, Error, SeqAccess};
use serde::ser::{SerializeSeq, SerializeTuple};
use crate::serde::Deserialize;

pub trait SerializeSeed {
    type Value: ?Sized;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<T: SerializeSeed + ?Sized> SerializeSeed for &mut T {
    type Value = T::Value;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        (**self).serialize(value, serializer)
    }
}

#[derive(Debug)]
pub struct ValueWithSeed<'a, Value: ?Sized, Seed>(pub &'a Value, pub UnsafeCell<Seed>);

impl<'a, Value: ?Sized, Seed: SerializeSeed<Value=Value>> ValueWithSeed<'a, Value, Seed> {
    pub fn new(value: &'a Value, seed: Seed) -> Self {
        Self(value, UnsafeCell::new(seed))
    }
}

impl<'a, Value: ?Sized, Seed: SerializeSeed<Value=Value>> Serialize for ValueWithSeed<'a, Value, Seed> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // SAFETY: serde calls serialize exactly once, so no aliased &mut exists
        let seed = unsafe { &mut *self.1.get() };
        seed.serialize(self.0, serializer)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatelessSerde<T: ?Sized>(pub PhantomData<T>);

impl<T: Serialize + ?Sized> SerializeSeed for StatelessSerde<T> {
    type Value = T;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for StatelessSerde<T> {
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        T::deserialize(deserializer)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PairSerde<U, V>(pub U, pub V);

impl<U: SerializeSeed, V: SerializeSeed> SerializeSeed for PairSerde<U, V>
where U::Value: Sized, V::Value: Sized {
    type Value = (U::Value, V::Value);

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_tuple(2)?;
        s.serialize_element(&ValueWithSeed::new(&value.0, &mut self.0))?;
        s.serialize_element(&ValueWithSeed::new(&value.1, &mut self.1))?;
        s.end()
    }
}

struct PairDeVisitor<U, V>(PairSerde<U, V>);

impl<'de, U: DeserializeSeed<'de>, V: DeserializeSeed<'de>> de::Visitor<'de> for PairDeVisitor<U, V> where
    U::Value: Sized, V::Value: Sized
{
    type Value = (U::Value, V::Value);

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "pair")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let u = seq.next_element_seed(self.0.0)?
            .ok_or_else(|| A::Error::invalid_length(0, &"pair"))?;
        let v = seq.next_element_seed(self.0.1)?
            .ok_or_else(|| A::Error::invalid_length(1, &"pair"))?;
        Ok((u, v))
    }
}

impl<'de, U: DeserializeSeed<'de>, V: DeserializeSeed<'de>> DeserializeSeed<'de> for PairSerde<U, V>
where
    U::Value: Sized, V::Value: Sized
{
    type Value = (U::Value, V::Value);

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(2, PairDeVisitor(self))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tuple4Serde<T1, T2, T3, T4>(pub T1, pub T2, pub T3, pub T4);

impl<
    T1: SerializeSeed,
    T2: SerializeSeed,
    T3: SerializeSeed,
    T4: SerializeSeed
> SerializeSeed for Tuple4Serde<T1, T2, T3, T4>
where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_tuple(4)?;
        s.serialize_element(&ValueWithSeed::new(&value.0, &mut self.0))?;
        s.serialize_element(&ValueWithSeed::new(&value.1, &mut self.1))?;
        s.serialize_element(&ValueWithSeed::new(&value.2, &mut self.2))?;
        s.serialize_element(&ValueWithSeed::new(&value.3, &mut self.3))?;
        s.end()
    }
}

struct Tuple4DeVisitor<T1, T2, T3, T4>(Tuple4Serde<T1, T2, T3, T4>);

impl<
    'de,
    T1: DeserializeSeed<'de>,
    T2: DeserializeSeed<'de>,
    T3: DeserializeSeed<'de>,
    T4: DeserializeSeed<'de>
> de::Visitor<'de> for Tuple4DeVisitor<T1, T2, T3, T4> where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "tuple 4")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let t1 = seq.next_element_seed(self.0.0)?
            .ok_or_else(|| A::Error::invalid_length(0, &"tuple 4"))?;
        let t2 = seq.next_element_seed(self.0.1)?
            .ok_or_else(|| A::Error::invalid_length(1, &"tuple 4"))?;
        let t3 = seq.next_element_seed(self.0.2)?
            .ok_or_else(|| A::Error::invalid_length(2, &"tuple 4"))?;
        let t4 = seq.next_element_seed(self.0.3)?
            .ok_or_else(|| A::Error::invalid_length(3, &"tuple 4"))?;
        Ok((t1, t2, t3, t4))
    }
}

impl<
    'de,
    T1: DeserializeSeed<'de>,
    T2: DeserializeSeed<'de>,
    T3: DeserializeSeed<'de>,
    T4: DeserializeSeed<'de>
> DeserializeSeed<'de> for Tuple4Serde<T1, T2, T3, T4>
where
    T1::Value: Sized, T2::Value: Sized, T3::Value: Sized, T4::Value: Sized
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(4, Tuple4DeVisitor(self))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VecSerde<T>(pub T);

impl<T: SerializeSeed + Clone> SerializeSeed for VecSerde<T>
where T::Value: Sized {
    type Value = [T::Value];

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            serializer.serialize_element(&ValueWithSeed::new(item, &mut self.0))?;
        }
        serializer.end()
    }
}

struct VecDeVisitor<T>(VecSerde<T>);

impl<'de, T: DeserializeSeed<'de> + Clone> de::Visitor<'de> for VecDeVisitor<T> where T::Value: Sized {
    type Value = Vec<T::Value>;

    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "vector")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut vec = seq.size_hint().map_or_else(Vec::new, Vec::with_capacity);
        while let Some(f) = seq.next_element_seed(self.0.0.clone())? {
            vec.push(f);
        }
        Ok(vec)
    }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for VecSerde<T>
where T::Value: Sized {
    type Value = Vec<T::Value>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(VecDeVisitor(self))
    }
}
