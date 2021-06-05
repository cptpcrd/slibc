use crate::internal_prelude::*;

use crate::{SigSet, TimeSpec, Timeval};

/// A set of file descriptors to be used with [`select()`]/[`pselect()`].
///
/// Note that a file descriptor can only contain file descriptors less than
/// [`FD_SETSIZE`](./constant.FD_SETSIZE.html). You can check if a given file descriptor will fit
/// in an `FdSet` using [`Self::can_contain()`].
#[derive(Copy, Clone)]
pub struct FdSet(libc::fd_set);

impl FdSet {
    /// Create an empty file descriptor set.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let mut set = MaybeUninit::uninit();
            libc::FD_ZERO(set.as_mut_ptr());
            Self(set.assume_init())
        }
    }

    /// Clear this file descriptor set.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            libc::FD_ZERO(&mut self.0);
        }
    }

    /// Check whether this file descriptor set contains a given file descriptor.
    ///
    /// This always returns `false` if `fd` cannot be added to a file descriptor set (see
    /// [`Self::can_contain()`]).
    #[inline]
    pub fn contains(&self, fd: RawFd) -> bool {
        Self::can_contain(fd) && unsafe { libc::FD_ISSET(fd, &self.0 as *const _ as *mut _) }
    }

    /// Add the specified file descriptor to the set.
    ///
    /// # Panics
    ///
    /// Panics if `fd` cannot be added to a file descriptor set (see [`Self::can_contain()`]).
    #[inline]
    pub fn add(&mut self, fd: RawFd) {
        if !Self::can_contain(fd) {
            panic!("file descriptor cannot fit in an FdSet");
        }

        unsafe {
            libc::FD_SET(fd, &mut self.0);
        }
    }

    /// Remove the specified file descriptor from the set if it is present.
    ///
    /// This does nothing if `fd` cannot be added to a file descriptor set (see
    /// [`Self::can_contain()`]).
    #[inline]
    pub fn remove(&mut self, fd: RawFd) {
        if Self::can_contain(fd) {
            unsafe {
                libc::FD_CLR(fd, &mut self.0);
            }
        }
    }

    /// Check whether the given file descriptor can fit in a file descriptor set.
    ///
    /// This returns `true` if the file descriptor is in the range `0..FD_SETSIZE`.
    #[inline]
    pub fn can_contain(fd: RawFd) -> bool {
        (fd as usize) < libc::FD_SETSIZE
    }

    /// Create an iterator over the file descriptors in this set.
    ///
    /// `n` specifies the maximum number of file descriptors in the set.
    #[inline]
    pub fn iter(self, n: usize) -> FdSetIter {
        FdSetIter { set: self, i: 0, n }
    }
}

impl Default for FdSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::iter::FromIterator<RawFd> for FdSet {
    #[inline]
    fn from_iter<I: IntoIterator<Item = RawFd>>(it: I) -> Self {
        let mut set = Self::new();
        set.extend(it);
        set
    }
}

impl Extend<RawFd> for FdSet {
    #[inline]
    fn extend<I: IntoIterator<Item = RawFd>>(&mut self, it: I) {
        for fd in it.into_iter() {
            self.add(fd);
        }
    }
}

#[derive(Clone)]
pub struct FdSetIter {
    set: FdSet,
    i: RawFd,
    n: usize,
}

impl FdSetIter {
    /// Returns the remaining number of file descriptors based on the `n` value passed to
    /// [`FdSet::iter()`].
    ///
    /// This is the original value of `n`, with 1 subtracted for each yielded file descriptor.
    #[inline]
    pub fn remaining_n(&self) -> usize {
        self.n
    }
}

impl Iterator for FdSetIter {
    type Item = RawFd;

    fn next(&mut self) -> Option<RawFd> {
        if self.n > 0 {
            while FdSet::can_contain(self.i) {
                let i = self.i;
                self.i += 1;
                if self.set.contains(i) {
                    self.n -= 1;
                    return Some(i);
                }
            }
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.n))
    }
}

#[inline]
pub fn select(
    nfds: RawFd,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    errfds: Option<&mut FdSet>,
    timeout: Option<&Timeval>,
) -> Result<usize> {
    let mut timeout: Option<Timeval> = timeout.cloned();

    let n = Error::unpack(unsafe {
        libc::select(
            nfds,
            readfds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            writefds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            errfds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            timeout.as_mut().map_or_else(core::ptr::null_mut, |t| t) as *mut _,
        )
    })?;

    Ok(n as usize)
}

