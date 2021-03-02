use core::fmt;

use crate::internal_prelude::*;

pub type Result<T> = core::result::Result<T, Error>;

/// Represents an OS error encountered when performing an operation.
#[derive(Clone, Eq, PartialEq)]
pub struct Error(i32);

impl Error {
    /// If `res` is -1, return the last OS error. Otherwise return `Ok(res)`.
    #[inline]
    pub(crate) fn unpack(res: i32) -> Result<i32> {
        if res == -1 {
            Err(Self::last())
        } else {
            Ok(res)
        }
    }

    /// If `res` is -1, return the last OS error. Otherwise return `Ok(res)`.
    #[inline]
    pub(crate) fn unpack_size(res: isize) -> Result<usize> {
        if res == -1 {
            Err(Self::last())
        } else {
            Ok(res as usize)
        }
    }

    /// If `res` is non-zero, return the last OS error. Otherwise return `Ok(())`.
    #[inline]
    pub(crate) fn unpack_nz(res: i32) -> Result<()> {
        if res != 0 {
            Err(Self::last())
        } else {
            Ok(())
        }
    }

    /// If `res` is non-zero, interpret it as an `errno` value and return the corresponding OS
    /// error. Otherwise return `Ok(())`.
    #[inline]
    pub(crate) fn unpack_eno(res: i32) -> Result<()> {
        if res != 0 {
            Err(Self(res))
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) const fn mid_nul() -> Self {
        Self(libc::EINVAL)
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) const fn no_nul() -> Self {
        Self(libc::EINVAL)
    }

    /// Get the last OS error that occured (i.e. the current `errno` value).
    #[inline]
    pub fn last() -> Self {
        Self(errno_get())
    }

    /// Construct an `Error` from an `errno` code.
    #[inline]
    pub fn from_code(eno: i32) -> Self {
        Self(eno)
    }

    /// Get the `errno` code represented by this `Error` object.
    #[inline]
    pub fn code(&self) -> i32 {
        self.0
    }

    pub(crate) fn strerror(&self) -> &'static str {
        // If the given error number is invalid (negative, 0, or out of range), most OSes allocate
        // memory and prints "Unknown error %d". This means it can't be 'static.
        //
        // (musl is the only libc I know of that doesn't do this.)
        //
        // So if the error code is <= 0 (or if the message returned by strerror() starts with
        // "Unknown error"), we return "Unknown error" instead.

        static UNKNOWN_ERROR: &str = "Unknown error";

        use core::cmp::Ordering;
        match self.0.cmp(&0) {
            // eno < 0 -> invalid
            Ordering::Less => return UNKNOWN_ERROR,
            // eno == 0 -> success
            Ordering::Equal => return "Success",

            _ => (),
        }

        let ptr = unsafe { libc::strerror(self.0) };
        debug_assert!(!ptr.is_null());

        let msg = core::str::from_utf8(unsafe { util::bytes_from_ptr(ptr) }).unwrap();

        #[cfg(not(all(target_os = "linux", target_env = "musl")))]
        if msg.starts_with(UNKNOWN_ERROR) {
            return UNKNOWN_ERROR;
        }

        msg
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.strerror())?;
        write!(f, " (code {})", self.0)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Error")
            .field("code", &self.0)
            .field("message", &self.strerror())
            .finish()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    #[inline]
    fn from(e: Error) -> Self {
        Self::from_raw_os_error(e.0)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nix")))]
#[cfg(feature = "nix")]
impl From<Error> for nix::Error {
    #[inline]
    fn from(e: Error) -> Self {
        Self::Sys(nix::errno::Errno::from_i32(e.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        assert_eq!(Error::from_code(libc::EPERM).code(), libc::EPERM);
        assert_eq!(Error::from_code(libc::ENOENT).code(), libc::ENOENT);
    }

    #[test]
    fn test_last() {
        errno_set(libc::EPERM);
        assert_eq!(Error::last().code(), libc::EPERM);

        errno_set(libc::ENOENT);
        assert_eq!(Error::last().code(), libc::ENOENT);
    }

    #[test]
    fn test_unpack() {
        errno_set(libc::ENOENT);

        assert_eq!(Error::unpack(0), Ok(0));
        assert_eq!(Error::unpack_size(0), Ok(0));
        assert_eq!(Error::unpack_nz(0), Ok(()));
        assert_eq!(Error::unpack(-1), Err(Error::from_code(libc::ENOENT)));
        assert_eq!(Error::unpack_size(-1), Err(Error::from_code(libc::ENOENT)));
        assert_eq!(Error::unpack_nz(-1), Err(Error::from_code(libc::ENOENT)));
    }

    #[test]
    fn test_strerror() {
        assert_eq!(Error::from_code(libc::EISDIR).strerror(), "Is a directory");

        assert_eq!(Error::from_code(-1).strerror(), "Unknown error");
        assert_eq!(Error::from_code(0).strerror(), "Success");

        #[cfg(any(target_env = "", target_env = "gnu"))]
        assert_eq!(Error::from_code(8192).strerror(), "Unknown error");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_display() {
        assert_eq!(
            Error::from_code(libc::EISDIR).to_string(),
            format!("Is a directory (code {})", libc::EISDIR)
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_debug() {
        assert_eq!(
            format!("{:?}", Error::from_code(libc::EISDIR)),
            format!(
                "Error {{ code: {}, message: \"Is a directory\" }}",
                libc::EISDIR
            )
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_from_error() {
        assert_eq!(
            std::io::Error::from(Error::from_code(libc::ENOENT)).raw_os_error(),
            Some(libc::ENOENT)
        );
    }
}
