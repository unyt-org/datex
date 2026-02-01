use crate::runtime::global_context::get_global_context;

use crate::prelude::*;
pub fn generate_uuid_string() -> String {
    let crypto = get_global_context().crypto;
    crypto.create_uuid()
}
