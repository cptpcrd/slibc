#[cfg(feature = "std")]
pub use std::ffi::{CStr, CString, FromBytesWithNulError, NulError, OsStr, OsString};

#[cfg(not(feature = "std"))]
mod core_impl;
#[cfg(not(feature = "std"))]
pub use core_impl::*;

#[cfg(all(feature = "alloc", not(feature = "std")))]
mod alloc_impl;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc_impl::*;

pub type RawFd = libc::c_int;

pub mod prelude {
    pub use super::RawFd;

    #[cfg(feature = "std")]
    pub use std::os::unix::ffi::{OsStrExt, OsStringExt};
}
