use binrw::io::Cursor;
use crate::prelude::*;

pub mod type_compiler;
pub mod value_compiler;

pub type ByteCursor = Cursor<Vec<u8>>;