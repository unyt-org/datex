use num_traits::ToPrimitive;

use crate::values::core_values::integer::Integer;

macro_rules! impl_integer_to_primitive {
    ($method:ident, $target:ty) => {
        #[inline]
        fn $method(&self) -> Option<$target> {
            ToPrimitive::$method(&self.0)
        }
    };
}

impl ToPrimitive for Integer {
    impl_integer_to_primitive!(to_isize, isize);
    impl_integer_to_primitive!(to_i8, i8);
    impl_integer_to_primitive!(to_i16, i16);
    impl_integer_to_primitive!(to_i32, i32);
    impl_integer_to_primitive!(to_i64, i64);
    impl_integer_to_primitive!(to_i128, i128);

    impl_integer_to_primitive!(to_usize, usize);
    impl_integer_to_primitive!(to_u8, u8);
    impl_integer_to_primitive!(to_u16, u16);
    impl_integer_to_primitive!(to_u32, u32);
    impl_integer_to_primitive!(to_u64, u64);
    impl_integer_to_primitive!(to_u128, u128);

    impl_integer_to_primitive!(to_f32, f32);
    impl_integer_to_primitive!(to_f64, f64);
}
