#[cfg(feature = "webgl_multiprocess")]
mod multiprocess;
#[cfg(feature = "webgl_multiprocess")]
pub use self::multiprocess::WebGLThreads;

#[cfg(feature = "webgl_sync")]
mod sync;
#[cfg(feature = "webgl_sync")]
pub use self::sync::WebGLThreads;

#[cfg(all(not(feature = "webgl_sync"), not(feature = "webgl_multiprocess")))]
mod inprocess;
#[cfg(all(not(feature = "webgl_sync"), not(feature = "webgl_multiprocess")))]
pub use self::inprocess::WebGLThreads;