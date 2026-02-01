pub mod prelude {
    pub use core::{
        cmp, fmt, hash, iter, mem, ops, option, result, slice, str,
    };

    pub use core::cell::{Cell, RefCell};

    pub use core::{
        option::Option::{self, None, Some},
        result::Result::{self, Err, Ok},
    };

    pub use crate::{
        collections::{HashMap, HashSet},
        compat,
        rc::Rc,
        std_random::RandomState,
        std_sync::Mutex,
        string::String,
        vec::Vec,
    };

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub use alloc::format;

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub use alloc::vec;
}
