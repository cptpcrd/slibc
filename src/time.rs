use crate::internal_prelude::*;

use core::time::Duration;

#[cfg(feature = "std")]
use std::time::SystemTime;

/// Represents a C `timespec` structure.
///
/// This can either represent a duration in time, or a timestamp. As such, this struct can be
/// converted to and from `Duration`s. and if the `std` feature is enabled then it can also be
/// converted to and from `SystemTime`s.
#[allow(deprecated)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct TimeSpec {
    pub tv_sec: libc::time_t,
    #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
    pub tv_nsec: i64,
    #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
    pub tv_nsec: libc::c_long,
}

/// This ensures that `TimeSpec` is the same size as `libc::timespec`. The layout is verified in a
/// test.
const _TIMESPEC_SIZE_CHECK: TimeSpec =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<libc::timespec>()]) };

impl AsRef<libc::timespec> for TimeSpec {
    #[inline]
    fn as_ref(&self) -> &libc::timespec {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl From<libc::timespec> for TimeSpec {
    #[inline]
    fn from(ts: libc::timespec) -> Self {
        Self {
            tv_sec: ts.tv_sec,
            tv_nsec: ts.tv_nsec,
        }
    }
}

impl From<Duration> for TimeSpec {
    #[inline]
    fn from(d: Duration) -> Self {
        Self {
            tv_sec: d.as_secs() as _,
            tv_nsec: d.subsec_nanos() as _,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<SystemTime> for TimeSpec {
    #[inline]
    fn from(t: SystemTime) -> Self {
        match t.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.into(),
            Err(e) => {
                let d = e.duration();
                Self {
                    tv_sec: (-(d.as_secs() as i64) - 1) as _,
                    tv_nsec: (1_000_000_000 - d.subsec_nanos()) as _,
                }
            }
        }
    }
}

impl core::convert::TryFrom<TimeSpec> for Duration {
    type Error = Duration;

    #[inline]
    fn try_from(t: TimeSpec) -> core::result::Result<Self, Self::Error> {
        if t.tv_sec >= 0 {
            Ok(Duration::new(t.tv_sec as _, t.tv_nsec as _))
        } else {
            Err(Duration::new(
                (-(t.tv_sec as i64) - 1) as _,
                (1_000_000_000 - t.tv_nsec) as _,
            ))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<TimeSpec> for SystemTime {
    #[inline]
    fn from(t: TimeSpec) -> Self {
        use core::convert::TryFrom;

        match Duration::try_from(t) {
            Ok(d) => SystemTime::UNIX_EPOCH + d,
            Err(d) => SystemTime::UNIX_EPOCH - d,
        }
    }
}

/// Represents a C `timeval` structure.
///
/// This can be converted to and from a [`TimeSpec]` (though converting a `TimeSpec` to a `Timeval`
/// is lossy), and from there it can be converted to and from `Duration` and `SystemTime`.
#[allow(deprecated)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct Timeval {
    pub tv_sec: libc::time_t,
    pub tv_usec: libc::suseconds_t,
}

/// This ensures that `Timeval` is the same size as `libc::timeval`. The layout is verified in a
/// test.
const _TIMEVAL_SIZE_CHECK: Timeval =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<libc::timeval>()]) };

impl AsRef<libc::timeval> for Timeval {
    #[inline]
    fn as_ref(&self) -> &libc::timeval {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl From<libc::timeval> for Timeval {
    #[inline]
    fn from(tv: libc::timeval) -> Self {
        Self {
            tv_sec: tv.tv_sec,
            tv_usec: tv.tv_usec,
        }
    }
}

impl From<TimeSpec> for Timeval {
    #[inline]
    fn from(t: TimeSpec) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_usec: (t.tv_nsec / 1000) as _,
        }
    }
}

impl From<Timeval> for TimeSpec {
    #[inline]
    fn from(t: Timeval) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_nsec: (t.tv_usec * 1000) as _,
        }
    }
}

/// A clock ID for use with e.g. [`clock_gettime()`].
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ClockId(libc::clockid_t);

macro_rules! define_clockids {
    ($(
        #[cfg($cfg:meta)]
        $(
            $(#[doc = $doc1:literal])*
            $name1:ident = $libc_name:ident,
        )*
        $(
            @sys,
            $(
                $(#[doc = $doc2:literal])*
                $name2:ident = $sys_name:ident,
            )+
        )?
    )*) => {
        impl ClockId {
            $(
                $(
                    #[cfg($cfg)]
                    #[cfg_attr(docsrs, doc(cfg($cfg)))]
                    $(#[doc = $doc1])*
                    pub const $name1: Self = Self(libc::$libc_name);
                )*
                $($(
                    #[cfg($cfg)]
                    #[cfg_attr(docsrs, doc(cfg($cfg)))]
                    $(#[doc = $doc2])*
                    pub const $name2: Self = Self(sys::$sys_name);
                )+)?
            )*
        }
    };
}

define_clockids! {
    #[cfg(all())]
    REALTIME = CLOCK_REALTIME,
    MONOTONIC = CLOCK_MONOTONIC,
    @sys,
    PROCESS_CPUTIME_ID = CLOCK_PROCESS_CPUTIME_ID,
    THREAD_CPUTIME_ID = CLOCK_THREAD_CPUTIME_ID,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    REALTIME_PRECISE = CLOCK_REALTIME_PRECISE,
    REALTIME_FAST = CLOCK_REALTIME_FAST,
    MONOTONIC_PRECISE = CLOCK_MONOTONIC_PRECISE,
    MONOTONIC_FAST = CLOCK_MONOTONIC_FAST,
    UPTIME_PRECISE = CLOCK_UPTIME_PRECISE,
    UPTIME_FAST = CLOCK_UPTIME_FAST,
    SECOND = CLOCK_SECOND,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd"))]
    @sys,
    VIRTUAL = CLOCK_VIRTUAL,
    PROF = CLOCK_PROF,

    #[cfg(target_os = "openbsd")]
    @sys,
    BOOTTIME = CLOCK_BOOTTIME,
    UPTIME = CLOCK_UPTIME,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    @sys,
    MONOTONIC_RAW = CLOCK_MONOTONIC_RAW,
    MONOTONIC_RAW_APPROX = CLOCK_MONOTONIC_RAW_APPROX,
    UPTIME_RAW = CLOCK_UPTIME_RAW,
    UPTIME_RAW_APPROX = CLOCK_UPTIME_RAW_APPROX,

    #[cfg(any(target_os = "linux", target_os = "android"))]
    REALTIME_ALARM = CLOCK_REALTIME_ALARM,
    REALTIME_COARSE = CLOCK_REALTIME_COARSE,
    TAI = CLOCK_TAI,
    MONOTONIC_COARSE = CLOCK_MONOTONIC_COARSE,
    MONOTONIC_RAW = CLOCK_MONOTONIC_RAW,
    BOOTTIME = CLOCK_BOOTTIME,
    BOOTTIME_ALARM = CLOCK_BOOTTIME_ALARM,
}

impl ClockId {
    /// See [`clock_getres()`].
    #[inline]
    pub fn getres(self) -> Result<TimeSpec> {
        clock_getres(self)
    }

    /// See [`clock_gettime()`].
    #[inline]
    pub fn gettime(self) -> Result<TimeSpec> {
        clock_gettime(self)
    }

    /// See [`clock_settime()`].
    #[inline]
    pub fn settime(self, t: TimeSpec) -> Result<()> {
        clock_settime(self, t)
    }

    /// Get the clock ID for the specified process using [`clock_getcpuclockid()`].
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )))
    )]
    #[cfg(any(freebsdlike, netbsdlike, target_os = "linux"))]
    #[inline]
    pub fn get_for_process(pid: libc::pid_t) -> Result<Self> {
        clock_getcpuclockid(pid)
    }

