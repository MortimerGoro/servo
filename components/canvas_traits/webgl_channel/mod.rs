#[cfg(feature = "webgl_ipc")]
mod ipc;
#[cfg(feature = "webgl_ipc")]
pub use self::ipc::*;

#[cfg(feature = "webgl_sync")]
mod sync;
#[cfg(feature = "webgl_sync")]
pub use self::sync::*;

#[cfg(all(not(feature = "webgl_sync"), not(feature = "webgl_ipc")))]
mod mpsc;
#[cfg(all(not(feature = "webgl_sync"), not(feature = "webgl_ipc")))]
pub use self::mpsc::*;