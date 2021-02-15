use crate::internal_prelude::*;

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

    #[cfg(target_os = "linux")]
    REALTIME_ALARM = CLOCK_REALTIME_ALARM,
    REALTIME_COARSE = CLOCK_REALTIME_COARSE,
    TAI = CLOCK_TAI,
    MONOTONIC_COARSE = CLOCK_MONOTONIC_COARSE,
    MONOTONIC_RAW = CLOCK_MONOTONIC_RAW,
    BOOTTIME = CLOCK_BOOTTIME,
    BOOTTIME_ALARM = CLOCK_BOOTTIME_ALARM,
}

impl ClockId {
    #[inline]
    pub fn getres(self) -> Result<TimeSpec> {
        clock_getres(self)
    }

    #[inline]
    pub fn gettime(self) -> Result<TimeSpec> {
        clock_gettime(self)
    }

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
    #[cfg(any(freebsdlike, netbsdlike, linux_like))]
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
/// Specifying 0 for `pid` is equivalent to specifying the current process's PID.
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
#[cfg(any(freebsdlike, netbsdlike, linux_like))]
#[inline]
pub fn clock_getcpuclockid(pid: libc::pid_t) -> Result<ClockId> {
    let mut clockid = MaybeUninit::uninit();

    match unsafe { sys::clock_getcpuclockid(pid, clockid.as_mut_ptr()) } {
        0 => Ok(ClockId(unsafe { clockid.assume_init() })),
        eno => Err(Error::from_code(eno)),
    }
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

    match unsafe { sys::pthread_getcpuclockid(thread, clockid.as_mut_ptr()) } {
        0 => Ok(ClockId(unsafe { clockid.assume_init() })),
        eno => Err(Error::from_code(eno)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

            if !isclose(left, right, 100_000) {
                panic!("assertion failed: {:?} !~ {:?}", left, right);
            }
        }};
    }

    #[cfg(any(freebsdlike, netbsdlike, linux_like))]
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
}
