use binrw::io::Cursor;
use crate::prelude::*;

pub mod type_compiler;
pub mod value_compiler;
pub mod shared_value_tracking;
mod core_compiler_context;

pub type ByteCursor = Cursor<Vec<u8>>;