    /// Get the clock ID for the specified thread using [`pthread_getcpuclockid()`].
    #[cfg_attr(
        docsrs,
        doc(cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd")))
    )]
    #[cfg(any(freebsdlike, target_os = "openbsd"))]
    #[inline]
    pub fn get_for_thread(thread: libc::pthread_t) -> Result<Self> {
        pthread_getcpuclockid(thread)
    }

    /// Get the raw `clockid_t` value of this clock ID.
    #[inline]
    pub fn as_raw(&self) -> libc::clockid_t {
        self.0
    }
}

#[inline]
pub fn clock_getres(clock: ClockId) -> Result<TimeSpec> {
    let mut buf = MaybeUninit::<TimeSpec>::uninit();
    Error::unpack_nz(unsafe { libc::clock_getres(clock.0, buf.as_mut_ptr() as *mut _) })?;
    Ok(unsafe { buf.assume_init() })
}

#[inline]
pub fn clock_gettime(clock: ClockId) -> Result<TimeSpec> {
    let mut buf = MaybeUninit::<TimeSpec>::uninit();
    Error::unpack_nz(unsafe { libc::clock_gettime(clock.0, buf.as_mut_ptr() as *mut _) })?;
    Ok(unsafe { buf.assume_init() })
}

#[inline]
pub fn clock_settime(clock: ClockId, t: TimeSpec) -> Result<()> {
    Error::unpack_nz(unsafe { sys::clock_settime(clock.0, &t as *const _ as *const _) })
}

