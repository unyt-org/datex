#[cfg(feature = "native_crypto")]
pub mod crypto;
#[cfg(feature = "native_time")]
pub mod time;

use crate::stdlib::sync::Arc;
use time::TimeNative;
use crypto::CryptoNative;
use crate::logger::init_logger_debug;
use crate::runtime::global_context::{set_global_context, GlobalContext};

#[cfg(all(
    feature = "native_crypto",
    feature = "native_time"
))]
pub fn get_global_context_native() -> GlobalContext {
    GlobalContext {
        crypto: Arc::new(CryptoNative),
        time: Arc::new(TimeNative),
        #[cfg(feature = "debug")]
        debug_flags: crate::runtime::global_context::DebugFlags::default(),
    }
}

#[cfg(all(
    feature = "native_crypto",
    feature = "native_time"
))]
pub fn init_global_context_native() {
    let global_ctx = get_global_context_native();
    set_global_context(global_ctx);
    init_logger_debug();
}
