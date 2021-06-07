use crate::internal_prelude::*;

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

/// Create a new kqueue instance.
///
/// This is a simple wrapper that calls `kqueue(2)` and handles errors.
///
/// The returned kqueue file descriptor will not be inherited by children of a `fork()`.
///
/// On macOS, the kqueue file descriptor will also have its close-on-exec flag set. On the BSDs, it
/// will NOT have its close-on-exec flag set. See [`kqueue_cloexec()`] (and `kqueue1()` on NetBSD).
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[inline]
pub fn kqueue() -> Result<FileDesc> {
    unsafe { Error::unpack_fdesc(libc::kqueue()) }
}

/// Create a new kqueue instance, specifying flags on it.
///
/// The `O_CLOEXEC`, `O_NONBLOCK`, and `O_NOSIGPIPE` flags can be passed to set those attributes on
/// the returned kqueue file descriptor.
#[cfg_attr(docsrs, doc(cfg(target_os = "netbsd")))]
#[cfg(target_os = "netbsd")]
#[inline]
pub fn kqueue1(flags: crate::OFlag) -> Result<FileDesc> {
    unsafe { Error::unpack_fdesc(libc::kqueue1(flags.bits())) }
}

/// Create a new kqueue instance with its close-on-exec flag set.
///
/// On NetBSD, this calls `kqueue1(O_CLOEXEC)`. On macOS, it simply calls [`kqueue()`] (since that
/// sets the close-on-exec flag by default). On other platforms, it calls [`kqueue()`] and sets the
/// close-on-exec flag on the returned kqueue instance (which does NOT create a race condition
/// because the kqueue instance is not inherited by `fork()`ed children).
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[inline]
pub fn kqueue_cloexec() -> Result<FileDesc> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "netbsd")] {
            return kqueue1(crate::OFlag::O_CLOEXEC);
        } else if #[cfg(apple)] {
            return kqueue();
        } else {
            let kq = kqueue()?;
            kq.set_cloexec(true)?;
            return Ok(kq);
        }
    }
}

/// Register events with the queue and return pending events to the user.
///
/// This is a low-level wrapper around `kevent(2)`. No higher-level APIs are currently available
/// because kqueue is very complex.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[inline]
pub fn kevent_raw(
    kq: RawFd,
    changes: &[libc::kevent],
    events: &mut [libc::kevent],
    timeout: Option<&crate::TimeSpec>,
) -> Result<usize> {
    use core::convert::TryInto;

    #[cfg(not(target_os = "netbsd"))]
    if changes.len() > libc::c_int::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    let n = Error::unpack(unsafe {
        libc::kevent(
            kq,
            changes.as_ptr(),
            changes.len() as _,
            events.as_mut_ptr(),
            events.len().try_into().unwrap_or(libc::c_int::MAX as _),
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
        )
    })?;

    Ok(n as usize)
}

/// A wrapper around a kqueue instance.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[derive(Debug)]
pub struct Kqueue(FileDesc);

impl Kqueue {
    /// See [`kqueue()`].
    #[inline]
    pub fn new() -> Result<Self> {
        kqueue().map(Self)
    }

    /// See [`kqueue1()`].
    #[cfg_attr(docsrs, doc(cfg(target_os = "netbsd")))]
    #[cfg(target_os = "netbsd")]
    #[inline]
    pub fn new_flags(flags: crate::OFlag) -> Result<Self> {
        kqueue1(flags).map(Self)
    }

    /// See [`kqueue_cloexec()`].
    #[inline]
    pub fn new_cloexec() -> Result<Self> {
        kqueue_cloexec().map(Self)
    }

    /// See [`kevent_raw()`].
    #[inline]
    pub fn kevent_raw(
        &self,
        changes: &[libc::kevent],
        events: &mut [libc::kevent],
        timeout: Option<&crate::TimeSpec>,
    ) -> Result<usize> {
        kevent_raw(self.fd(), changes, events, timeout)
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `Kqueue` wrapper around the given kqueue file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid kqueue instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl AsRef<BorrowedFd> for Kqueue {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for Kqueue {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for Kqueue {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for Kqueue {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloexec_flag() {
        #[cfg(not(apple))]
        assert!(!Kqueue::new().unwrap().as_ref().get_cloexec().unwrap());

        assert!(Kqueue::new_cloexec()
            .unwrap()
            .as_ref()
            .get_cloexec()
            .unwrap());

        #[cfg(target_os = "netbsd")]
        {
            assert!(!Kqueue::new_flags(crate::OFlag::empty())
                .unwrap()
                .as_ref()
                .get_cloexec()
                .unwrap());
            assert!(!Kqueue::new_flags(crate::OFlag::O_CLOEXEC)
                .unwrap()
                .as_ref()
                .get_cloexec()
                .unwrap());
        }
    }
}
