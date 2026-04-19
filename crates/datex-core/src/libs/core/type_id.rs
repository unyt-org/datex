use core::{fmt::Display, mem::variant_count};
use core::str::FromStr;
use crate::{
    libs::core::core_lib_id::{
        CoreLibIdIndex, CoreLibIdTrait, TYPE_SPACE_BASE,
        TYPE_VARIANT_SPACE_BASE,
    },
    prelude::*,
    values::core_values::{
        decimal::typed_decimal::DecimalTypeVariant,
        integer::typed_integer::IntegerTypeVariant,
    },
};
use datex_macros_internal::CoreLibString;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{Display, EnumString};
use crate::libs::core::core_lib_id::CoreLibId;
use crate::libs::core::value_id::CoreLibValueId;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    IntoPrimitive,
    TryFromPrimitive,
    EnumString,
    Display
)]
#[repr(u16)]
#[strum(serialize_all = "snake_case")]
/// A base type defined in the core library
/// Every variant automatically gets mapped to a new nominal type definition with
/// the enum variant name in lowercase as the name, and stored in the core library map
/// with the name as a key.
pub enum CoreLibBaseTypeId {
    Type,     // #core.type
    Null,     // #core.null
    Boolean,  // #core.boolean
    Integer,  // #core.integer
    Decimal,  // #core.decimal
    Text,     // #core.text
    Endpoint, // #core.endpoint
    List,     // #core.List
    #[strum(serialize = "Map")]
    Map,      // #core.Map
    #[strum(serialize = "Callable")]
    Callable, // #core.Callable
    #[strum(serialize = "Unit")]
    Unit,     // #core.Unit
    #[strum(serialize = "Never")]
    Never,    // #core.never
    #[strum(serialize = "Unknown")]
    Unknown,  // #core.unknown
    #[strum(serialize = "Range")]
    Range,    // #core.range
}

const INTEGER_VARIANT_COUNT: u16 = variant_count::<IntegerTypeVariant>() as u16;
const DECIMAL_VARIANT_COUNT: u16 = variant_count::<DecimalTypeVariant>() as u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoreLibVariantTypeId {
    Integer(IntegerTypeVariant),
    Decimal(DecimalTypeVariant),
}

impl Display for CoreLibVariantTypeId {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{}/", CoreLibBaseTypeId::from(self.clone()))?;
        match self {
            CoreLibVariantTypeId::Integer(variant) => {
                write!(f, "{}", variant)
            }
            CoreLibVariantTypeId::Decimal(variant) => {
                write!(f, "{}", variant)
            }
        }
    }
}

impl TryFrom<CoreLibIdIndex> for CoreLibVariantTypeId {
    type Error = ();

    fn try_from(id: CoreLibIdIndex) -> Result<Self, Self::Error> {
        let id = id.checked_sub(TYPE_VARIANT_SPACE_BASE).ok_or(())?;
        if id < INTEGER_VARIANT_COUNT {
            Ok(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::try_from(id as u8).unwrap(),
            ))
        } else if id < (INTEGER_VARIANT_COUNT + DECIMAL_VARIANT_COUNT) {
            Ok(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::try_from(
                    (id - INTEGER_VARIANT_COUNT) as u8,
                )
                .unwrap(),
            ))
        } else {
            Err(())
        }
    }
}

impl Into<CoreLibIdIndex> for CoreLibVariantTypeId {
    fn into(self) -> CoreLibIdIndex {
        CoreLibIdIndex(
            TYPE_VARIANT_SPACE_BASE
                + match self {
                    CoreLibVariantTypeId::Integer(variant) => variant as u16,
                    CoreLibVariantTypeId::Decimal(variant) => {
                        INTEGER_VARIANT_COUNT + (variant as u16)
                    }
                },
        )
    }
}

