use serde::Serializer;
pub trait SerializeSeedOwned {
    type Value;

    fn serialize_owned<S>(
        &mut self,
        value: Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> SerializeSeedOwned for &mut T
where
    T: SerializeSeedOwned + ?Sized,
{
    type Value = T::Value;

    fn serialize_owned<S>(
        &mut self,
        value: Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).serialize_owned(value, serializer)
    }
}

use core::{
    cell::{Cell, UnsafeCell},
    mem::ManuallyDrop,
};
use serde::{Serialize, ser::Error};

pub struct ValueWithOwnedSeed<Value, Seed> {
    value: UnsafeCell<ManuallyDrop<Value>>,
    seed: UnsafeCell<Seed>,
    used: Cell<bool>,
}

impl<Value, Seed> ValueWithOwnedSeed<Value, Seed> {
    pub fn new(value: Value, seed: Seed) -> Self {
        Self {
            value: UnsafeCell::new(ManuallyDrop::new(value)),
            seed: UnsafeCell::new(seed),
            used: Cell::new(false),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tuple4Serde<T1, T2, T3, T4>(pub T1, pub T2, pub T3, pub T4);

impl<T1, T2, T3, T4> SerializeSeedOwned for Tuple4Serde<T1, T2, T3, T4>
where
    T1: SerializeSeedOwned,
    T2: SerializeSeedOwned,
    T3: SerializeSeedOwned,
    T4: SerializeSeedOwned,
{
    type Value = (T1::Value, T2::Value, T3::Value, T4::Value);

    fn serialize_owned<S>(
        &mut self,
        value: Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (v1, v2, v3, v4) = value;

        let mut tuple = serializer.serialize_tuple(4)?;

        tuple.serialize_element(&ValueWithOwnedSeed::new(v1, &mut self.0))?;
        tuple.serialize_element(&ValueWithOwnedSeed::new(v2, &mut self.1))?;
        tuple.serialize_element(&ValueWithOwnedSeed::new(v3, &mut self.2))?;
        tuple.serialize_element(&ValueWithOwnedSeed::new(v4, &mut self.3))?;

        tuple.end()
    }
}

impl<Value, Seed> Serialize for ValueWithOwnedSeed<Value, Seed>
where
    Seed: SerializeSeedOwned<Value = Value>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.used.replace(true) {
            return Err(S::Error::custom(
                "ValueWithOwnedSeed cannot be serialized more than once",
            ));
        }

        unsafe {
            let value = ManuallyDrop::take(&mut *self.value.get());
            let seed = &mut *self.seed.get();

            seed.serialize_owned(value, serializer)
        }
    }
}

impl<Value, Seed> Drop for ValueWithOwnedSeed<Value, Seed> {
    fn drop(&mut self) {
        if !self.used.get() {
            unsafe {
                ManuallyDrop::drop(&mut *self.value.get());
            }
        }
    }
}

use serde::ser::SerializeTuple;

#[derive(Debug, Clone, Copy)]
pub struct PairSerde<U, V>(pub U, pub V);

impl<U, V> SerializeSeedOwned for PairSerde<U, V>
where
    U: SerializeSeedOwned,
    V: SerializeSeedOwned,
{
    type Value = (U::Value, V::Value);

    fn serialize_owned<S>(
        &mut self,
        value: Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (u_value, v_value) = value;

        let mut tuple = serializer.serialize_tuple(2)?;

        tuple.serialize_element(&ValueWithOwnedSeed::new(
            u_value,
            &mut self.0,
        ))?;

        tuple.serialize_element(&ValueWithOwnedSeed::new(
            v_value,
            &mut self.1,
        ))?;

        tuple.end()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VecSerde<T>(pub T);

impl<T> SerializeSeedOwned for VecSerde<T>
where
    T: SerializeSeedOwned,
{
    type Value = alloc::vec::Vec<T::Value>;

    fn serialize_owned<S>(
        &mut self,
        value: Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(value.len()))?;

        for item in value {
            seq.serialize_element(&ValueWithOwnedSeed::new(item, &mut self.0))?;
        }

        seq.end()
    }
}
