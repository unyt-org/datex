use crate::runtime::global_context::get_global_context;

use crate::{
    compat::heap::{boxed::Box, vec},
    rc::Rc,
    string::String,
    vec::Vec,
};
use alloc::format;
pub fn generate_uuid_string() -> String {
    let crypto = get_global_context().crypto;
    crypto.create_uuid()
}
