use core::fmt;

use crate::internal_prelude::*;

/// Get the current thread's `errno` value.
#[inline]
pub fn errno_get() -> libc::c_int {
    unsafe { *util::errno_ptr() }
}

/// Set the current thread's `errno` value.
#[inline]
pub fn errno_set(eno: libc::c_int) {
    unsafe {
        *util::errno_ptr() = eno;
    }
}

macro_rules! define_errno {
    ($(#[cfg($cfg:meta)] $($name:ident,)+ $(@$name2:ident = $val2:expr,)*)*) => {
        /// Represents an `errno` value.
        ///
        /// # `Errno` vs. `Error`
        ///
        /// `Error` wraps an error code without performing any validation of the value; you could
        /// have an `Error` storing the code -1. `Errno`, however, translates unknown values into
        /// `Errno::Unknown`.
        ///
        /// # Interaction with `Error`
        ///
        /// - `Errno`s can be converted `.into()` `Error`s (`Unknown` is translated to 0).
        /// - `Errno`s can be compared to `Error`s. They will compare as equal if a) they store the
        ///   same error code and b) the `Errno` is NOT `Errno::Unknown`. For example:
        ///
        /// ```
        /// use slibc::{Errno, Error};
        /// assert_eq!(Error::from_code(Errno::EPERM as i32), Errno::EPERM);
        /// assert_eq!(Errno::EPERM, Error::from_code(Errno::EPERM as i32));
        /// assert_ne!(Errno::Unknown, Error::from_code(0));
        /// ```
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[repr(i32)]
        #[non_exhaustive]
        pub enum Errno {
            Unknown = 0,
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                $name = libc::$name,
            )*)*
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                $name2 = $val2,
            )*)*
        }

        static ERRNOS: &[Errno] = &[
            $($(
                #[cfg($cfg)]
                Errno::$name,
            )*)*
            $($(
                #[cfg($cfg)]
                Errno::$name2,
            )*)*
        ];

        impl Errno {
            #[inline]
            fn from_code_match(eno: i32) -> Self {
                match eno {
                    $($(
                        #[cfg($cfg)]
                        libc::$name => Self::$name,
                    )*)*
                    $($(
                        #[cfg($cfg)]
                        $val2 => Self::$name2,
                    )*)*
                    _ => Self::Unknown,
                }
            }

            #[inline]
            fn name_match(self) -> &'static str {
                match self {
                    $($(
                        #[cfg($cfg)]
                        Self::$name => stringify!($name),
                    )*)*
                    $($(
                        #[cfg($cfg)]
                        Self::$name2 => stringify!($name2),
                    )*)*
                    Self::Unknown => "Unknown",
                }
            }
        }
    }
}

define_errno! {
    #[cfg(all())]
    EPERM,
    ENOENT,
    EEXIST,
    EISDIR,
    ENOTDIR,
    ESRCH,
    EINTR,
    EIO,
    ENXIO,
    E2BIG,
    ENOEXEC,
    EACCES,
    EAGAIN,
    EALREADY,
    EBADF,
    EBUSY,
    ECHILD,
    EDEADLK,
    EFAULT,
    EFBIG,
    EINPROGRESS,
    EINVAL,
    ENOTBLK,
    ENFILE,
    EMFILE,
    ENOTTY,
    EXDEV,
    ETXTBSY,
    ENOSPC,
    ESPIPE,
    EROFS,
    EMLINK,
    EPIPE,
    EDOM,
    ERANGE,
    ENOTSOCK,
    EDESTADDRREQ,
    EMSGSIZE,
    EPROTOTYPE,
    ENOPROTOOPT,
    EPROTONOSUPPORT,
    ESOCKTNOSUPPORT,
    EOPNOTSUPP,
    EPFNOSUPPORT,
    EAFNOSUPPORT,
    EADDRINUSE,
    EADDRNOTAVAIL,
    ENETDOWN,
    ENETUNREACH,
    ENETRESET,
    ECONNABORTED,
    ECONNRESET,
    ENOBUFS,
    EISCONN,
    ENOTCONN,
    ESHUTDOWN,
    ETOOMANYREFS,
    ETIMEDOUT,
    ECONNREFUSED,
    ELOOP,
    ENAMETOOLONG,
    EHOSTDOWN,
    EHOSTUNREACH,
    ENOTEMPTY,
    EUSERS,
    EDQUOT,
    ESTALE,
    EREMOTE,
    ENOLCK,
    ENOSYS,
    EIDRM,
    ENOMSG,
    EOVERFLOW,
    ECANCELED,
    EILSEQ,
    EBADMSG,
    EPROTO,
    ENOMEM,
    ENODEV,

    #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    ))]
    ENOATTR,
    ENEEDAUTH,
    EAUTH,
    EFTYPE,
    EPROGUNAVAIL,
    EPROGMISMATCH,
    EPROCUNAVAIL,
    ERPCMISMATCH,
    EBADRPC,
    EPROCLIM,

    #[cfg(target_os = "linux")]
    EBADE,
    EBADFD,
    EBADR,
    EBADRQC,
    EBADSLT,
    ECHRNG,
    ECOMM,
    EHWPOISON,
    EISNAM,
    EKEYEXPIRED,
    EKEYREJECTED,
    EKEYREVOKED,
    ENOKEY,
    EREMOTEIO,
    EL2HLT,
    EL2NSYNC,
    EL3HLT,
    EL3RST,
    ELNRNG,
    EUNATCH,
    ENOCSI,
    EXFULL,
    ENOANO,
    EBFONT,
    ENOTNAM,
    ERFKILL,
    ENAVAIL,
    EUCLEAN,
    ESTRPIPE,
    ELIBEXEC,
    ELIBSCN,
    ELIBMAX,
    ELIBBAD,
    ELIBACC,
    EDOTDOT,
    ERESTART,
    ENOTUNIQ,
    EADV,
    ESRMNT,
    ENOPKG,
    ENONET,
    EREMCHG,

    #[cfg(not(target_os = "openbsd"))]
    EMULTIHOP,
    ENOLINK,

    #[cfg(target_os = "freebsd")]
    EDOOFUS,
    ENOTCAPABLE,
    ECAPMODE,
    @EINTEGRITY = 97,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    ENOPOLICY,
    EQFULL,
    EBADMACHO,
    ESHLIBVERS,
    EBADARCH,
    EBADEXEC,
    EDEVERR,
    EPWROFF,

    #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "macos", target_os = "ios"))]
    ETIME,
    ENODATA,
    ENOSR,
    ENOSTR,

    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    ENOMEDIUM,
    EMEDIUMTYPE,

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    ))]
    ENOTSUP,

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "macos",
        target_os = "ios",
    ))]
    ENOTRECOVERABLE,
    EOWNERDEAD,
}

