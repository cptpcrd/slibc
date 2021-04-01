use core::convert::TryInto;

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

use crate::internal_prelude::*;

bitflags::bitflags! {
    /// Flags for [`epoll_create1()`] or [`Epoll::new()`].
    pub struct EpollFlags: libc::c_int {
        /// Set the close-on-exec flag on the new file descriptor.
        const CLOEXEC = libc::EPOLL_CLOEXEC;
    }
}

bitflags::bitflags! {
    pub struct EpollEvents: u32 {
        const IN = libc::EPOLLIN as u32;
        const OUT = libc::EPOLLOUT as u32;
        const RDHUP = libc::EPOLLRDHUP as u32;
        const PRI = libc::EPOLLPRI as u32;
        const ERR = libc::EPOLLERR as u32;
        const HUP = libc::EPOLLHUP as u32;
        const ET = libc::EPOLLET as u32;
        const ONESHOT = libc::EPOLLONESHOT as u32;
        const WAKEUP = libc::EPOLLWAKEUP as u32;
        const EXCLUSIVE = 0x10000000;
    }
}

/// An operation to be performed by [`epoll_ctl()`] or [`Epoll::ctl()`].
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
#[allow(clippy::upper_case_acronyms)]
pub enum EpollCtlOp {
    ADD = libc::EPOLL_CTL_ADD,
    MOD = libc::EPOLL_CTL_MOD,
    DEL = libc::EPOLL_CTL_DEL,
}

/// An event returned by an epoll file descriptor.
///
/// See [`epoll_wait()`].
#[repr(transparent)]
pub struct EpollEvent(libc::epoll_event);

impl EpollEvent {
    #[inline]
    pub fn new(events: EpollEvents, data: u64) -> Self {
        Self(libc::epoll_event {
            events: events.bits(),
            u64: data,
        })
    }

    /// The events that occurred on the file descriptor that triggered this event.
    #[inline]
    pub fn events(&self) -> EpollEvents {
        EpollEvents::from_bits_truncate(self.0.events)
    }

    /// The "data" associated with this entry in the epoll instance's interest list.
    ///
    /// This is commonly used to store the file descriptor number.
    #[inline]
    pub fn data(&self) -> u64 {
        self.0.u64
    }
}

/// Create a new epoll instance and return a file descriptor referring to it.
#[inline]
pub fn epoll_create1(flags: EpollFlags) -> Result<FileDesc> {
    let fd = Error::unpack(unsafe { libc::epoll_create1(flags.bits()) })?;

    Ok(unsafe { FileDesc::new(fd) })
}

/// Add, modify, or delete an entry in the interest list of the epoll instance referred to by
/// `epfd`.
#[inline]
pub fn epoll_ctl(epfd: RawFd, op: EpollCtlOp, fd: RawFd, event: &mut EpollEvent) -> Result<()> {
    Error::unpack_nz(unsafe { libc::epoll_ctl(epfd, op as _, fd, event as *mut _ as *mut _) })
}

/// Wait for an event on the specified epoll instance.
///
/// `events` specifies a buffer of `EpollEvent`s where information on the ready events should be
/// placed.
///
/// `timeout` is the amount of time in milliseconds that this function should block until either a)
/// an event becomes available or b) a signal handler interrupts the call. A timeout of 0 will
/// cause this function to never block, and a timeout of -1 will block indefinitely.
#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout: libc::c_int) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::epoll_wait(
            epfd,
            events.as_mut_ptr() as *mut _,
            events.len().try_into().unwrap_or(libc::c_int::MAX),
            timeout,
        )
    })?;

    Ok(n as usize)
}

/// Atomically replace the signal mask and wait for an event on the specified epoll instance.
///
/// `epfd`, `events`, and `timeout` are as for [`epoll_wait()`]. See `epoll_pwait(2)` for more
/// information on `sigmask`.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub fn epoll_pwait(
    epfd: RawFd,
    events: &mut [EpollEvent],
    timeout: libc::c_int,
    sigmask: Option<&crate::SigSet>,
) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::epoll_pwait(
            epfd,
            events.as_mut_ptr() as *mut _,
            events.len().try_into().unwrap_or(libc::c_int::MAX),
            timeout,
            sigmask.map_or_else(core::ptr::null, |s| s.as_ref()),
        )
    })?;

    Ok(n as usize)
}

