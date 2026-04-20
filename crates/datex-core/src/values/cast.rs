use crate::types::nominal_type_definition::NominalTypeDefinition;
use crate::types::r#type::Type;
use crate::values::core_value::CoreValue;
use crate::values::core_values::boolean::Boolean;
use crate::values::core_values::callable::Callable;
use crate::values::core_values::decimal::Decimal;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;
use crate::values::core_values::endpoint::Endpoint;
use crate::values::core_values::integer::Integer;
use crate::values::core_values::integer::typed_integer::TypedInteger;
use crate::values::core_values::list::List;
use crate::values::core_values::map::Map;
use crate::values::core_values::range::Range;
use crate::values::core_values::text::Text;

macro_rules! impl_try_from_core_value {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl TryFrom<CoreValue> for $type {
                type Error = ();
                fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                    match value { CoreValue::$variant(v) => Ok(v), _ => Err(()) }
                }
            }
        )*
    };
}

// Implement [TryFrom] for each CoreValue variant
impl_try_from_core_value! {
    Integer             => Integer,
    TypedInteger        => TypedInteger,
    Decimal             => Decimal,
    TypedDecimal        => TypedDecimal,
    Boolean             => Boolean,
    Endpoint            => Endpoint,
    Text                => Text,
    List                => List,
    Map                 => Map,
    Type                => Type,
    NominalTypeDefinition => NominalTypeDefinition,
    Range               => Range,
    Callable            => Callable,
}