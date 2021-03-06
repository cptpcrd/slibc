use crate::internal_prelude::*;

use core::convert::TryInto;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct IoVecMut<'a>(libc::iovec, PhantomData<&'a mut [u8]>);

impl<'a> IoVecMut<'a> {
    #[inline]
    pub fn new(s: &'a mut [u8]) -> Self {
        Self(
            libc::iovec {
                iov_base: s.as_mut_ptr() as *mut _,
                iov_len: s.len(),
            },
            PhantomData,
        )
    }

    /// Advance a slice of `IoVecMut`s.
    ///
    /// This may modify elements in the slice if `n` only goes partially into one of the slices.
    ///
    /// ```
    /// # use slibc::IoVecMut;
    /// let mut buf1 = [0, 1, 2];
    /// let mut buf2 = [3, 4, 5];
    /// let mut bufs = [IoVecMut::new(&mut buf1), IoVecMut::new(&mut buf2)];
    /// let mut vecs = &mut bufs[..];
    ///
    /// vecs = IoVecMut::advance(vecs, 4);
    /// assert_eq!(vecs, [IoVecMut::new(&mut [4, 5])]);
    /// ```
    pub fn advance(mut bufs: &mut [Self], mut n: usize) -> &mut [Self] {
        while let Some((first, _)) = bufs.split_first_mut() {
            if n < first.len() {
                first.0.iov_base = unsafe { first.0.iov_base.add(n) };
                first.0.iov_len -= n;
                break;
            }

            n -= first.len();
            bufs = &mut bufs[1..];
        }

        bufs
    }
}

impl<'a> Deref for IoVecMut<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.0.iov_base as *const _, self.0.iov_len) }
    }
}

impl<'a> DerefMut for IoVecMut<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.0.iov_base as *mut _, self.0.iov_len) }
    }
}

impl PartialEq for IoVecMut<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl Eq for IoVecMut<'_> {}

impl Hash for IoVecMut<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

impl fmt::Debug for IoVecMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IoVecMut").field(&self.deref()).finish()
    }
}

#[repr(transparent)]
pub struct IoVec<'a>(libc::iovec, PhantomData<&'a [u8]>);

impl<'a> IoVec<'a> {
    #[inline]
    pub fn new(s: &'a [u8]) -> Self {
        Self(
            libc::iovec {
                iov_base: s.as_ptr() as *mut _,
                iov_len: s.len(),
            },
            PhantomData,
        )
    }

    /// Advance a slice of `IoVec`s.
    ///
    /// This may modify elements in the slice if `n` only goes partially into one of the slices.
    ///
    /// ```
    /// # use slibc::IoVec;
    /// let mut buf1 = [0, 1, 2];
    /// let mut buf2 = [3, 4, 5];
    /// let mut bufs = [IoVec::new(&mut buf1), IoVec::new(&mut buf2)];
    /// let mut vecs = &mut bufs[..];
    ///
    /// vecs = IoVec::advance(vecs, 4);
    /// assert_eq!(vecs, [IoVec::new(&mut [4, 5])]);
    /// ```
    pub fn advance(mut bufs: &mut [Self], mut n: usize) -> &mut [Self] {
        while let Some((first, _)) = bufs.split_first_mut() {
            if n < first.len() {
                first.0.iov_base = unsafe { first.0.iov_base.add(n) };
                first.0.iov_len -= n;
                break;
            }

            n -= first.len();
            bufs = &mut bufs[1..];
        }

        bufs
    }
}

impl<'a> Deref for IoVec<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.0.iov_base as *const _, self.0.iov_len) }
    }
}

impl PartialEq for IoVec<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl Eq for IoVec<'_> {}

impl Hash for IoVec<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

impl fmt::Debug for IoVec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IoVec").field(&self.deref()).finish()
    }
}

impl<'a> From<IoVecMut<'a>> for IoVec<'a> {
    #[inline]
    fn from(i: IoVecMut<'a>) -> IoVec<'a> {
        Self(i.0, PhantomData)
    }
}

#[inline]
pub fn readv(fd: RawFd, iov: &mut [IoVecMut]) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::readv(
            fd,
            iov.as_ptr() as *const _,
            iov.len().try_into().unwrap_or(i32::MAX),
        )
    })
}

#[inline]
pub fn writev(fd: RawFd, iov: &[IoVec]) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::writev(
            fd,
            iov.as_ptr() as *const _,
            iov.len().try_into().unwrap_or(i32::MAX),
        )
    })
}

#[cfg_attr(docsrs, doc(cfg(not(any(target_os = "macos", target_os = "ios")))))]
#[cfg(not(apple))]
#[inline]
pub fn preadv(fd: RawFd, iov: &mut [IoVecMut], offset: u64) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::preadv(
            fd,
            iov.as_ptr() as *const _,
            iov.len().try_into().unwrap_or(i32::MAX),
            offset as _,
        )
    })
}

