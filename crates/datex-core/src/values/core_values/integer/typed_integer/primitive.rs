use num_traits::ToPrimitive;

use crate::values::core_values::integer::typed_integer::TypedInteger;

macro_rules! impl_typed_integer_to_primitive {
    ($method:ident, $target:ty) => {
        #[inline]
        fn $method(&self) -> Option<$target> {
            match self {
                TypedInteger::IBig(value) => ToPrimitive::$method(value),

                TypedInteger::I8(value) => ToPrimitive::$method(value),
                TypedInteger::I16(value) => ToPrimitive::$method(value),
                TypedInteger::I32(value) => ToPrimitive::$method(value),
                TypedInteger::I64(value) => ToPrimitive::$method(value),
                TypedInteger::I128(value) => ToPrimitive::$method(value),

                TypedInteger::U8(value) => ToPrimitive::$method(value),
                TypedInteger::U16(value) => ToPrimitive::$method(value),
                TypedInteger::U32(value) => ToPrimitive::$method(value),
                TypedInteger::U64(value) => ToPrimitive::$method(value),
                TypedInteger::U128(value) => ToPrimitive::$method(value),
            }
        }
    };
}

impl ToPrimitive for TypedInteger {
    impl_typed_integer_to_primitive!(to_isize, isize);
    impl_typed_integer_to_primitive!(to_i8, i8);
    impl_typed_integer_to_primitive!(to_i16, i16);
    impl_typed_integer_to_primitive!(to_i32, i32);
    impl_typed_integer_to_primitive!(to_i64, i64);
    impl_typed_integer_to_primitive!(to_i128, i128);

    impl_typed_integer_to_primitive!(to_usize, usize);
    impl_typed_integer_to_primitive!(to_u8, u8);
    impl_typed_integer_to_primitive!(to_u16, u16);
    impl_typed_integer_to_primitive!(to_u32, u32);
    impl_typed_integer_to_primitive!(to_u64, u64);
    impl_typed_integer_to_primitive!(to_u128, u128);

    impl_typed_integer_to_primitive!(to_f32, f32);
    impl_typed_integer_to_primitive!(to_f64, f64);
}
