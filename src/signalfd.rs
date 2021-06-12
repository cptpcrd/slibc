use crate::internal_prelude::*;
use crate::{SigSet, Signal};

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

bitflags::bitflags! {
    /// Flags to [`signalfd()`].
    ///
    /// See `signalfd(2)` for more information.
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct SigFdFlags: libc::c_int {
        const NONBLOCK = libc::SFD_NONBLOCK;
        const CLOEXEC = libc::SFD_CLOEXEC;
    }
}

/// Create or modify a signal file descriptor.
///
/// For a high-level interface, see [`SignalFd`].
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn signalfd<F: Into<Option<RawFd>>>(fd: F, mask: &SigSet, flags: SigFdFlags) -> Result<RawFd> {
    Error::unpack(unsafe { libc::signalfd(fd.into().unwrap_or(-1), mask.as_ref(), flags.bits()) })
}

/// A wrapper around a signal file descriptor.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Debug)]
pub struct SignalFd(FileDesc);

impl SignalFd {
    /// Create a new signal file descriptor.
    ///
    /// `mask` specifies the signals that should be monitored, and `flags` specifies creation
    /// flags.
    #[inline]
    pub fn new(mask: &SigSet, flags: SigFdFlags) -> Result<Self> {
        let fd = signalfd(None, mask, flags)?;
        Ok(unsafe { Self::from_fd(fd) })
    }

    /// Replace this signal file descriptor's mask.
    #[inline]
    pub fn set_mask(&self, mask: &SigSet) -> Result<()> {
        signalfd(self.0.fd(), mask, SigFdFlags::empty())?;
        Ok(())
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `SignalFd` wrapper around the given signalfd file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid signalfd instance, and it must not be in
    /// use by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }

    /// Read one or more [`SigFdSigInfo`] structures from this file descriptor's queue.
    ///
    /// This function will return the number of structures that were read.
    #[inline]
    pub fn read_siginfos(&self, buf: &mut [SigFdSigInfo]) -> Result<usize> {
        let n = self.0.read(unsafe {
            core::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut u8,
                buf.len() * core::mem::size_of::<SigFdSigInfo>(),
            )
        })?;

        Ok(n / core::mem::size_of::<SigFdSigInfo>())
    }
}

impl From<SignalFd> for FileDesc {
    #[inline]
    fn from(s: SignalFd) -> Self {
        s.0
    }
}

impl AsRef<BorrowedFd> for SignalFd {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for SignalFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for SignalFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for SignalFd {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct SigFdSigInfo(libc::signalfd_siginfo);

macro_rules! ssi_attrs {
    ($($(#[doc = $doc:literal])* $name:ident -> $type:ty,)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> $type {
                self.0.$name as _
            }
        )*
    }
}

impl SigFdSigInfo {
    /// Return a new `SigFdSigInfo` struct with all fields zeroed.
    #[inline]
    pub fn zeroed() -> Self {
        // SAFETY: signalfd_siginfo is entirely integer fields; zeroing it out is valid
        // initialization
        Self(unsafe { core::mem::zeroed() })
    }

    ssi_attrs! {
        ssi_signo -> i32,
        ssi_code -> i32,
        ssi_pid -> libc::pid_t,
        ssi_uid -> u32,
        ssi_fd -> i32,
        ssi_tid -> u32,
        ssi_band -> u32,
        ssi_overrun -> u32,
        ssi_trapno -> u32,
        ssi_status -> i32,
        ssi_int -> i32,
        ssi_ptr -> u64,
        ssi_utime -> u64,
        ssi_stime -> u64,
        ssi_addr -> u64,
        ssi_addr_lsb -> u16,
    }

    #[inline]
    pub fn signal(&self) -> Option<Signal> {
        Signal::from_i32(self.ssi_signo())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags() {
        let sfd = SignalFd::new(&SigSet::empty(), SigFdFlags::empty()).unwrap();
        assert!(!sfd.as_ref().get_cloexec().unwrap());
        assert!(!sfd.as_ref().get_nonblocking().unwrap());

        let sfd = SignalFd::new(&SigSet::empty(), SigFdFlags::CLOEXEC).unwrap();
        assert!(sfd.as_ref().get_cloexec().unwrap());
        assert!(!sfd.as_ref().get_nonblocking().unwrap());

        let sfd = SignalFd::new(&SigSet::empty(), SigFdFlags::NONBLOCK).unwrap();
        assert!(!sfd.as_ref().get_cloexec().unwrap());
        assert!(sfd.as_ref().get_nonblocking().unwrap());

        let sfd =
            SignalFd::new(&SigSet::empty(), SigFdFlags::CLOEXEC | SigFdFlags::NONBLOCK).unwrap();
        assert!(sfd.as_ref().get_cloexec().unwrap());
        assert!(sfd.as_ref().get_nonblocking().unwrap());
    }
}
