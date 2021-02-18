use core::fmt;

use crate::internal_prelude::*;

/// Get the current thread's `errno` value.
#[inline]
pub fn get_errno() -> libc::c_int {
    unsafe { *util::errno_ptr() }
}

/// Set the current thread's `errno` value.
#[inline]
pub fn set_errno(eno: libc::c_int) {
    unsafe {
        *util::errno_ptr() = eno;
    }
}

macro_rules! define_errno {
    ($(#[cfg($cfg:meta)] $($name:ident,)*)*) => {
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
        }

        #[allow(dead_code)]
        static ERRNOS: &[Errno] = &[
            $($(
                #[cfg($cfg)]
                Errno::$name,
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
                    _ => Self::Unknown,
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
    EMULTIHOP,
    ENOLINK,
    EPROTO,
    ENOMEM,
    ENODEV,

    #[cfg(bsd)]
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

    #[cfg(target_os = "freebsd")]
    EDOOFUS,
    ENOTCAPABLE,
    ECAPMODE,

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
    #[inline]
    pub fn from_code(eno: i32) -> Self {
        Self::from_code_match(eno)
    }

    #[inline]
    pub fn desc(self) -> &'static str {
        if self == Self::Unknown {
            "Unknown error"
        } else {
            Error::from(self).strerror()
        }
    }

    #[inline]
    pub fn last() -> Self {
        Self::from_code(get_errno())
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
        nix::errno::Errno::from_code(e as i32)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getset_errno() {
        set_errno(0);
        assert_eq!(get_errno(), 0);
        set_errno(0);
        assert_eq!(get_errno(), 0);

        set_errno(libc::EINVAL);
        assert_eq!(get_errno(), libc::EINVAL);
        set_errno(libc::EINVAL);
        assert_eq!(get_errno(), libc::EINVAL);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_errno_thread() {
        set_errno(0);
        assert_eq!(get_errno(), 0);

        std::thread::spawn(|| {
            set_errno(libc::EINVAL);
            assert_eq!(get_errno(), libc::EINVAL);
        })
        .join()
        .unwrap();

        assert_eq!(get_errno(), 0);
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
        set_errno(0);
        assert_eq!(Errno::last(), Errno::Unknown);
        set_errno(libc::ENOENT);
        assert_eq!(Errno::last(), Errno::ENOENT);
        set_errno(-1);
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
        for &eno in ERRNOS {
            assert_eq!(eno, Error::from(eno));
            assert_eq!(Error::from(eno), eno);
        }

        assert_ne!(Errno::EPERM, Error::from(Errno::EACCES));
        assert_ne!(Error::from(Errno::EACCES), Errno::EPERM);
        assert_ne!(Error::from_code(0), Errno::Unknown);
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
