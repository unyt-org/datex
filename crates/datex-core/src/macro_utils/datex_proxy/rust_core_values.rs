use crate::macro_utils::datex_proxy::DatexValueProxy;

impl DatexValueProxy for u8 {}
impl DatexValueProxy for u16 {}
impl DatexValueProxy for u32 {}
impl DatexValueProxy for u64 {}
impl DatexValueProxy for i8 {}
impl DatexValueProxy for i16 {}
impl DatexValueProxy for i32 {}
impl DatexValueProxy for i64 {}
impl DatexValueProxy for f32 {}
impl DatexValueProxy for f64 {}
impl DatexValueProxy for String {}

// TODO not implemented yet
// impl DatexValueProxy for char {}
// impl DatexValueProxy for isize {}
// impl DatexValueProxy for usize {}
