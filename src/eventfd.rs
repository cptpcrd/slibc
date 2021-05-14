use crate::internal_prelude::*;

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct EventfdFlags: libc::c_int {
        /// Set the close-on-exec flag on the new file descriptor.
        const CLOEXEC = libc::EFD_CLOEXEC;
        /// Set the new file descriptor as non-blocking.
        const NONBLOCK = libc::EFD_NONBLOCK;
        /// Provide semaphore-like semantics for reads from the new file descriptor.
        ///
        /// See `eventfd(2)` for more information.
        const SEMAPHORE = libc::EFD_SEMAPHORE;
    }
}

/// Create a new event file descriptor.
///
/// `initval` specifies the initial value for the counter. `flags` specifies attributes used to
/// modify the behavior of the eventfd.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn eventfd(initval: u32, flags: EventfdFlags) -> Result<FileDesc> {
    unsafe { Error::unpack_fdesc(libc::eventfd(initval, flags.bits)) }
}

/// Read one 8-byte integer from the given event file descriptor.
///
/// See `eventfd(2)` for exact semantics.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn eventfd_read(fd: RawFd) -> Result<u64> {
    let mut buf = [0u8; 8];
    if crate::read(fd, &mut buf)? != 8 {
        return Err(Error::from_code(libc::EINVAL));
    }
    Ok(u64::from_ne_bytes(buf))
}

/// Write one 8-byte integer into the given event file descriptor.
///
/// See `eventfd(2)` for exact semantics.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn eventfd_write(fd: RawFd, value: u64) -> Result<()> {
    if crate::write(fd, &value.to_ne_bytes())? != 8 {
        return Err(Error::from_code(libc::EINVAL));
    }
    Ok(())
}

/// A wrapper around an event file descriptor.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Debug)]
pub struct EventFd(FileDesc);

impl EventFd {
    /// Create a new event file descriptor.
    ///
    /// See [`eventfd()`].
    #[inline]
    pub fn new(initval: u32, flags: EventfdFlags) -> Result<Self> {
        eventfd(initval, flags).map(Self)
    }

    /// Read one 8-byte integer from the event file descriptor.
    ///
    /// See `eventfd(2)` for exact semantics.
    #[inline]
    pub fn read(&self) -> Result<u64> {
        eventfd_read(self.fd())
    }

    /// Write one 8-byte integer into the event file descriptor.
    ///
    /// See `eventfd(2)` for exact semantics.
    #[inline]
    pub fn write(&self, value: u64) -> Result<()> {
        eventfd_write(self.fd(), value)
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `EventFd` wrapper around the given eventfd file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid eventfd instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl AsRef<BorrowedFd> for EventFd {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for EventFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for EventFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for EventFd {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eventfd_default() {
        let evfd = EventFd::new(0, EventfdFlags::CLOEXEC).unwrap();
        assert!(evfd.as_ref().get_cloexec().unwrap());
        assert!(!evfd.as_ref().get_nonblocking().unwrap());

        evfd.write(1).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        evfd.write(u64::MAX - 1).unwrap();
        assert_eq!(evfd.read().unwrap(), u64::MAX - 1);

        evfd.as_ref().set_nonblocking(true).unwrap();

        evfd.write(1).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        // read()ing when the counter is 0 fails with EAGAIN if the file descriptor is non-blocking
        assert_eq!(evfd.read().unwrap_err(), Errno::EAGAIN);
    }

    #[test]
    fn test_eventfd_semaphore() {
        let evfd = EventFd::new(0, EventfdFlags::NONBLOCK | EventfdFlags::SEMAPHORE).unwrap();
        assert!(!evfd.as_ref().get_cloexec().unwrap());
        assert!(evfd.as_ref().get_nonblocking().unwrap());

        evfd.write(1).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        // Semaphores slowly decrement as they are read()
        evfd.write(2).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        assert_eq!(evfd.read().unwrap(), 1);
        // read()ing when the counter is 0 fails with EAGAIN if the file descriptor is non-blocking
        assert_eq!(evfd.read().unwrap_err(), Errno::EAGAIN);

        evfd.as_ref().set_nonblocking(false).unwrap();

        evfd.write(1).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        // Semaphores slowly decrement as they are read()
        evfd.write(2).unwrap();
        assert_eq!(evfd.read().unwrap(), 1);
        assert_eq!(evfd.read().unwrap(), 1);
    }
}