#[inline]
pub fn pselect(
    nfds: RawFd,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    errfds: Option<&mut FdSet>,
    timeout: Option<&TimeSpec>,
    sigmask: Option<&SigSet>,
) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::pselect(
            nfds,
            readfds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            writefds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            errfds.map_or_else(core::ptr::null_mut, |f| &mut f.0),
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
            sigmask.map_or_else(core::ptr::null, |s| s.as_ref()),
        )
    })?;

    Ok(n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_fdset() {
        let mut fds = FdSet::default();
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[]);

        fds.add(0);
        fds.add(1);
        fds.add(2);
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[0, 1, 2]);

        fds.remove(2);
        fds.remove(3);
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[0, 1]);

        // No-ops
        fds.remove(-1);
        fds.remove(libc::FD_SETSIZE as RawFd);
        fds.remove(RawFd::MIN);
        fds.remove(RawFd::MAX);
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[0, 1]);

        fds.clear();
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[]);

        fds = [3, 4].iter().copied().collect();
        assert_eq!(fds.iter(libc::FD_SETSIZE).collect::<Vec<_>>(), &[3, 4]);

        assert!(FdSet::can_contain(0));
        assert!(FdSet::can_contain(libc::FD_SETSIZE as RawFd - 1));
        assert!(!FdSet::can_contain(-1));
        assert!(!FdSet::can_contain(libc::FD_SETSIZE as RawFd));
    }

    #[test]
    fn test_fdsetiter() {
        let mut it = [0, 1, 2].iter().copied().collect::<FdSet>().iter(4);
        assert_eq!(it.remaining_n(), 4);
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.remaining_n(), 3);
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.remaining_n(), 2);
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.remaining_n(), 1);
        assert_eq!(it.next(), None);
        assert_eq!(it.remaining_n(), 1);
    }

    #[test]
    fn test_select() {
        let (r1, w1) = crate::pipe().unwrap();
        let (r2, w2) = crate::pipe().unwrap();

        let mut readfds = FdSet::new();
        let mut writefds = FdSet::new();
        let mut errfds = FdSet::new();

        macro_rules! load_sets {
            () => {
                readfds.add(r1.fd());
                readfds.add(r2.fd());
                errfds.add(r1.fd());
                errfds.add(r2.fd());
                errfds.add(w1.fd());
                errfds.add(w2.fd());
            };
        }

        let nfds = [r1.fd(), r2.fd(), w1.fd(), w2.fd()]
            .iter()
            .copied()
            .max()
            .unwrap();

        let timeout_0 = crate::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        // Nothing to start
        load_sets!();
        assert_eq!(
            select(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0)
            )
            .unwrap(),
            0
        );

        // Now we write some data and test again
        w1.write_all(b"a").unwrap();
        load_sets!();
        assert_eq!(
            select(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0)
            )
            .unwrap(),
            1
        );
        assert!(readfds.contains(r1.fd()));

        // Now make sure reading two files works
        w2.write_all(b"a").unwrap();
        load_sets!();
        assert_eq!(
            select(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0)
            )
            .unwrap(),
            2
        );
        assert!(readfds.contains(r1.fd()));
        assert!(readfds.contains(r2.fd()));

        // Now try without a timeout
        load_sets!();
        assert_eq!(
            select(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                None
            )
            .unwrap(),
            2
        );
        assert!(readfds.contains(r1.fd()));
    }

    #[test]
    fn test_pselect() {
        let (r1, w1) = crate::pipe().unwrap();
        let (r2, w2) = crate::pipe().unwrap();

        let mut readfds = FdSet::new();
        let mut writefds = FdSet::new();
        let mut errfds = FdSet::new();

        macro_rules! load_sets {
            () => {
                readfds.add(r1.fd());
                readfds.add(r2.fd());
                errfds.add(r1.fd());
                errfds.add(r2.fd());
                errfds.add(w1.fd());
                errfds.add(w2.fd());
            };
        }

        let nfds = [r1.fd(), r2.fd(), w1.fd(), w2.fd()]
            .iter()
            .copied()
            .max()
            .unwrap();

        let timeout_0 = crate::TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        // Nothing to start
        load_sets!();
        assert_eq!(
            pselect(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0),
                None,
            )
            .unwrap(),
            0
        );

        // Now we write some data and test again
        w1.write_all(b"a").unwrap();
        load_sets!();
        assert_eq!(
            pselect(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0),
                None,
            )
            .unwrap(),
            1
        );
        assert!(readfds.contains(r1.fd()));

        // Now make sure reading two files works
        w2.write_all(b"a").unwrap();
        load_sets!();
        assert_eq!(
            pselect(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                Some(&timeout_0),
                None,
            )
            .unwrap(),
            2
        );
        assert!(readfds.contains(r1.fd()));
        assert!(readfds.contains(r2.fd()));

        // Now try without a timeout
        load_sets!();
        assert_eq!(
            pselect(
                nfds,
                Some(&mut readfds),
                Some(&mut writefds),
                Some(&mut errfds),
                None,
                None,
            )
            .unwrap(),
            2
        );
        assert!(readfds.contains(r1.fd()));
    }
}
