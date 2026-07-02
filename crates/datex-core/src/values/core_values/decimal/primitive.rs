use num_traits::ToPrimitive;

use crate::values::core_values::decimal::Decimal;

macro_rules! impl_decimal_to_integer {
    ($method:ident, $target:ty) => {
        #[inline]
        fn $method(&self) -> Option<$target> {
            match self {
                Decimal::Nan | Decimal::Infinity | Decimal::NegInfinity => None,
                Decimal::Zero | Decimal::NegZero => Some(0),
                Decimal::Finite(value) => ToPrimitive::$method(value),
            }
        }
    };
}

macro_rules! impl_decimal_to_float {
    ($method:ident, $target:ty) => {
        #[inline]
        fn $method(&self) -> Option<$target> {
            match self {
                Decimal::Nan => Some(<$target>::NAN),
                Decimal::Infinity => Some(<$target>::INFINITY),
                Decimal::NegInfinity => Some(<$target>::NEG_INFINITY),
                Decimal::Zero => Some(0.0),
                Decimal::NegZero => Some(-0.0),
                Decimal::Finite(value) => ToPrimitive::$method(value),
            }
        }
    };
}

impl ToPrimitive for Decimal {
    impl_decimal_to_integer!(to_isize, isize);
    impl_decimal_to_integer!(to_i8, i8);
    impl_decimal_to_integer!(to_i16, i16);
    impl_decimal_to_integer!(to_i32, i32);
    impl_decimal_to_integer!(to_i64, i64);
    impl_decimal_to_integer!(to_i128, i128);

    impl_decimal_to_integer!(to_usize, usize);
    impl_decimal_to_integer!(to_u8, u8);
    impl_decimal_to_integer!(to_u16, u16);
    impl_decimal_to_integer!(to_u32, u32);
    impl_decimal_to_integer!(to_u64, u64);
    impl_decimal_to_integer!(to_u128, u128);

    impl_decimal_to_float!(to_f32, f32);
    impl_decimal_to_float!(to_f64, f64);
}
