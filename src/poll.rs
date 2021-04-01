use core::convert::TryInto;

use crate::internal_prelude::*;

bitflags::bitflags! {
    pub struct PollEvents: libc::c_short {
        const IN = libc::POLLIN;
        const RDNORM = libc::POLLRDNORM;
        const RDBAND = libc::POLLRDBAND;
        const PRI = libc::POLLPRI;
        const OUT = libc::POLLOUT;
        const WRNORM = libc::POLLWRNORM;
        const WRBAND = libc::POLLWRBAND;
        const ERR = libc::POLLERR;
        const HUP = libc::POLLHUP;
        const NVAL = libc::POLLNVAL;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct PollFd {
    pub fd: RawFd,
    pub events: PollEvents,
    pub revents: PollEvents,
}

impl PollFd {
    #[inline]
    pub const fn new(fd: RawFd, events: PollEvents) -> Self {
        Self {
            fd,
            events,
            revents: PollEvents::empty(),
        }
    }
}

#[inline]
pub fn poll(fds: &mut [PollFd], timeout: libc::c_int) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::poll(
            fds.as_mut_ptr() as *mut _,
            fds.len().try_into().unwrap(),
            timeout,
        )
    })?;

    Ok(n as usize)
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(linuxlike, freebsdlike, netbsdlike))]
#[inline]
pub fn ppoll(
    fds: &mut [PollFd],
    timeout: Option<crate::TimeSpec>,
    sigmask: Option<&crate::SigSet>,
) -> Result<usize> {
    let n = Error::unpack(unsafe {
        sys::ppoll(
            fds.as_mut_ptr() as *mut _,
            fds.len().try_into().unwrap(),
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
            sigmask.map_or_else(core::ptr::null, |s| s.as_ref()),
        )
    })?;

    Ok(n as usize)
}
