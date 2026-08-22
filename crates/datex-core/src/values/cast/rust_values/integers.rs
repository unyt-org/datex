#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::integer::typed_integer::TypedInteger;

    impl ToDatexExpressionData for u8 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U8(*self))
        }
    }
    
    impl ToDatexExpressionData for u16 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U16(*self))
        }
    }
    
    impl ToDatexExpressionData for u32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U32(*self))
        }
    }
    
    impl ToDatexExpressionData for u64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U64(*self))
        }
    }
    
    impl ToDatexExpressionData for u128 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U128(*self))
        }
    }
    
    impl ToDatexExpressionData for i8 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I8(*self))
        }
    }
    
    impl ToDatexExpressionData for i16 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I16(*self))
        }
    }
    
    impl ToDatexExpressionData for i32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I32(*self))
        }
    }
    
    impl ToDatexExpressionData for i64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I64(*self))
        }
    }
    
    impl ToDatexExpressionData for i128 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I128(*self))
        }
    }
    
    impl ToDatexExpressionData for usize {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            #[cfg(target_pointer_width = "32")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::U32(*self as u32))
            }
            #[cfg(target_pointer_width = "64")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::U64(*self as u64))
            }
        }
    }
    
    impl ToDatexExpressionData for isize {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            #[cfg(target_pointer_width = "32")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::I32(*self as i32))
            }
            #[cfg(target_pointer_width = "64")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::I64(*self as i64))
            }
        }
    }
}