impl Errno {
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
    ))]
    pub const ENOTSUP: Self = Self::EOPNOTSUPP;

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    ))]
    pub const EWOULDBLOCK: Self = Self::EAGAIN;
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    pub const EDEADLOCK: Self = Self::EDEADLK;

    /// Get the `Errno` value corresponding to the given error code.
    ///
    /// Note: All values that are not in this enum will be translated to `Errno::Unknown`.
    #[inline]
    pub fn from_code(eno: i32) -> Self {
        Self::from_code_match(eno)
    }

    /// Get the "name" of the given error number (e.g. `ENOENT`).
    pub fn name(self) -> &'static str {
        self.name_match()
    }

    /// Get the "description" (i.e. `strerror()`) for the given error number.
    #[inline]
    pub fn desc(self) -> &'static str {
        if self == Self::Unknown {
            "Unknown error"
        } else {
            crate::strerror::strerror(self as i32)
        }
    }

    /// Get the last `errno` value.
    ///
    /// This is equivalent to `Errno::from_code(errno_get())`.
    #[inline]
    pub fn last() -> Self {
        Self::from_code(errno_get())
    }

    /// Create an iterator over all of this system's `Errno` values.
    ///
    /// The iterator yields the values in an arbitrary order. It does NOT yield `Unknown`.
    #[inline]
    pub fn iter() -> ErrnoIter {
        ErrnoIter(ERRNOS.iter().copied())
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self, self.desc())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for Errno {}

impl PartialEq<Error> for Errno {
    #[inline]
    fn eq(&self, other: &Error) -> bool {
        *self != Errno::Unknown && *self as i32 == other.code()
    }
}

impl PartialEq<Errno> for Error {
    #[inline]
    fn eq(&self, other: &Errno) -> bool {
        *other != Errno::Unknown && self.code() == *other as i32
    }
}

