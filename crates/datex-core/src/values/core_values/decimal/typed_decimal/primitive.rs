use num_traits::ToPrimitive;

use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

macro_rules! impl_to_primitive_method {
    ($method:ident, $output:ty) => {
        #[inline]
        fn $method(&self) -> Option<$output> {
            match self {
                TypedDecimal::F32(value) => value.into_inner().$method(),
                TypedDecimal::F64(value) => value.into_inner().$method(),
                TypedDecimal::Decimal(value) => value.$method(),
            }
        }
    };
}

impl ToPrimitive for TypedDecimal {
    impl_to_primitive_method!(to_isize, isize);
    impl_to_primitive_method!(to_i8, i8);
    impl_to_primitive_method!(to_i16, i16);
    impl_to_primitive_method!(to_i32, i32);
    impl_to_primitive_method!(to_i64, i64);
    impl_to_primitive_method!(to_i128, i128);

    impl_to_primitive_method!(to_usize, usize);
    impl_to_primitive_method!(to_u8, u8);
    impl_to_primitive_method!(to_u16, u16);
    impl_to_primitive_method!(to_u32, u32);
    impl_to_primitive_method!(to_u64, u64);
    impl_to_primitive_method!(to_u128, u128);

    impl_to_primitive_method!(to_f32, f32);
    impl_to_primitive_method!(to_f64, f64);
}
