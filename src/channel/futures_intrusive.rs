#[cfg(not(feature = "std"))]
pub use futures_intrusive::sync::LocalManualResetEvent as ManualResetEvent;
/// Re-export of `ManualResetEvent` from the `futures-intrusive` crate,
/// adapted for `no_std` environments.
/// When compiled with the `std` feature, it uses the standard `ManualResetEvent`.
/// Otherwise, it uses `LocalManualResetEvent` for `no_std` compatibility.
/// In this case, it is not `Send` and `Sync` and can therefore not
/// be used across threads.
#[cfg(feature = "std")]
pub use futures_intrusive::sync::ManualResetEvent;
