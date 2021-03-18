//! ## General rules
//!
//! - Unless otherwise stated, for functions that take a buffer argument to store data in:
//!    - If the function fails, the contents of the buffer are unspecified.
//!    - If a terminating nul is placed in the buffer, the contents of the buffer beyond the
//!      terminating nul are unspecified (they may have been overwritten).
//!
//! ## Safety
//!
//! Some libc functions *cannot* be made safe, often because of thread-safety issues. If a function
//! in this crate is `unsafe`, its use should be taken seriously. Do **NOT** fall into the trap of
//! simply adding `unsafe { }` around everything.
//!
//! ## File descriptor handling
//!
//! WARNING: Unlike with the standard library, file descriptors opened with using `slibc` do NOT
//! have their close-on-exec flag set by default! In most cases, the documentation for each
//! function will tell you what the close-on-exec equivalent is.
//!
//! For all libc functions that return a file descriptor (such as `dup()`), the equivalent function
//! in `slibc` returns a [`FileDesc`] struct. This struct wraps a file descriptor,
//! automatically closes it when dropped, and provides several helper methods. If the `std` feature
//! is enabled (the default), then it also implements some of the `std::io` traits.
//!
//! ## FFI strings
//!
//! The string types in `std::ffi` are extremely useful for interfacing with C code, and `slibc`
//! uses them extensively (both internally and in public APIs). However, these string types have
//! no equivalents in `core` or `alloc`, which makes them impossible to use in `#![no_std]`
//! environments.
//!
//! As a result, when the `std` feature is not enabled, `slibc` uses its own versions of `CStr`,
//! `CString`, `OsStr`, and `OsString`. These structs are available under the [`ffi`] module (if
//! `std` *is* enabled, then the versions in `std::ffi` are re-exported there). They have
//! equivalents of nearly all the methods and trait implementations that are available on the
//! `std::ffi` versions; differences from those versions should be reported as bugs.
//!
//! ## String/path handling
//!
//! Most C functions that accept strings require them to be nul-terminated (one notable exception
//! is `sethostname(2)`). However, most Rust strings are not nul-terminated. As a result, before
//! passing paths to the underlying C functions, most Rust code must allocate memory for a buffer
//! to store the string plus the terminating nul.
//!
//! If either the `std` or `alloc` feature is enabled, `slibc` does this transparently. As a
//! result, strings that are not nul-terminated (like `str`, `String`, `OsStr`, and `OsString`,
//! though see [FFI strings](#ffi-strings) above) can be used, like this:
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! slibc::chdir(".").unwrap();
//! # }
//! ```
//!
//! However, note that this is wasteful, since it allocates memory just to perform a single
//! operation. If you want to avoid allocating memory (or you are in a `#![no_std]` crate without
//! an allocator), look into [the `cstr` crate](https://crates.io/crates/cstr):
//!
//! ```ignore
//! use cstr::cstr;
//! slibc::unistd::chdir(cstr!(".")).unwrap();
//! ```
//!
//! ## Compared to `nix`
//!
//! Some of the interfaces in `slibc` were designed after corresponding interfaces in `nix`, but
//! `slibc` does *not* slavishly mimic `nix`'s API.

#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

mod as_path;
mod err;
mod sys;
mod util;

pub use as_path::AsPath;
pub use err::{Error, Result};

mod internal_prelude {
    pub use core::mem::MaybeUninit;

    pub(crate) use super::util::IntParseBytes;
    pub(crate) use super::{sys, util, AsPath, Errno, Error, Result};

    pub use super::ffi::prelude::*;
    pub use super::ffi::{CStr, OsStr};
    pub use super::{BorrowedFd, FileDesc};

    pub use super::fcntl::OFlag;
    pub use super::string::memchr;

    #[cfg(feature = "alloc")]
    pub use super::ffi::{CString, OsString};

    pub use super::errno::{errno_get, errno_set};

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::{
        borrow::{Cow, ToOwned},
        boxed::Box,
        format,
        string::{String, ToString},
        vec::Vec,
    };
}

pub mod ffi;

mod borrowed_fd;
mod fdesc;

pub use borrowed_fd::*;
pub use fdesc::*;

mod errno;
mod fcntl;
mod ioctl;
mod limits;
mod mman;
mod pty;
mod resource;
mod sched;
mod signal;
mod stat;
mod stdio;
mod stdlib;
mod string;
mod time;
mod uio;
mod unistd;
mod utsname;
mod wait;

pub use errno::*;
pub use fcntl::*;
pub use ioctl::*;
pub use limits::*;
pub use mman::*;
pub use pty::*;
pub use resource::*;
pub use sched::*;
pub use signal::*;
pub use stat::*;
pub use stdio::*;
pub use stdlib::*;
pub use string::*;
pub use time::*;
pub use uio::*;
pub use unistd::*;
pub use utsname::*;
pub use wait::*;

mod dirent;
pub use dirent::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod grp;
        mod pwd;
        pub use grp::*;
        pub use pwd::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod signalfd;
        mod statx;
        mod sysinfo;
        pub use signalfd::*;
        pub use statx::*;
        pub use sysinfo::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    ))] {
        mod sysctl;
        pub use sysctl::*;
    }
}

/// A collection of functions that return `&'static CStr`s for various commonly used paths.
pub mod c_paths {
    use crate::ffi::CStr;

    /// Return an `&'static CStr` containing a single dot (`.`).
    #[inline]
    pub fn dot() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b".\0") }
    }

    /// Return an `&'static CStr` containing two dots (`..`).
    #[inline]
    pub fn dotdot() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"..\0") }
    }

    /// Return an `&'static CStr` containing a single slash (`/`).
    #[inline]
    pub fn slash() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"/\0") }
    }
}