/// Get the clock ID of the specified process's CPU-time clock.
///
/// Specifying 0 for `pid` will return a clock ID that can be used to measure the current process's
/// CPU time.
///
/// If `pid` is 0 or the current process's PID, the returned clock ID may be a "magic" value that
/// always refers to the process that checks it with [`clock_gettime()`] (even in, say, a
/// `fork()`ed child).
///
/// Additionally, some OSes (for example, OpenBSD) may not accept `pid` values other than 0 or the
/// current process's PID.
///
/// # Returned errors
///
/// The error codes returned vary wildly across platforms. For example, on Linux, musl libc doesn't
/// properly return `ESRCH` for nonexistant processes (it instead returns `EINVAL`). It may be
/// useful to simply ignore the error code and check if the process exists.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(freebsdlike, netbsdlike, target_os = "linux"))]
#[inline]
pub fn clock_getcpuclockid(pid: libc::pid_t) -> Result<ClockId> {
    let mut clockid = MaybeUninit::uninit();
    Error::unpack_eno(unsafe { sys::clock_getcpuclockid(pid, clockid.as_mut_ptr()) })?;
    Ok(ClockId(unsafe { clockid.assume_init() }))
}

/// Get the clock ID of the specified thread's CPU-time clock.
///
/// The warnings regarding errors returned from [`clock_getcpuclockid()`] apply.
#[cfg_attr(
    docsrs,
    doc(cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd")))
)]
#[cfg(any(freebsdlike, target_os = "openbsd"))]
#[inline]
pub fn pthread_getcpuclockid(thread: libc::pthread_t) -> Result<ClockId> {
    let mut clockid = MaybeUninit::uninit();
    Error::unpack_eno(unsafe { sys::pthread_getcpuclockid(thread, clockid.as_mut_ptr()) })?;
    Ok(ClockId(unsafe { clockid.assume_init() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timespec_timeval_layout() {
        let ts1 = TimeSpec {
            tv_sec: 123,
            tv_nsec: 456,
        };
        let ts2 = libc::timespec {
            tv_sec: 123,
            tv_nsec: 456,
        };
        assert_eq!(ts1.as_ref(), &ts2);
        assert_eq!(
            unsafe { core::mem::transmute::<_, libc::timespec>(ts1) },
            ts2
        );
        assert_eq!(ts1, TimeSpec::from(ts2));

        let tv1 = Timeval {
            tv_sec: 123,
            tv_usec: 456,
        };
        let tv2 = libc::timeval {
            tv_sec: 123,
            tv_usec: 456,
        };
        assert_eq!(tv1.as_ref(), &tv2);
        assert_eq!(
            unsafe { core::mem::transmute::<_, libc::timeval>(tv1) },
            tv2
        );
        assert_eq!(tv1, Timeval::from(tv2));
    }

    fn isclose(t1: TimeSpec, t2: TimeSpec, nsec: u32) -> bool {
        if t1.tv_sec == t2.tv_sec {
            (t1.tv_sec - t2.tv_sec).abs() < nsec as _
        } else if t1.tv_sec == t2.tv_sec - 1 {
            1000000 - (t1.tv_sec - t2.tv_sec) < nsec as _
        } else if t2.tv_sec == t1.tv_sec - 1 {
            1000000 - (t2.tv_sec - t1.tv_sec) < nsec as _
        } else {
            false
        }
    }

    macro_rules! assert_close {
        ($left:expr, $right:expr $(,)?) => {{
            let left = $left;
            let right = $right;

            assert!(
                isclose(left, right, 100_000),
                "assertion failed: {:?} !~ {:?}",
                left,
                right,
            );
        }};
    }

    #[test]
    fn test_consistent_clocks() {
        for clock in &[ClockId::REALTIME, ClockId::MONOTONIC] {
            assert_close!(clock.gettime().unwrap(), clock.gettime().unwrap());
        }
    }

    #[cfg(any(freebsdlike, netbsdlike, target_os = "linux"))]
    #[test]
    fn test_clock_getcpuclockid() {
        assert_close!(
            ClockId::get_for_process(0).unwrap().gettime().unwrap(),
            ClockId::PROCESS_CPUTIME_ID.gettime().unwrap(),
        );

        assert_close!(
            ClockId::get_for_process(crate::getpid())
                .unwrap()
                .gettime()
                .unwrap(),
            ClockId::PROCESS_CPUTIME_ID.gettime().unwrap(),
        );

        match unsafe { crate::fork() }.unwrap() {
            None => unsafe { crate::_exit(1) },

            Some(pid) => {
                assert!(unsafe { libc::waitpid(pid, core::ptr::null_mut(), 0) } > 0);

                let eno = ClockId::get_for_process(pid).unwrap_err().code();
                assert!(matches!(
                    eno,
                    libc::ESRCH | libc::EINVAL | libc::EPERM | libc::ENOSYS
                ));
            }
        }
    }

    #[cfg(any(freebsdlike, target_os = "openbsd"))]
    #[test]
    fn test_pthread_getcpuclockid() {
        assert_close!(
            ClockId::get_for_thread(unsafe { libc::pthread_self() })
                .unwrap()
                .gettime()
                .unwrap(),
            ClockId::THREAD_CPUTIME_ID.gettime().unwrap(),
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_std_time_timespec() {
        use std::convert::TryFrom;

        assert_eq!(
            TimeSpec {
                tv_sec: 100,
                tv_nsec: 100,
            },
            TimeSpec::from(Duration::new(100, 100))
        );

        assert_eq!(
            Duration::try_from(TimeSpec {
                tv_sec: 100,
                tv_nsec: 100,
            }),
            Ok(Duration::new(100, 100))
        );
        assert_eq!(
            Duration::try_from(TimeSpec {
                tv_sec: -100,
                tv_nsec: 100,
            }),
            Err(Duration::new(99, 999_999_900))
        );

        assert_eq!(
            TimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            TimeSpec::from(SystemTime::UNIX_EPOCH)
        );

        assert_eq!(
            TimeSpec {
                tv_sec: 100,
                tv_nsec: 100,
            },
            TimeSpec::from(SystemTime::UNIX_EPOCH + Duration::new(100, 100))
        );
        assert_eq!(
            TimeSpec {
                tv_sec: -101,
                tv_nsec: 999_999_900,
            },
            TimeSpec::from(SystemTime::UNIX_EPOCH - Duration::new(100, 100))
        );

        assert_eq!(
            SystemTime::from(TimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            }),
            SystemTime::UNIX_EPOCH
        );

        assert_eq!(
            SystemTime::from(TimeSpec {
                tv_sec: 100,
                tv_nsec: 100,
            }),
            SystemTime::UNIX_EPOCH + Duration::new(100, 100)
        );
        assert_eq!(
            SystemTime::from(TimeSpec {
                tv_sec: -101,
                tv_nsec: 999_999_900,
            }),
            SystemTime::UNIX_EPOCH - Duration::new(100, 100)
        );
    }
}
