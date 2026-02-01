pub mod crypto;
pub mod time;

use crate::compat::sync::Arc;
use crypto::CryptoMock;
use crate::runtime::global_context::{set_global_context, GlobalContext};
use crate::mock_globals::time::TimeMock;

pub fn get_global_mock_context() -> GlobalContext {
    GlobalContext {
        crypto: Arc::new(CryptoMock),
        time: Arc::new(TimeMock),
        #[cfg(feature = "debug")]
        debug_flags: crate::runtime::global_context::DebugFlags::default(),
    }
}

pub fn init_global_mock_context() {
    let global_ctx = get_global_mock_context();
    set_global_context(global_ctx);
}
