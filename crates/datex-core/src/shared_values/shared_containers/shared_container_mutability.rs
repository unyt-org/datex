use core::fmt::Display;
use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use crate::serde::Deserialize;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TryFromPrimitive, BinRead, BinWrite)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum SharedContainerMutability {
    Immutable = 0,
    Mutable = 1,
}


pub mod mutability_as_int {
    use super::SharedContainerMutability;
    use crate::prelude::*;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        value: &SharedContainerMutability,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            SharedContainerMutability::Mutable => serializer.serialize_u8(0),
            SharedContainerMutability::Immutable => serializer.serialize_u8(1),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<SharedContainerMutability, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = u8::deserialize(deserializer)?;
        Ok(match opt {
            0 => SharedContainerMutability::Mutable,
            1 => SharedContainerMutability::Immutable,
            x => {
                return Err(D::Error::custom(format!(
                    "invalid mutability code: {}",
                    x
                )));
            }
        })
    }
}
pub mod mutability_option_as_int {
    use super::SharedContainerMutability;

    use crate::prelude::*;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        value: &Option<SharedContainerMutability>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(SharedContainerMutability::Mutable) => {
                serializer.serialize_u8(0)
            }
            Some(SharedContainerMutability::Immutable) => {
                serializer.serialize_u8(1)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<SharedContainerMutability>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<u8>::deserialize(deserializer)?;
        Ok(match opt {
            Some(0) => Some(SharedContainerMutability::Mutable),
            Some(1) => Some(SharedContainerMutability::Immutable),
            Some(x) => {
                return Err(D::Error::custom(format!(
                    "invalid mutability code: {}",
                    x
                )));
            }
            None => None,
        })
    }
}


impl Display for SharedContainerMutability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainerMutability::Mutable => write!(f, "mut"),
            SharedContainerMutability::Immutable => write!(f, ""),
        }
    }
}