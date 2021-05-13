use crate::internal_prelude::*;

bitflags::bitflags! {
    #[repr(transparent)]
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
    /// Create a new `PollFd` structure.
    ///
    /// This constructor initializes `fd` and `events`, leaving `revents` empty.
    #[inline]
    pub const fn new(fd: RawFd, events: PollEvents) -> Self {
        Self {
            fd,
            events,
            revents: PollEvents::empty(),
        }
    }
}

/// Poll for new data on the specified file descriptors.
///
/// `fds` specifies which file descriptors should be polled for new data; it is also used to return
/// the events that were triggered on these file descriptors. See `poll(2)` for more details. On
/// success, the number of elements in `fds` for which the `revents` field has been filled in is
/// returned.
///
/// `timeout` specifies the maximum number of milliseconds to wait for an event to occur before
/// returning. If it is 0, `poll()` will return immediately. If it is -1, `poll()` will block
/// indefinitely (i.e. infinite timeout) waiting for events. (As a nonstandard extension, Linux
/// provides an infinite timeout if `timeout` is any negative number, not just -1.)
#[inline]
pub fn poll(fds: &mut [PollFd], timeout: libc::c_int) -> Result<usize> {
    if fds.len() > libc::nfds_t::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    let n =
        Error::unpack(unsafe { libc::poll(fds.as_mut_ptr() as *mut _, fds.len() as _, timeout) })?;

    Ok(n as usize)
}

/// Identical to [`poll()`], but allows changing the signal mask and providing a higher precision
/// timeout.
///
/// See `ppoll(2)` for a discussion of the meaning of `sigmask`. If `sigmask` is `None`, `ppoll()`
/// behaves like `poll()` (except for the different type of the `timeout` argument).
///
/// If `timeout` is `None`, `ppoll()` will block indefinitely waiting for events. Otherwise,
/// `timeout` specifies the maximum time to wait for an event to occur before returning.
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
