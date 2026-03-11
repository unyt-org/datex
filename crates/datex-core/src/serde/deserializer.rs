use crate::{
    runtime::execution::{ExecutionInput, ExecutionOptions, execute_dxb_sync},
    serde::error::DeserializationError,
    values::{
        core_value::CoreValue,
        core_values::{
            integer::typed_integer::TypedInteger, map::BorrowedMapKey,
        },
        value,
        value::Value,
        value_container::ValueContainer,
    },
};
use core::{result::Result, unreachable};
use serde::{
    Deserialize, Deserializer,
    de::{
        DeserializeOwned, EnumAccess, IntoDeserializer, VariantAccess, Visitor,
        value::StrDeserializer,
    },
    forward_to_deserialize_any,
};

use crate::{prelude::*, runtime::RuntimeInternal};
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;

/// Deserialize a value of type T from a byte slice containing DXB data
pub fn from_bytes<T>(input: &[u8]) -> Result<T, DeserializationError>
where
    T: DeserializeOwned,
{
    let runtime = RuntimeInternal::stub();
    let context = ExecutionInput::new(
        input,
        ExecutionCallerMetadata::local_default(),
        ExecutionOptions { verbose: true },
        Rc::new(runtime),
    );
    let value = execute_dxb_sync(context)
        .map_err(DeserializationError::ExecutionError)?
        .expect("DXB execution returned no value");

    let deserializer = DatexDeserializer::new_from_value_container(&value);
    T::deserialize(deserializer)
}

#[cfg(feature = "compiler")]
pub fn from_script<T>(script: &str) -> Result<T, DeserializationError>
where
    T: DeserializeOwned,
{
    let (dxb, _) = crate::compiler::compile_script(
        script,
        crate::compiler::CompileOptions::default(),
    )
    .map_err(|err| DeserializationError::CanNotReadFile(err.to_string()))?;
    from_bytes(&dxb)
}

#[cfg(all(feature = "std", feature = "compiler"))]
pub fn from_dx_file<T>(
    path: std::path::PathBuf,
) -> Result<T, DeserializationError>
where
    T: DeserializeOwned,
{
    let input = std::fs::read_to_string(path)
        .map_err(|err| DeserializationError::CanNotReadFile(err.to_string()))?;
    from_script(&input)
}

/// Create a deserializer from a DX script string
/// This will extract a static value from the script without executing it
/// and use that value for deserialization
/// If no static value is found, an error is returned
/// This is useful for deserializing simple values like integer, text, map and list
/// without the need to execute the script
/// Note: This does not support expressions or computations in the script
/// For example, the script `{ "key": 42 }` will work, but the script `{ "key": 40 + 2 }` will not
/// because the latter requires execution to evaluate the expression
/// and extract the value
#[cfg(feature = "compiler")]
pub fn from_static_script<T>(script: &str) -> Result<T, DeserializationError>
where
    T: DeserializeOwned,
{
    let value = crate::compiler::extract_static_value_from_script(script)
        .map_err(DeserializationError::CompilerError)?
        .ok_or(DeserializationError::NoStaticValueFound)?;
    let deserializer = DatexDeserializer::new_from_value_container(&value);
    T::deserialize(deserializer)
}

/// Deserialize a value of type T from a ValueContainer
pub fn from_value_container<T>(
    value: &ValueContainer,
) -> Result<T, DeserializationError>
where
    T: serde::de::DeserializeOwned,
{
    let deserializer = DatexDeserializer::new_from_value_container(value);
    T::deserialize(deserializer)
}

#[derive(Clone)]
pub enum DatexDeserializer<'de> {
    ValueContainer(&'de ValueContainer),
    Text(&'de str),
}

impl<'de> DatexDeserializer<'de> {
    fn new_from_value_container(value: &'de ValueContainer) -> Self {
        Self::ValueContainer(value)
    }
    fn new_from_str(text: &'de str) -> Self {
        Self::Text(text)
    }
    fn new_from_borrowed_map_key(key: BorrowedMapKey<'de>) -> Self {
        match key {
            BorrowedMapKey::Text(s) => Self::Text(s),
            BorrowedMapKey::Value(v) => Self::ValueContainer(v),
        }
    }

    pub(crate) fn to_value_container(&self) -> Cow<'de, ValueContainer> {
        match self {
            DatexDeserializer::ValueContainer(v) => Cow::Borrowed(v),
            DatexDeserializer::Text(s) => Cow::Owned(ValueContainer::from(*s)),
        }
    }
}

