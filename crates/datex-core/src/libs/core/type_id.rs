use core::mem::variant_count;

use crate::{
    libs::core::core_lib_id::{CoreLibIdTrait, TYPE_VARIANT_SPACE_BASE},
    prelude::*,
    shared_values::pointer_address::{ExternalPointerAddress, PointerAddress},
    values::core_values::{
        decimal::typed_decimal::DecimalTypeVariant,
        integer::typed_integer::IntegerTypeVariant,
    },
};
use datex_macros_internal::CoreLibString;
use num_enum::TryFromPrimitive;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, CoreLibString, TryFromPrimitive,
)]
#[repr(u16)]
pub enum CoreLibBaseTypeId {
    Type,     // #core.type
    Null,     // #core.null
    Boolean,  // #core.boolean
    Integer,  // #core.integer
    Decimal,  // #core.decimal
    Text,     // #core.text
    Endpoint, // #core.endpoint
    List,     // #core.List
    Map,      // #core.Map
    Callable, // #core.Callable
    Unit,     // #core.Unit
    Never,    // #core.never
    Unknown,  // #core.unknown
    Range,    // #core.range
}

const INTEGER_VARIANT_COUNT: u16 = variant_count::<IntegerTypeVariant>() as u16;
const DECIMAL_VARIANT_COUNT: u16 = variant_count::<DecimalTypeVariant>() as u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CoreLibVariantTypeId {
    Integer(IntegerTypeVariant),
    Decimal(DecimalTypeVariant),
}

impl CoreLibIdTrait for CoreLibVariantTypeId {
    fn to_u16(&self) -> u16 {
        TYPE_VARIANT_SPACE_BASE
            + match self {
                CoreLibVariantTypeId::Integer(variant) => *variant as u16,
                CoreLibVariantTypeId::Decimal(variant) => {
                    INTEGER_VARIANT_COUNT + (*variant as u16)
                }
            }
    }

    fn try_from_u16(id: u16) -> Option<Self> {
        let id = id - TYPE_VARIANT_SPACE_BASE;
        if id < INTEGER_VARIANT_COUNT {
            Some(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::try_from(id as u8).unwrap(),
            ))
        } else if id < INTEGER_VARIANT_COUNT + DECIMAL_VARIANT_COUNT {
            Some(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::try_from(
                    (id - INTEGER_VARIANT_COUNT) as u8,
                )
                .unwrap(),
            ))
        } else {
            None
        }
    }

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

impl CoreLibIdTrait for CoreLibBaseTypeId {
    fn to_u16(&self) -> u16 {
        *self as u16
    }

    fn try_from_u16(id: u16) -> Option<Self> {
        match id {
            0 => Some(CoreLibBaseTypeId::Null),
            1 => Some(CoreLibBaseTypeId::Type),
            2 => Some(CoreLibBaseTypeId::Boolean),
            3 => Some(CoreLibBaseTypeId::Callable),
            4 => Some(CoreLibBaseTypeId::Endpoint),
            5 => Some(CoreLibBaseTypeId::Text),
            6 => Some(CoreLibBaseTypeId::List),
            7 => Some(CoreLibBaseTypeId::Unit),
            8 => Some(CoreLibBaseTypeId::Map),
            9 => Some(CoreLibBaseTypeId::Never),
            10 => Some(CoreLibBaseTypeId::Unknown),
            11 => Some(CoreLibBaseTypeId::Range),

            TYPE_INTEGER_BASE => Some(CoreLibBaseTypeId::Integer(None)),
            n if (TYPE_INTEGER_BASE + 1..TYPE_DECIMAL_BASE).contains(&n) => {
                IntegerTypeVariant::try_from((n - TYPE_INTEGER_BASE) as u8)
                    .ok()
                    .map(|v| CoreLibBaseTypeId::Integer(Some(v)))
            }

            TYPE_DECIMAL_BASE => Some(CoreLibBaseTypeId::Decimal(None)),
            n if n > TYPE_DECIMAL_BASE => {
                DecimalTypeVariant::try_from((n - TYPE_DECIMAL_BASE) as u8)
                    .ok()
                    .map(|v| CoreLibBaseTypeId::Decimal(Some(v)))
            }

            _ => None,
        }
    }

    fn to_bytes(&self) -> [u8; 3] {
        (self.to_u16() as u32).to_le_bytes()[0..3]
            .try_into()
            .unwrap()
    }

    fn name(&self) -> String {
        Self::to_string(self)
    }
}

impl From<CoreLibBaseTypeId> for PointerAddress {
    fn from(id: CoreLibBaseTypeId) -> Self {
        PointerAddress::External(ExternalPointerAddress::from(&id))
    }
}

impl From<&CoreLibBaseTypeId> for ExternalPointerAddress {
    fn from(id: &CoreLibBaseTypeId) -> Self {
        ExternalPointerAddress::Builtin(id.to_bytes())
    }
}

impl TryFrom<&ExternalPointerAddress> for CoreLibBaseTypeId {
    type Error = ();
    fn try_from(address: &ExternalPointerAddress) -> Result<Self, Self::Error> {
        match address {
            ExternalPointerAddress::Builtin(bytes) => {
                let mut id_array = [0u8; 4];
                id_array[0..3].copy_from_slice(bytes);
                let id = u32::from_le_bytes(id_array);
                match CoreLibBaseTypeId::try_from_u16(id as u16) {
                    Some(core_id) => Ok(core_id),
                    None => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}