impl CoreLibVariantTypeId {
    pub fn base_type_id(&self) -> CoreLibBaseTypeId {
        match self {
            CoreLibVariantTypeId::Integer(_) => CoreLibBaseTypeId::Integer,
            CoreLibVariantTypeId::Decimal(_) => CoreLibBaseTypeId::Decimal,
        }
    }
    pub fn variant_name(&self) -> String {
        match self {
            CoreLibVariantTypeId::Integer(variant) => format!("{}", variant),
            CoreLibVariantTypeId::Decimal(variant) => format!("{}", variant),
        }
    }
    pub fn variant_ids(
        base_id: &CoreLibBaseTypeId,
    ) -> Vec<CoreLibVariantTypeId> {
        match base_id {
            CoreLibBaseTypeId::Integer => IntegerTypeVariant::iter()
                .map(CoreLibVariantTypeId::Integer)
                .collect(),
            CoreLibBaseTypeId::Decimal => DecimalTypeVariant::iter()
                .map(CoreLibVariantTypeId::Decimal)
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl CoreLibIdTrait for CoreLibVariantTypeId {
    fn name(&self) -> String {
        match self {
            CoreLibVariantTypeId::Integer(variant) => {
                format!("{}", variant)
            }
            CoreLibVariantTypeId::Decimal(variant) => {
                format!("{}", variant)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoreLibTypeId {
    Base(CoreLibBaseTypeId),
    Variant(CoreLibVariantTypeId),
}

impl From<CoreLibBaseTypeId> for CoreLibTypeId {
    fn from(id: CoreLibBaseTypeId) -> Self {
        CoreLibTypeId::Base(id)
    }
}

impl From<CoreLibVariantTypeId> for CoreLibTypeId {
    fn from(id: CoreLibVariantTypeId) -> Self {
        CoreLibTypeId::Variant(id)
    }
}


impl CoreLibTypeId {
    pub fn try_from_str(string: &str) -> Option<Self> {
        CoreLibBaseTypeId::from_str(string)
            .map(CoreLibTypeId::Base)
            .ok()
    }
}

impl From<CoreLibVariantTypeId> for CoreLibBaseTypeId {
    fn from(id: CoreLibVariantTypeId) -> Self {
        match id {
            CoreLibVariantTypeId::Integer(_) => CoreLibBaseTypeId::Integer,
            CoreLibVariantTypeId::Decimal(_) => CoreLibBaseTypeId::Decimal,
        }
    }
}

impl CoreLibIdTrait for CoreLibTypeId {
    fn name(&self) -> String {
        match self {
            CoreLibTypeId::Base(base_id) => base_id.name(),
            CoreLibTypeId::Variant(variant_id) => variant_id.name(),
        }
    }
}

impl Display for CoreLibTypeId {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        match self {
            CoreLibTypeId::Base(base_id) => {
                write!(f, "{}", base_id.to_string())
            }
            CoreLibTypeId::Variant(variant_id) => {
                write!(f, "{}", variant_id.to_string())
            }
        }
    }
}

impl From<CoreLibTypeId> for CoreLibIdIndex {
    fn from(type_id: CoreLibTypeId) -> Self {
        match type_id {
            CoreLibTypeId::Base(base_id) => base_id.into(),
            CoreLibTypeId::Variant(variant_id) => variant_id.into(),
        }
    }
}
impl TryFrom<CoreLibIdIndex> for CoreLibTypeId {
    type Error = ();

    fn try_from(value: CoreLibIdIndex) -> Result<Self, Self::Error> {
        if let Ok(base_id) = CoreLibBaseTypeId::try_from(value) {
            Ok(CoreLibTypeId::Base(base_id))
        } else if let Ok(variant_id) = CoreLibVariantTypeId::try_from(value) {
            Ok(CoreLibTypeId::Variant(variant_id))
        } else {
            Err(())
        }
    }
}

impl CoreLibIdTrait for CoreLibBaseTypeId {
    fn name(&self) -> String {
        Self::to_string(self)
    }
}

impl From<CoreLibBaseTypeId> for CoreLibIdIndex {
    fn from(base_id: CoreLibBaseTypeId) -> Self {
        CoreLibIdIndex((base_id as u16) + TYPE_SPACE_BASE)
    }
}
impl TryFrom<CoreLibIdIndex> for CoreLibBaseTypeId {
    type Error = ();

    fn try_from(id: CoreLibIdIndex) -> Result<Self, Self::Error> {
        let id = id.0.checked_sub(TYPE_SPACE_BASE).ok_or(())?;
        CoreLibBaseTypeId::try_from(id).map_err(|_| ())
    }
}