impl<'de> IntoDeserializer<'de, DeserializationError>
    for DatexDeserializer<'de>
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
impl<'de> Deserializer<'de> for DatexDeserializer<'de> {
    type Error = DeserializationError;

    forward_to_deserialize_any! {
        bool char str string bytes byte_buf
        tuple seq unit struct ignored_any
    }

    /// Deserialize any value from the value container
    /// This is the main entry point for deserialization
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(s) => visitor.visit_string(s.to_string()),
            DatexDeserializer::ValueContainer(value) => match value {
                // TODO #148 implement missing mapping
                ValueContainer::Local(value::Value { inner, .. }) => {
                    match inner {
                        CoreValue::Null => visitor.visit_none(),
                        CoreValue::Boolean(b) => visitor.visit_bool(b.0),
                        CoreValue::TypedInteger(i) => match i {
                            TypedInteger::I128(i) => visitor.visit_i128(*i),
                            TypedInteger::U128(u) => visitor.visit_u128(*u),
                            TypedInteger::I64(i) => visitor.visit_i64(*i),
                            TypedInteger::U64(u) => visitor.visit_u64(*u),
                            TypedInteger::I32(i) => visitor.visit_i32(*i),
                            TypedInteger::U32(u) => visitor.visit_u32(*u),
                            TypedInteger::I16(i) => visitor.visit_i16(*i),
                            TypedInteger::U16(u) => visitor.visit_u16(*u),
                            TypedInteger::I8(i) => visitor.visit_i8(*i),
                            TypedInteger::U8(u) => visitor.visit_u8(*u),
                            TypedInteger::IBig(i) => {
                                visitor.visit_i128(i.as_i128().unwrap())
                            }
                        },
                        CoreValue::Text(s) => visitor.visit_string(s.0.clone()),
                        CoreValue::Endpoint(endpoint) => {
                            let endpoint_str = endpoint.to_string();
                            visitor.visit_string(endpoint_str)
                        }
                        CoreValue::Map(obj) => {
                            let map = obj
                                .iter()
                                .map(|(k, v)| {
                                    (
                                DatexDeserializer::new_from_borrowed_map_key(k),
                                DatexDeserializer::new_from_value_container(v),
                            )
                                })
                                .collect::<Vec<_>>();
                            visitor.visit_map(
                                serde::de::value::MapDeserializer::new(
                                    map.into_iter(),
                                ),
                            )
                        }
                        CoreValue::List(list) => {
                            let vec: Vec<DatexDeserializer<'de>> = list
                                .iter()
                                .map(
                                    DatexDeserializer::new_from_value_container,
                                )
                                .collect::<Vec<_>>();
                            visitor.visit_seq(
                                serde::de::value::SeqDeserializer::new(
                                    vec.into_iter(),
                                ),
                            )
                        }
                        e => unreachable!("Unsupported core value: {:?}", e),
                    }
                }
                _ => unreachable!("Refs are not supported in deserialization"),
            },
        }
    }

    /// Deserialize unit structs from the value container
    /// For example:
    ///     struct MyUnitStruct;
    /// will be deserialized from:
    ///     ()
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    /// Deserialize options from null or some value in the value container
    /// For example:
    ///     Some(42) will be deserialized from 42
    ///     None will be deserialized from null
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            DatexDeserializer::ValueContainer(value)
                if value.to_value().borrow().is_null() =>
            {
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    /// Deserialize newtype structs from single values or tuples in the value container
    /// For example:
    ///     struct MyNewtypeStruct(i32);
    /// will be deserialized from:
    ///     42
    /// or
    ///     (42,)
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // TODO #395: handle structurally typed maps and lists
        // if let ValueContainer::Value(Value {
        //     inner: CoreValue::Array(array),
        //     ..
        // }) = self.value
        // {
        //     let values = array.into_iter().map(DatexDeserializer::from_value);
        //     visitor.visit_seq(serde::de::value::SeqDeserializer::new(values))
        // } else if let ValueContainer::Value(Value {
        //     inner: CoreValue::Struct(structure),
        //     ..
        // }) = &self.value
        // {
        //     if structure.size() == 2 {
        //         let first_entry = structure.at_unchecked(0);
        //         if let ValueContainer::Value(Value {
        //             inner: CoreValue::Text(text),
        //             ..
        //         }) = first_entry
        //             && text.0.starts_with("datex::")
        //         {
        //             let second_entry = structure.at_unchecked(1);
        //             return visitor.visit_newtype_struct(
        //                 DatexDeserializer::from_value(second_entry.clone()),
        //             );
        //         }
        //     }
        //     visitor
        //         .visit_newtype_struct(DatexDeserializer::from_value(self.value))
        // } else {
        //
        // }

        visitor.visit_seq(serde::de::value::SeqDeserializer::new(
            vec![self].into_iter(),
        ))
    }

    /// Deserialize tuple structs from a list in the value container
    /// For example:
    ///     struct MyTupleStruct(i32, String);
    /// will be deserialized from:
    ///     [42, "Hello"]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let DatexDeserializer::ValueContainer(ValueContainer::Local(
            Value {
                inner: CoreValue::List(list),
                ..
            },
        )) = self
        {
            visitor.visit_seq(serde::de::value::SeqDeserializer::new(
                list.iter().map(DatexDeserializer::new_from_value_container),
            ))
        } else {
            Err(DeserializationError::Custom(
                "expected map for tuple struct".to_string(),
            ))
        }
    }

    /// Deserialize maps from list of key-value pairs
    /// For example:
    ///     {"key1": value1, "key2": value2}
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let DatexDeserializer::ValueContainer(ValueContainer::Local(
            Value {
                inner: CoreValue::Map(map),
                ..
            },
        )) = self
        {
            let entries = map.iter().map(|(k, v)| {
                (
                    DatexDeserializer::new_from_borrowed_map_key(k),
                    DatexDeserializer::new_from_value_container(v),
                )
            });
            visitor.visit_map(serde::de::value::MapDeserializer::new(entries))
        } else {
            Err(DeserializationError::Custom("expected map".to_string()))
        }
    }

    /// Deserialize identifiers from various formats:
    /// - Direct text: "identifier"
    /// - Single-key map: {"Identifier": ...}
    /// - Tuple with single text element: ("identifier", ...)
    fn deserialize_identifier<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(s) => visitor.visit_string(s.to_string()),
            DatexDeserializer::ValueContainer(value) => match value {
                // Direct text
                ValueContainer::Local(Value {
                    inner: CoreValue::Text(s),
                    ..
                }) => visitor.visit_string(s.0.clone()),

                // Single-key map {"Identifier": ...}
                ValueContainer::Local(Value {
                    inner: CoreValue::Map(o),
                    ..
                }) => {
                    if o.size() == 1 {
                        let (key, _) = o.iter().next().unwrap();
                        if let BorrowedMapKey::Text(string) = key {
                            visitor.visit_string(string.to_string())
                        } else {
                            Err(DeserializationError::Custom(
                                "Expected text key for identifier".to_string(),
                            ))
                        }
                    } else {
                        Err(DeserializationError::Custom(
                            "Expected single-key map for identifier"
                                .to_string(),
                        ))
                    }
                }

                _ => Err(DeserializationError::Custom(
                    "Expected identifier".to_string(),
                )),
            },
        }
    }

    /// Deserialize enums from various formats:
    /// - Unit variants: "Variant"
    /// - Newtype variants: {"Variant": value}
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(s) => {
                visitor.visit_enum(EnumDeserializer {
                    variant: s,
                    value: None,
                })
            }
            DatexDeserializer::ValueContainer(value) => match value {
                // Default representation: ("Variant", value)
                value @ ValueContainer::Local(Value {
                    inner: CoreValue::List(t),
                    ..
                }) => {
                    if t.is_empty() {
                        return Err(DeserializationError::Custom(
                            "Expected non-empty tuple for enum".to_string(),
                        ));
                    }
                    let deserializer =
                        DatexDeserializer::new_from_value_container(value);
                    visitor.visit_enum(EnumDeserializer {
                        variant: "_tuple",
                        value: Some(deserializer),
                    })
                }

                // Map with single key = variant name
                ValueContainer::Local(Value {
                    inner: CoreValue::Map(o),
                    ..
                }) => {
                    if o.size() != 1 {
                        return Err(DeserializationError::Custom(
                            "Expected single-key map for enum".to_string(),
                        ));
                    }

                    let (variant_name, value) = o.iter().next().unwrap();
                    if let BorrowedMapKey::Text(variant) = variant_name {
                        let deserializer =
                            DatexDeserializer::new_from_value_container(value);
                        visitor.visit_enum(EnumDeserializer {
                            variant,
                            value: Some(deserializer),
                        })
                    } else {
                        Err(DeserializationError::Custom(
                            "Expected text variant name".to_string(),
                        ))
                    }
                }
                // TODO #396: handle structurally typed maps
                // ValueContainer::Value(Value {
                //     inner: CoreValue::Struct(o),
                //     ..
                // }) => {
                //     if o.size() != 1 {
                //         return Err(DeserializationError::Custom(
                //             "Expected single-key object for enum".to_string(),
                //         ));
                //     }
                //
                //     let (variant_name, value) = o.into_iter().next().unwrap();
                //
                //     let deserializer = DatexDeserializer::from_value(value);
                //     visitor.visit_enum(EnumDeserializer {
                //         variant: variant_name,
                //         value: deserializer,
                //     })
                // }

                // unit variants stored directly as text
                ValueContainer::Local(Value {
                    inner: CoreValue::Text(s),
                    ..
                }) => visitor.visit_enum(EnumDeserializer {
                    variant: &s.0,
                    value: None,
                }),

                e => Err(DeserializationError::Custom(format!(
                    "Expected enum representation, found: {}",
                    e
                ))),
            },
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("f32".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Decimal(decimal) => {
                        visitor.visit_f32(decimal.into_f32())
                    }
                    CoreValue::TypedDecimal(typed_decimal) => {
                        visitor.visit_f32(typed_decimal.as_f32())
                    }
                    CoreValue::Integer(integer) => {
                        visitor.visit_f32(integer.as_f32())
                    }
                    CoreValue::TypedInteger(typed_integer) => {
                        visitor.visit_f32(typed_integer.as_f32())
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "f32".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("f64".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Decimal(decimal) => {
                        visitor.visit_f64(decimal.into_f64())
                    }
                    CoreValue::TypedDecimal(typed_decimal) => {
                        visitor.visit_f64(typed_decimal.as_f64())
                    }
                    CoreValue::Integer(integer) => {
                        visitor.visit_f64(integer.as_f64())
                    }
                    CoreValue::TypedInteger(typed_integer) => {
                        visitor.visit_f64(typed_integer.as_f64())
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "f64".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("i8".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_i8(i.as_wrapped_i8())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_i8(i.as_integer().as_wrapped_i8())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_i8(d.as_integer().unwrap() as i8)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_i8(d.as_integer().unwrap() as i8)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "i8".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("i16".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_i16(i.as_wrapped_i16())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_i16(i.as_integer().as_wrapped_i16())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_i16(d.as_integer().unwrap() as i16)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_i16(d.as_integer().unwrap() as i16)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "i16".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("i32".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_i32(i.as_wrapped_i32())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_i32(i.as_integer().as_wrapped_i32())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_i32(d.as_integer().unwrap() as i32)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_i32(d.as_integer().unwrap() as i32)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "i32".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("i64".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_i64(i.as_wrapped_i64())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_i64(i.as_integer().as_wrapped_i64())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_i64(d.as_integer().unwrap())
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_i64(d.as_integer().unwrap())
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "i64".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("i128".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_i128(i.as_wrapped_i128())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_i128(i.as_integer().as_wrapped_i128())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_i128(d.as_integer().unwrap() as i128)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_i128(d.as_integer().unwrap() as i128)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "i128".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("u8".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_u8(i.as_wrapped_u8())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_u8(i.as_integer().as_wrapped_u8())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_u8(d.as_integer().unwrap() as u8)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_u8(d.as_integer().unwrap() as u8)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "u8".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("u16".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_u16(i.as_wrapped_u16())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_u16(i.as_integer().as_wrapped_u16())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_u16(d.as_integer().unwrap() as u16)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_u16(d.as_integer().unwrap() as u16)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "u16".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("u32".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_u32(i.as_wrapped_u32())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_u32(i.as_integer().as_wrapped_u32())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_u32(d.as_integer().unwrap() as u32)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_u32(d.as_integer().unwrap() as u32)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "u32".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("u64".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_u64(i.as_wrapped_u64())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_u64(i.as_integer().as_wrapped_u64())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_u64(d.as_integer().unwrap() as u64)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_u64(d.as_integer().unwrap() as u64)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "u64".to_string(),
                    )),
                }
            }
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DatexDeserializer::Text(_s) => {
                Err(DeserializationError::CanNotDeserialize("u128".to_string()))
            }
            DatexDeserializer::ValueContainer(value) => {
                match &value.to_value().borrow().inner {
                    CoreValue::Integer(i) => {
                        visitor.visit_u128(i.as_wrapped_u128())
                    }
                    CoreValue::TypedInteger(i) => {
                        visitor.visit_u128(i.as_integer().as_wrapped_u128())
                    }
                    CoreValue::Decimal(d) => {
                        visitor.visit_u128(d.as_integer().unwrap() as u128)
                    }
                    CoreValue::TypedDecimal(d) => {
                        visitor.visit_u128(d.as_integer().unwrap() as u128)
                    }
                    _ => Err(DeserializationError::CanNotDeserialize(
                        "u128".to_string(),
                    )),
                }
            }
        }
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// Enum deserializer helper
/// Used to deserialize enum variants
/// For example:
///     enum MyEnum {
///         Variant1,
///         Variant2(i32),
///     }
/// will be deserialized from:
///     "Variant1" or {"Variant2": 42}
struct EnumDeserializer<'de> {
    variant: &'de str,
    value: Option<DatexDeserializer<'de>>,
}
impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = DeserializationError;
    type Variant = VariantDeserializer<'de>;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize::<StrDeserializer<Self::Error>>(
            self.variant.into_deserializer(),
        )?;
        Ok((variant, VariantDeserializer { value: self.value }))
    }
}

/// Variant deserializer helper
/// Used to deserialize enum variant contents
/// For example:
///     enum MyEnum {
///         Variant1,
///         Variant2(i32),
///     }
/// will be deserialized from:
///     "Variant1" or {"Variant2": 42}
struct VariantDeserializer<'de> {
    value: Option<DatexDeserializer<'de>>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = DeserializationError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(DeserializationError::Custom(
                "Expected value for newtype variant".to_string(),
            )),
        }
    }

    fn tuple_variant<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(value) => value.deserialize_tuple(len, visitor),
            None => visitor.visit_unit(),
        }
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(value) => value.deserialize_struct("", fields, visitor),
            None => visitor.visit_unit(),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "compiler")]
