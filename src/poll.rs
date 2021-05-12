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
    if fds.len() > libc::nfds_t::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    let n =
        Error::unpack(unsafe { libc::poll(fds.as_mut_ptr() as *mut _, fds.len() as _, timeout) })?;

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
    if fds.len() > libc::nfds_t::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    let n = Error::unpack(unsafe {
        sys::ppoll(
            fds.as_mut_ptr() as *mut _,
            fds.len() as _,
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
            sigmask.map_or_else(core::ptr::null, |s| s.as_ref()),
        )
    })?;

    Ok(n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll() {
        let (r1, w1) = crate::pipe().unwrap();
        let (r2, w2) = crate::pipe().unwrap();

        let mut fds = [
            PollFd::new(r1.fd(), PollEvents::IN),
            PollFd {
                fd: r2.fd(),
                events: PollEvents::IN,
                revents: PollEvents::empty(),
            },
        ];

        // Nothing to start
        assert_eq!(poll(&mut fds, 0).unwrap(), 0);

        // Now we write some data and test again
        w1.write_all(b"a").unwrap();
        assert_eq!(poll(&mut fds, 0).unwrap(), 1);
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);

        // Now make sure reading two files works
        w2.write_all(b"a").unwrap();
        assert_eq!(poll(&mut fds, 0).unwrap(), 2);
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);
        assert_eq!(fds[1].fd, r2.fd());
        assert_eq!(fds[1].revents, PollEvents::IN);

        // Now try without a timeout
        assert_eq!(poll(&mut fds, 0).unwrap(), 2);
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);
        assert_eq!(fds[1].fd, r2.fd());
        assert_eq!(fds[1].revents, PollEvents::IN);
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    ))]
    #[test]
    fn test_ppoll() {
        let (r1, w1) = crate::pipe().unwrap();
        let (r2, w2) = crate::pipe().unwrap();

        let mut fds = [
            PollFd {
                fd: r1.fd(),
                events: PollEvents::IN,
                revents: PollEvents::empty(),
            },
            PollFd {
                fd: r2.fd(),
                events: PollEvents::IN,
                revents: PollEvents::empty(),
            },
        ];

        // Nothing to start
        assert_eq!(
            ppoll(
                &mut fds,
                Some(crate::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0
                }),
                None
            )
            .unwrap(),
            0,
        );

        // Now we write some data and test again
        w1.write_all(b"a").unwrap();
        assert_eq!(
            ppoll(
                &mut fds,
                Some(crate::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0
                }),
                None
            )
            .unwrap(),
            1,
        );
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);

        // Now make sure reading two files works
        w2.write_all(b"a").unwrap();
        assert_eq!(
            ppoll(
                &mut fds,
                Some(crate::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0
                }),
                None
            )
            .unwrap(),
            2,
        );
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);
        assert_eq!(fds[1].fd, r2.fd());
        assert_eq!(fds[1].revents, PollEvents::IN);

        // Now try without a timeout
        assert_eq!(ppoll(&mut fds, None, None).unwrap(), 2);
        assert_eq!(fds[0].fd, r1.fd());
        assert_eq!(fds[0].revents, PollEvents::IN);
        assert_eq!(fds[1].fd, r2.fd());
        assert_eq!(fds[1].revents, PollEvents::IN);
    }
}
