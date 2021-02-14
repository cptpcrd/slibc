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
            $(#[doc = $doc:literal])*
            $name:ident = $libc_name:ident,
        )+
    )*) => {
        impl ClockId {
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                $(#[doc = $doc])*
                pub const $name: Self = Self(libc::$libc_name);
            )*)*
        }
    };
}

define_clockids! {
    #[cfg(all())]
    REALTIME = CLOCK_REALTIME,
    MONOTONIC = CLOCK_MONOTONIC,
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
    VIRTUAL = CLOCK_VIRTUAL,
    PROF = CLOCK_PROF,

    #[cfg(target_os = "openbsd")]
    BOOTTIME = CLOCK_BOOTTIME,
    UPTIME = CLOCK_UPTIME,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
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

    /// Get the clock ID for this process using `clock_getcpuclockid()`.
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
    #[cfg(any(freebsdlike, netbsdlike, linux))]
    #[inline]
    pub fn get_for_process(pid: libc::pid_t) -> Result<Self> {
        clock_getcpuclockid(pid)
    }

    /// Get the clock ID for the given thread using `pthread_getcpuclockid()`.
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
        )))
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
#[cfg(any(freebsdlike, netbsdlike, linux))]
#[inline]
pub fn clock_getcpuclockid(pid: libc::pid_t) -> Result<ClockId> {
    let mut clockid = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { sys::clock_getcpuclockid(pid, clockid.as_mut_ptr()) })?;
    Ok(ClockId(unsafe { clockid.assume_init() }))
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd")))
)]
#[cfg(any(freebsdlike, target_os = "openbsd"))]
#[inline]
pub fn pthread_getcpuclockid(thread: libc::pthread_t) -> Result<ClockId> {
    let mut clockid = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { sys::pthread_getcpuclockid(thread, clockid.as_mut_ptr()) })?;
    Ok(ClockId(unsafe { clockid.assume_init() }))
}