/// Wait for an event with a timeout in nanosecond resolution.
///
/// This is identical to [`epoll_pwait()`], except that the timeout takes a
/// [`TimeSpec`](./struct.TimeSpec.html) argument, allowing timeouts to be specified with
/// nanosecond resolution.
///
/// This system call was added in Linux 5.11; it will fail with `ENOSYS` on older kernels.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub fn epoll_pwait2(
    epfd: RawFd,
    events: &mut [EpollEvent],
    timeout: Option<crate::TimeSpec>,
    sigmask: Option<&crate::SigSet>,
) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::syscall(
            libc::SYS_epoll_pwait2,
            epfd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            events.len().try_into().unwrap_or(libc::c_int::MAX),
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
            sigmask.map_or_else(core::ptr::null, |s| s.as_ref()),
            core::mem::size_of::<libc::sigset_t>(),
        ) as i32
    })?;

    Ok(n as usize)
}

/// A wrapper around an epoll instance.
#[derive(Debug)]
pub struct Epoll(FileDesc);

impl Epoll {
    /// Create a new epoll instance with the specified flags.
    #[inline]
    pub fn new(flags: EpollFlags) -> Result<Self> {
        epoll_create1(flags).map(Self)
    }

    #[inline]
    pub fn ctl(&self, op: EpollCtlOp, fd: RawFd, events: EpollEvents, data: u64) -> Result<()> {
        epoll_ctl(self.0.fd(), op, fd, &mut EpollEvent::new(events, data))
    }

    /// Add a file descriptor to the interest list of this epoll instance.
    #[inline]
    pub fn add(&self, fd: RawFd, events: EpollEvents, data: u64) -> Result<()> {
        self.ctl(EpollCtlOp::ADD, fd, events, data)
    }

    /// Modify the settings associated with the given file descriptor in the interest list of this
    /// epoll instance.
    #[inline]
    pub fn modify(&self, fd: RawFd, events: EpollEvents, data: u64) -> Result<()> {
        self.ctl(EpollCtlOp::MOD, fd, events, data)
    }

    /// Remove the given file descriptor from the interest list of this epoll instance.
    #[inline]
    pub fn del(&self, fd: RawFd) -> Result<()> {
        Error::unpack_nz(unsafe {
            libc::epoll_ctl(self.fd(), EpollCtlOp::DEL as _, fd, core::ptr::null_mut())
        })
    }

    /// Wait for new events on this epoll instance.
    ///
    /// See [`epoll_wait()`].
    #[inline]
    pub fn wait(&self, events: &mut [EpollEvent], timeout: libc::c_int) -> Result<usize> {
        epoll_wait(self.0.fd(), events, timeout)
    }

    /// Wait for new events on this epoll instance.
    ///
    /// See [`epoll_pwait()`].
    #[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn pwait(
        &self,
        events: &mut [EpollEvent],
        timeout: libc::c_int,
        sigmask: Option<&crate::SigSet>,
    ) -> Result<usize> {
        epoll_pwait(self.0.fd(), events, timeout, sigmask)
    }

    /// Wait for new events on this epoll instance.
    ///
    /// See [`epoll_pwait2()`].
    ///
    /// This system call was added in Linux 5.11; it will fail with `ENOSYS` on older kernels.
    #[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn pwait2(
        &self,
        events: &mut [EpollEvent],
        timeout: Option<crate::TimeSpec>,
        sigmask: Option<&crate::SigSet>,
    ) -> Result<usize> {
        epoll_pwait2(self.0.fd(), events, timeout, sigmask)
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `Epoll` wrapper around the given epoll file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid epoll instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl AsRef<BorrowedFd> for Epoll {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for Epoll {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for Epoll {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for Epoll {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}