#[cfg_attr(docsrs, doc(cfg(not(any(target_os = "macos", target_os = "ios")))))]
#[cfg(not(apple))]
#[inline]
pub fn pwritev(fd: RawFd, iov: &[IoVec], offset: u64) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::pwritev(
            fd,
            iov.as_ptr() as *const _,
            iov.len().try_into().unwrap_or(i32::MAX),
            offset as _,
        )
    })
}

#[cfg(linuxlike)]
bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[derive(Default)]
    pub struct RWFlags: libc::c_int {
        const HIPRI = sys::RWF_HIPRI;
        const DSYNC = sys::RWF_DSYNC;
        const SYNC = sys::RWF_SYNC;
        const NOWAIT = sys::RWF_NOWAIT;
        const APPEND = sys::RWF_APPEND;
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn preadv2(
    fd: RawFd,
    iov: &mut [IoVecMut],
    offset: Option<u64>,
    flags: RWFlags,
) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::syscall(
            libc::SYS_preadv2,
            fd,
            iov.as_ptr() as *const libc::iovec,
            iov.len().try_into().unwrap_or(i32::MAX),
            offset.unwrap_or(u64::MAX) as libc::off_t,
            flags.bits(),
        ) as isize
    })
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn pwritev2(fd: RawFd, iov: &[IoVec], offset: Option<u64>, flags: RWFlags) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::syscall(
            libc::SYS_pwritev2,
            fd,
            iov.as_ptr() as *const libc::iovec,
            iov.len().try_into().unwrap_or(i32::MAX),
            offset.unwrap_or(u64::MAX) as libc::off_t,
            flags.bits(),
        ) as isize
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iovecmut_advance() {
        let mut buf1 = [0, 1, 2];
        let mut buf2 = [3, 4, 5];
        let mut buf3 = [6, 7, 8];
        let mut bufs = [
            IoVecMut::new(&mut buf1),
            IoVecMut::new(&mut buf2),
            IoVecMut::new(&mut buf3),
        ];
        let mut vecs = &mut bufs[..];

        let mut buf4 = [4, 5];
        let mut buf5 = [6, 7, 8];

        vecs = IoVecMut::advance(vecs, 4);
        assert_eq!(vecs, &[IoVecMut::new(&mut buf4), IoVecMut::new(&mut buf5)]);

        let mut buf6 = [6, 7, 8];

        vecs = IoVecMut::advance(vecs, 2);
        assert_eq!(vecs, &[IoVecMut::new(&mut buf6)]);
    }

    #[test]
    fn test_iovec_advance() {
        let buf1 = [0, 1, 2];
        let buf2 = [3, 4, 5];
        let buf3 = [6, 7, 8];
        let mut bufs = [IoVec::new(&buf1), IoVec::new(&buf2), IoVec::new(&buf3)];
        let mut vecs = &mut bufs[..];

        vecs = IoVec::advance(vecs, 4);
        assert_eq!(vecs, &[IoVec::new(&[4, 5]), IoVec::new(&[6, 7, 8])]);

        vecs = IoVec::advance(vecs, 2);
        assert_eq!(vecs, &[IoVec::new(&[6, 7, 8])]);
    }

    #[test]
    fn test_iovec_mut() {
        let mut buf = [0, 1, 2];
        let mut vec = IoVecMut::new(&mut buf);
        vec.copy_from_slice(&[3, 4, 5]);
        assert_eq!(vec.deref(), &[3, 4, 5]);
    }

    #[test]
    fn test_iovec_from() {
        assert_eq!(
            IoVec::from(IoVecMut::new(&mut [b'a', b'b', b'c'])),
            IoVec::new(b"abc")
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_iovecs_debug() {
        assert_eq!(format!("{:?}", IoVec::new(&[0, 1, 2])), "IoVec([0, 1, 2])");

        assert_eq!(
            format!("{:?}", IoVecMut::new(&mut [0, 1, 2])),
            "IoVecMut([0, 1, 2])"
        );
    }

    #[test]
    fn test_readv_writev() {
        fn writev_all(f: &FileDesc, mut iov: &mut [IoVec]) {
            while !iov.is_empty() {
                let n = f.writev(iov).unwrap();
                iov = IoVec::advance(iov, n);
            }
        }

        fn readv_exact(f: &FileDesc, mut iov: &mut [IoVecMut]) {
            while !iov.is_empty() {
                let n = f.readv(iov).unwrap();
                debug_assert!(n > 0);
                iov = IoVecMut::advance(iov, n);
            }
        }

        let (r, w) = crate::pipe().unwrap();

        writev_all(&w, &mut [IoVec::new(b"abc"), IoVec::new(b"def")]);
        drop(w);

        let mut buf = [0; 6];
        let (buf1, buf2) = buf.split_at_mut(2);
        readv_exact(&r, &mut [IoVecMut::new(buf1), IoVecMut::new(buf2)]);

        assert_eq!(buf, *b"abcdef");
    }
}
