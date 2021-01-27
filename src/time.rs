use crate::internal_prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct TimeSpec {
    pub tv_sec: libc::time_t,
    #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
    pub tv_nsec: i64,
    #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
    pub tv_nsec: libc::c_long,
}

#[repr(i32)]
pub enum Clock {
    RealTime = libc::CLOCK_REALTIME as i32,

    Monotonic = libc::CLOCK_MONOTONIC as i32,

    #[cfg(target_os = "linux")]
    BootTime = libc::CLOCK_BOOTTIME as i32,
}

#[inline]
pub fn clock_getres(clock: Clock) -> Result<TimeSpec> {
    let mut buf = MaybeUninit::<TimeSpec>::uninit();
    Error::unpack_nz(unsafe {
        libc::clock_getres(clock as libc::clockid_t, buf.as_mut_ptr() as *mut _)
    })?;
    Ok(unsafe { buf.assume_init() })
}

#[inline]
pub fn clock_gettime(clock: Clock) -> Result<TimeSpec> {
    let mut buf = MaybeUninit::<TimeSpec>::uninit();
    Error::unpack_nz(unsafe {
        libc::clock_gettime(clock as libc::clockid_t, buf.as_mut_ptr() as *mut _)
    })?;
    Ok(unsafe { buf.assume_init() })
}

#[inline]
pub fn clock_settime(clock: Clock, t: TimeSpec) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::clock_settime(clock as libc::clockid_t, &t as *const _ as *const _)
    })
}
