use crate::internal_prelude::*;

use crate::time::{ClockId, TimeSpec};

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

// XXX: Hardcoding this is not ideal
const TFD_IOC_SET_TICKS: libc::c_ulong = 1074287616;

bitflags::bitflags! {
    /// Flags for [`timerfd_create()`].
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct TimerfdFlags: libc::c_int {
        /// Set the close-on-exec flag on the new file descriptor.
        const CLOEXEC = libc::TFD_CLOEXEC;
        /// Set the new file descriptor as non-blocking.
        const NONBLOCK = libc::TFD_NONBLOCK;
    }
}

bitflags::bitflags! {
    /// Flags for [`timerfd_settime()`].
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[derive(Default)]
    pub struct TimerfdTimerFlags: libc::c_int {
        const ABSTIME = libc::TFD_TIMER_ABSTIME;
        const CANCEL_ON_SET = sys::TFD_TIMER_CANCEL_ON_SET;
    }
}

/// Create a new timer file descriptor.
///
/// `clockid` specifies the clock used to mark the progress of the timer (see `timerfd_create(2)`.
/// `flags` specifies attributes used to modify the behavior of the timerfd.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn timerfd_create(clockid: ClockId, flags: TimerfdFlags) -> Result<FileDesc> {
    unsafe { Error::unpack_fdesc(libc::timerfd_create(clockid.as_raw() as _, flags.bits)) }
}

#[inline]
fn tspec_empty(ts: &libc::timespec) -> bool {
    ts.tv_sec == 0 && ts.tv_nsec == 0
}

/// Arm/disarm the timer.
///
/// `fd` is a file descriptor referring to the timerfd to operate on.
///
/// `flags` is a set of flags that modify behavior. See [`TimerfdTimerFlags`] and
/// `timerfd_create(2)` for more information.
///
/// `interval` specifies the period for repeated expirations after the initial expiration, and
/// `value` specifies the initial expiration time. If `interval` is `None` then the timer will only
/// expire once; if `value` is `None` then the timer is disarmed. (This is done by translating
/// `None` to `TimerSpec { tv_sec: 0, tv_nsec: 0 }`; specifying that value will also perform the
/// same actions.)
///
/// On success, the previous `(interval, value)` settings are returned, as for
/// [`timerfd_gettime()`].
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn timerfd_settime(
    fd: RawFd,
    flags: TimerfdTimerFlags,
    interval: Option<TimeSpec>,
    value: Option<TimeSpec>,
) -> Result<(Option<TimeSpec>, Option<TimeSpec>)> {
    let new_itspec = libc::itimerspec {
        it_interval: *interval
            .unwrap_or(TimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            })
            .as_ref(),
        it_value: *value
            .unwrap_or(TimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            })
            .as_ref(),
    };

    let mut old_itspec = MaybeUninit::uninit();
    Error::unpack_nz(unsafe {
        libc::timerfd_settime(fd, flags.bits, &new_itspec, old_itspec.as_mut_ptr())
    })?;
    let old_itspec = unsafe { old_itspec.assume_init() };

    Ok((
        if !tspec_empty(&old_itspec.it_interval) {
            Some(old_itspec.it_interval.into())
        } else {
            None
        },
        if !tspec_empty(&old_itspec.it_value) {
            Some(old_itspec.it_value.into())
        } else {
            None
        },
    ))
}

/// Get the current setting of the timer file descriptor.
///
/// This returns a `(interval, value)` tuple. See [`timerfd_settime()`] and `timerfd_create(2)`
/// for more information.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn timerfd_gettime(fd: RawFd) -> Result<(Option<TimeSpec>, Option<TimeSpec>)> {
    let mut itspec = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::timerfd_gettime(fd, itspec.as_mut_ptr()) })?;
    let itspec = unsafe { itspec.assume_init() };

    Ok((
        if !tspec_empty(&itspec.it_interval) {
            Some(itspec.it_interval.into())
        } else {
            None
        },
        if !tspec_empty(&itspec.it_value) {
            Some(itspec.it_value.into())
        } else {
            None
        },
    ))
}

/// A wrapper around an timer file descriptor.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Debug)]
pub struct TimerFd(FileDesc);

impl TimerFd {
    /// Create a new timer file descriptor.
    ///
    /// See [`timerfd_create()`].
    #[inline]
    pub fn new(clockid: ClockId, flags: TimerfdFlags) -> Result<Self> {
        timerfd_create(clockid, flags).map(Self)
    }

    /// Arm/disarm the timer.
    ///
    /// See [`timerfd_settime()`].
    #[inline]
    pub fn settime(
        &self,
        flags: TimerfdTimerFlags,
        interval: Option<TimeSpec>,
        value: Option<TimeSpec>,
    ) -> Result<(Option<TimeSpec>, Option<TimeSpec>)> {
        timerfd_settime(self.fd(), flags, interval, value)
    }

    /// Get the current setting of the timer file descriptor.
    ///
    /// See [`timerfd_gettime()`].
    #[inline]
    pub fn gettime(&self) -> Result<(Option<TimeSpec>, Option<TimeSpec>)> {
        timerfd_gettime(self.fd())
    }

    /// Read the number of timer expirations that have occurred.
    ///
    /// See the description of the `read(2)` operation in `timerfd_create(2)` for more information.
    #[inline]
    pub fn read_expirations(&self) -> Result<u64> {
        let mut buf = [0u8; 8];
        if crate::read(self.fd(), &mut buf)? != 8 {
            return Err(Error::from_code(libc::EINVAL));
        }
        Ok(u64::from_ne_bytes(buf))
    }

    /// Adjust the number of timer expirations that have occurred.
    ///
    /// This is only available if the kernel was built with `CONFIG_CHECKPOINT_RESTORE`. See the
    /// description of the `ioctl(2)` operation in `timerfd_create(2)` for more information.
    #[inline]
    pub fn set_expirations(&self, mut expirations: u64) -> Result<()> {
        unsafe {
            crate::ioctl(
                self.fd(),
                TFD_IOC_SET_TICKS,
                &mut expirations as *mut _ as *mut _,
            )?;
        }
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

    /// Create a new `TimerFd` wrapper around the given timerfd file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid timerfd instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl From<TimerFd> for FileDesc {
    #[inline]
    fn from(t: TimerFd) -> Self {
        t.0
    }
}

impl AsRef<BorrowedFd> for TimerFd {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for TimerFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for TimerFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for TimerFd {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timerfd_expirations() {
        let tfd = TimerFd::new(ClockId::MONOTONIC, TimerfdFlags::CLOEXEC).unwrap();
        assert_eq!(
            tfd.settime(
                Default::default(),
                None,
                Some(TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 1,
                }),
            )
            .unwrap(),
            (None, None)
        );
        // Wait for one expiration
        assert_eq!(tfd.read_expirations().unwrap(), 1);
        assert_eq!(tfd.gettime().unwrap(), (None, None));

        // No more expirations
        tfd.as_ref().set_nonblocking(true).unwrap();
        assert_eq!(tfd.read_expirations().unwrap_err(), Errno::EAGAIN);
        tfd.as_ref().set_nonblocking(false).unwrap();

        match tfd.set_expirations(10) {
            Ok(()) => assert_eq!(tfd.read_expirations().unwrap(), 10),
            Err(e) => assert!(matches!(e.code(), libc::ENOTTY | libc::ENOSYS), "{}", e),
        }
    }
}
