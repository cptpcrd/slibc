use crate::internal_prelude::*;

use crate::Signal;

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    pub struct PidFdOpenFlags: libc::c_uint {
    }
}

bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    pub struct PidFdGetfdFlags: libc::c_uint {
    }
}

bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    pub struct PidFdSignalFlags: libc::c_uint {
    }
}

/// Obtain a file descriptor referring to the specified process.
///
/// This system call was added in Linux 5.3. For more information, see `pidfd_open(2)`.
///
/// The returned PID file descriptor will have its close-on-exec flag set.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn pidfd_open(pid: libc::pid_t, flags: PidFdOpenFlags) -> Result<FileDesc> {
    unsafe { Error::unpack_fdesc(libc::syscall(libc::SYS_pidfd_open, pid, flags.bits()) as i32) }
}

/// Obtain a duplicate of another process's file descriptor.
///
/// This system call was added in Linux 5.6.
///
/// `pidfd` is a PID file descriptor (opened with e.g. [`pidfd_open()`]) referring to the process,
/// and `targetfd` specifies which of that process's file descriptors should be duplicated. The
/// returned file descriptor will have its close-on-exec flag set.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn pidfd_getfd(pidfd: RawFd, targetfd: RawFd, flags: PidFdGetfdFlags) -> Result<FileDesc> {
    unsafe {
        Error::unpack_fdesc(
            libc::syscall(libc::SYS_pidfd_getfd, pidfd, targetfd, flags.bits()) as i32,
        )
    }
}

/// Send a signal to a process specified by a PID file descriptor.
///
/// This calls the `pidfd_send_signal()` system call with a null `info` argument (issues with the
/// Rust definitions of `siginfo_t` currently make it difficult to initialize a `siginfo_t`
/// structure properly).
///
/// The `pidfd_send_signal()` system call was added in Linux 5.1. See `pidfd_send_signal(2)` for
/// more information.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn pidfd_send_signal_simple<S: Into<Option<Signal>>>(
    pidfd: RawFd,
    sig: S,
    flags: PidFdSignalFlags,
) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            pidfd,
            sig.into().map_or(0, |s| s.as_i32()),
            core::ptr::null_mut::<libc::siginfo_t>(),
            flags.bits(),
        ) as i32
    })
}

/// A wrapper around a PID file descriptor opened with `pidfd_open(2)`.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[derive(Debug)]
pub struct PidFd(FileDesc);

impl PidFd {
    /// See [`pidfd_open()`].
    #[inline]
    pub fn open(pid: libc::pid_t, flags: PidFdOpenFlags) -> Result<Self> {
        pidfd_open(pid, flags).map(Self)
    }

    /// See [`pidfd_getfd()`].
    #[inline]
    pub fn getfd(&self, targetfd: RawFd, flags: PidFdGetfdFlags) -> Result<FileDesc> {
        pidfd_getfd(self.fd(), targetfd, flags)
    }

    /// See [`pidfd_send_signal_simple()`].
    #[inline]
    pub fn send_signal_simple<S: Into<Option<Signal>>>(
        &self,
        sig: S,
        flags: PidFdSignalFlags,
    ) -> Result<()> {
        pidfd_send_signal_simple(self.fd(), sig, flags)
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `PidFd` wrapper around the given pidfd file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid pidfd instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl From<PidFd> for FileDesc {
    #[inline]
    fn from(s: PidFd) -> Self {
        s.0
    }
}

impl AsRef<BorrowedFd> for PidFd {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for PidFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for PidFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for PidFd {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pidfd_getfd() {
        if pidfd_getfd(-1, -1, PidFdGetfdFlags::empty()).unwrap_err() != Errno::EBADF {
            return;
        }

        let pfd = PidFd::open(crate::getpid(), PidFdOpenFlags::empty()).unwrap();
        assert!(pfd.as_ref().get_cloexec().unwrap());
        let r1 = crate::pipe().unwrap().0;
        let r2 = pfd.getfd(r1.fd(), PidFdGetfdFlags::empty()).unwrap();
        assert!(r2.get_cloexec().unwrap());

        let r1_stat = r1.stat().unwrap();
        let r2_stat = r2.stat().unwrap();
        assert_eq!(r1_stat.dev(), r2_stat.dev());
        assert_eq!(r1_stat.ino(), r2_stat.ino());
    }
}