impl From<Errno> for Error {
    #[inline]
    fn from(e: Errno) -> Self {
        Self::from_code(e as i32)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<Errno> for std::io::Error {
    #[inline]
    fn from(e: Errno) -> Self {
        Self::from_raw_os_error(e as i32)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nix")))]
#[cfg(feature = "nix")]
impl From<Errno> for nix::errno::Errno {
    #[inline]
    fn from(e: Errno) -> Self {
        nix::errno::Errno::from_i32(e as i32)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nix")))]
#[cfg(feature = "nix")]
impl From<Errno> for nix::Error {
    #[inline]
    fn from(e: Errno) -> Self {
        Self::Sys(e.into())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nix")))]
#[cfg(feature = "nix")]
impl From<nix::errno::Errno> for Errno {
    #[inline]
    fn from(e: nix::errno::Errno) -> Self {
        Self::from_code(e as i32)
    }
}

/// An iterator over all of this system's [`Errno`] values.
///
/// This is created by [`Errno::iter()`].
#[derive(Clone, Debug)]
pub struct ErrnoIter(core::iter::Copied<core::slice::Iter<'static, Errno>>);

impl Iterator for ErrnoIter {
    type Item = Errno;

    #[inline]
    fn next(&mut self) -> Option<Errno> {
        self.0.next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Errno> {
        self.0.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Errno> {
        self.0.last()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl ExactSizeIterator for ErrnoIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl core::iter::FusedIterator for ErrnoIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geterrno_set() {
        errno_set(0);
        assert_eq!(errno_get(), 0);
        errno_set(0);
        assert_eq!(errno_get(), 0);

        errno_set(libc::EINVAL);
        assert_eq!(errno_get(), libc::EINVAL);
        errno_set(libc::EINVAL);
        assert_eq!(errno_get(), libc::EINVAL);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_errno_thread() {
        errno_set(0);
        assert_eq!(errno_get(), 0);

        std::thread::spawn(|| {
            errno_set(libc::EINVAL);
            assert_eq!(errno_get(), libc::EINVAL);
        })
        .join()
        .unwrap();

        assert_eq!(errno_get(), 0);
    }

    #[test]
    fn test_errno_from_code() {
        macro_rules! check {
            ($eno:expr) => {
                assert_eq!(Errno::from_code($eno), Errno::from_code_match($eno));
            };
        }

        for &eno in ERRNOS {
            check!(eno as i32);
        }

        for eno in -1000..6000 {
            check!(eno);
        }
    }

    #[test]
    fn test_errno_desc() {
        assert_eq!(Errno::Unknown.desc(), "Unknown error");
        assert_eq!(Errno::ENOENT.desc(), "No such file or directory");
        assert_eq!(Errno::ENOTDIR.desc(), "Not a directory");
    }

    #[test]
    fn test_errno_last() {
        errno_set(0);
        assert_eq!(Errno::last(), Errno::Unknown);
        errno_set(libc::ENOENT);
        assert_eq!(Errno::last(), Errno::ENOENT);
        errno_set(-1);
        assert_eq!(Errno::last(), Errno::Unknown);
    }

    #[test]
    fn test_errno_missing() {
        // For every error number in 1-4096, make sure that if strerror() recognizes it, then
        // Errno::from_code() does too.

        for eno in 1..4096 {
            let msg = Error::from_code(eno).strerror();
            let errno = Errno::from_code(eno);

            if !matches!(msg, "Unknown error" | "No error information") {
                assert_ne!(errno, Errno::Unknown, "{}", eno);
                assert_eq!(errno.desc(), msg, "{}", eno);
            }
        }
    }

    #[test]
    fn test_errno_error_eq() {
        for eno in Errno::iter() {
            assert_eq!(eno, Error::from(eno));
            assert_eq!(Error::from(eno), eno);
        }

        assert_ne!(Errno::EPERM, Error::from(Errno::EACCES));
        assert_ne!(Error::from(Errno::EACCES), Errno::EPERM);
        assert_ne!(Error::from_code(0), Errno::Unknown);
    }

    #[test]
    fn test_errno_alias() {
        // Make sure that these 2 (which are `const`s on most platforms) can be used in a `match`.
        assert!(matches!(Errno::ENOTSUP, Errno::ENOTSUP));
        assert!(matches!(Errno::EWOULDBLOCK, Errno::EWOULDBLOCK));
        // Same here on Linux
        #[cfg(target_os = "linux")]
        assert!(matches!(Errno::EDEADLOCK, Errno::EDEADLOCK));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_errno_name() {
        for eno in Errno::iter() {
            assert_ne!(eno, Errno::Unknown);
            assert_eq!(format!("{:?}", eno), eno.name());
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_errno_display() {
        assert_eq!(format!("{}", Errno::Unknown), "Unknown: Unknown error");
        assert_eq!(
            format!("{}", Errno::ENOENT),
            "ENOENT: No such file or directory"
        );
        assert_eq!(format!("{}", Errno::ENOTDIR), "ENOTDIR: Not a directory");
    }

    #[test]
    fn test_errno_into_error() {
        assert_eq!(Error::from(Errno::ENOENT).code(), libc::ENOENT);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_errno_into_io_error() {
        assert_eq!(
            std::io::Error::from(Errno::ENOENT).raw_os_error(),
            Some(libc::ENOENT)
        );
    }

    #[cfg(feature = "nix")]
    #[test]
    fn test_errno_nix() {
        assert_eq!(
            nix::errno::Errno::from(Errno::ENOENT),
            nix::errno::Errno::ENOENT
        );
        assert_eq!(
            nix::errno::Errno::from(Errno::Unknown),
            nix::errno::Errno::UnknownErrno
        );

        assert_eq!(
            nix::Error::from(Errno::ENOENT),
            nix::Error::Sys(nix::errno::Errno::ENOENT)
        );
        assert_eq!(
            nix::Error::from(Errno::Unknown),
            nix::Error::Sys(nix::errno::Errno::UnknownErrno)
        );

        assert_eq!(Errno::from(nix::errno::Errno::ENOENT), Errno::ENOENT);
        assert_eq!(Errno::from(nix::errno::Errno::UnknownErrno), Errno::Unknown);
    }
}