mod tests {
    use super::*;
    use crate::{
        compiler::{CompileOptions, compile_script},
        serde::serializer::to_bytes,
        values::core_values::endpoint::Endpoint,
    };
    use serde::{Deserialize, Serialize};

    use crate::prelude::*;
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[derive(Deserialize, Serialize, Debug)]
    enum TestEnum {
        Variant1,
        Variant2,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestStruct2 {
        test_enum: TestEnum,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestWithOptionalField {
        optional_field: Option<String>,
    }

    #[derive(Deserialize)]
    struct TestStructWithEndpoint {
        endpoint: Endpoint,
    }

    #[derive(Deserialize)]
    struct TestStructWithOptionalEndpoint {
        endpoint: Option<Endpoint>,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestNestedStruct {
        nested: TestStruct,
    }

    #[test]
    fn nested_struct_serde() {
        let script = r#"
            {
                nested: {
                    field1: "Hello",
                    field2: 47
                }
            }
        "#;
        let result: TestNestedStruct = super::from_script(script).unwrap();
        assert_eq!(
            result,
            TestNestedStruct {
                nested: TestStruct {
                    field1: "Hello".to_string(),
                    field2: 47
                }
            }
        );
    }

    #[test]
    fn struct_from_bytes() {
        let data = to_bytes(&TestStruct {
            field1: "Hello".to_string(),
            field2: 42,
        })
        .unwrap();
        let result: TestStruct = from_bytes(&data).unwrap();
        assert!(!result.field1.is_empty());
    }

    #[test]
    fn from_script() {
        let script = r#"
            {
                field1: "Hello",
                field2: 42 + 5 // This will be evaluated to 47
            }
        "#;
        let result: TestStruct = super::from_script(script).unwrap();
        assert!(!result.field1.is_empty());
    }

    #[test]
    fn test_from_static_script() {
        let script = r#"
            {
                field1: "Hello",
                field2: 42
            }
        "#;
        let result: TestStruct = from_static_script(script).unwrap();
        assert!(!result.field1.is_empty());
    }

    #[test]
    fn enum_1() {
        let script = r#""Variant1""#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestEnum =
            from_bytes(&dxb).expect("Failed to deserialize TestEnum");
        assert!(core::matches!(result, TestEnum::Variant1));
    }

    #[test]
    fn enum_2() {
        let script = r#""Variant2""#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestEnum =
            from_bytes(&dxb).expect("Failed to deserialize TestEnum");
        assert!(core::matches!(result, TestEnum::Variant2));
    }

    #[test]
    fn struct_with_enum() {
        let script = r#"
            {
                test_enum: "Variant1"
            }
        "#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestStruct2 =
            from_bytes(&dxb).expect("Failed to deserialize TestStruct2");
        assert!(core::matches!(result.test_enum, TestEnum::Variant1));
    }

    #[test]
    fn endpoint() {
        let script = r#"
            {
                endpoint: @jonas
            }
        "#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestStructWithEndpoint = from_bytes(&dxb)
            .expect("Failed to deserialize TestStructWithEndpoint");
        assert_eq!(result.endpoint.to_string(), "@jonas");
    }

    #[test]
    fn optional_field() {
        let script = r#"
            {
                optional_field: "Optional Value"
            }
        "#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestWithOptionalField = from_bytes(&dxb)
            .expect("Failed to deserialize TestWithOptionalField");
        assert!(result.optional_field.is_some());
        assert_eq!(result.optional_field.unwrap(), "Optional Value");
    }

    #[test]
    fn optional_field_empty() {
        let script = r#"
            {
                optional_field: null
            }
        "#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestWithOptionalField = from_bytes(&dxb)
            .expect("Failed to deserialize TestWithOptionalField");
        assert!(result.optional_field.is_none());
    }

    #[test]
    fn optional_endpoint() {
        let script = r#"
            {
                endpoint: @jonas
            }
        "#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: TestStructWithOptionalEndpoint = from_bytes(&dxb)
            .expect("Failed to deserialize TestStructWithOptionalEndpoint");
        assert!(result.endpoint.is_some());
        assert_eq!(result.endpoint.unwrap().to_string(), "@jonas");
    }

    #[derive(Deserialize, Serialize, Debug)]
    enum ExampleEnum {
        Variant1(String),
        Variant2(i32),
    }

    #[test]
    fn map() {
        let script = "{Variant1: \"Hello\"}";
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: ExampleEnum =
            from_bytes(&dxb).expect("Failed to deserialize ExampleEnum");
        assert!(core::matches!(result, ExampleEnum::Variant1(_)));

        let script = r#"{"Variant2": 42}"#;
        let dxb = compile_script(script, CompileOptions::default())
            .expect("Failed to compile script")
            .0;
        let result: ExampleEnum =
            from_bytes(&dxb).expect("Failed to deserialize ExampleEnum");
        assert!(core::matches!(result, ExampleEnum::Variant2(_)));
    }
}
