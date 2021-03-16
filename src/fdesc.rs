#[cfg(feature = "std")]
use std::io::prelude::*;
#[cfg(feature = "std")]
use std::os::unix::prelude::*;

use crate::ffi::RawFd;
use crate::internal_prelude::*;

/// A helper struct that wraps a file descriptor and provides useful methods.
///
/// The file descriptor is automatically closed when the `FileDesc` struct is dropped.
#[must_use = "either explicitly `drop()` this FileDesc to close the file descriptor or `.forget()` it to leave it open"]
#[derive(Debug)]
pub struct FileDesc(BorrowedFd);

impl FileDesc {
    /// Create a new `FileDesc` wrapper around a raw file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must be valid and not in use elsewhere.
    #[inline]
    pub const unsafe fn new(fd: RawFd) -> Self {
        Self(BorrowedFd::new(fd))
    }

    /// Take ownership of the inner file descriptor.
    ///
    /// This serves a similar purpose to `IntoRawFd::into_raw_fd()` in `std`; its purpose is to
    /// expose similar functionality in `#![no_std]` crates.
    ///
    /// After this method is called, the caller is responsible for closing the file descriptor.
    /// Failing to do so may result in resource leaks.
    #[must_use = "use `.forget()` if you don't need the inner file descriptor"]
    #[inline]
    pub fn into_fd(self) -> RawFd {
        let fd = self.fd();
        core::mem::forget(self);
        fd
    }

    /// "Forget" about this file descriptor without closing it.
    ///
    /// WARNING: This may result in file descriptor leaks, especially since the file descriptor is
    /// not returned as with [`FileDesc::into_fd()`].
    ///
    /// `fdesc.forget()` is equivalent to `std::mem::forget(file)`.
    #[inline]
    pub fn forget(self) {
        core::mem::forget(self);
    }
}

impl Drop for FileDesc {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd());
        }
    }
}

#[cfg(feature = "std")]
impl Read for FileDesc {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut self.0, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        Read::read_vectored(&mut self.0, bufs)
    }
}

#[cfg(feature = "std")]
impl Write for FileDesc {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut self.0, buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        Write::write_vectored(&mut self.0, bufs)
    }
}

#[cfg(feature = "std")]
impl Seek for FileDesc {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        use crate::SeekPos;
        use std::io::SeekFrom;

        let pos = match pos {
            SeekFrom::Start(off) => SeekPos::Start(off),
            SeekFrom::End(off) => SeekPos::End(off),
            SeekFrom::Current(off) => SeekPos::Current(off),
        };

        Ok(crate::lseek(self.fd(), pos)?)
    }
}

#[cfg(feature = "std")]
impl FromRawFd for FileDesc {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::new(fd)
    }
}

#[cfg(feature = "std")]
impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for FileDesc {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_fd()
    }
}

#[cfg(feature = "std")]
impl From<std::fs::File> for FileDesc {
    #[inline]
    fn from(f: std::fs::File) -> Self {
        unsafe { Self::new(f.into_raw_fd()) }
    }
}

#[cfg(feature = "std")]
impl From<FileDesc> for std::process::Stdio {
    #[inline]
    fn from(f: FileDesc) -> Self {
        unsafe { Self::from_raw_fd(f.into_raw_fd()) }
    }
}

impl AsRef<BorrowedFd> for FileDesc {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self
    }
}

impl AsMut<BorrowedFd> for FileDesc {
    #[inline]
    fn as_mut(&mut self) -> &mut BorrowedFd {
        self
    }
}

impl core::ops::Deref for FileDesc {
    type Target = BorrowedFd;

    #[inline]
    fn deref(&self) -> &BorrowedFd {
        &self.0
    }
}

impl core::ops::DerefMut for FileDesc {
    #[inline]
    fn deref_mut(&mut self) -> &mut BorrowedFd {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_fdesc() -> FileDesc {
        crate::pipe().unwrap().0
    }

    fn fd_valid(fd: RawFd) -> bool {
        crate::fcntl_getfd(fd).is_ok()
    }

    #[test]
    fn test_into_fd_drop_forget() {
        let fdesc = get_fdesc();
        let fd = fdesc.into_fd();
        assert!(fd_valid(fd));

        unsafe {
            FileDesc::new(fd).forget();
        }
        assert!(fd_valid(fd));

        unsafe {
            drop(FileDesc::new(fd));
        }
    }

    #[test]
    fn test_cloexec() {
        let fdesc = get_fdesc();
        assert!(!fdesc.get_cloexec().unwrap());

        fdesc.set_cloexec(false).unwrap();
        assert!(!fdesc.get_cloexec().unwrap());
        fdesc.set_cloexec(false).unwrap();
        assert!(!fdesc.get_cloexec().unwrap());

        fdesc.set_cloexec(true).unwrap();
        assert!(fdesc.get_cloexec().unwrap());
        fdesc.set_cloexec(true).unwrap();
        assert!(fdesc.get_cloexec().unwrap());

        fdesc.set_cloexec(false).unwrap();
        assert!(!fdesc.get_cloexec().unwrap());
    }

    #[test]
    fn test_dup() {
        let fdesc = get_fdesc();

        let fdesc2 = fdesc.dup().unwrap();
        assert!(!fdesc2.get_cloexec().unwrap());

        let fdesc3 = fdesc.dup_cloexec().unwrap();
        assert!(fdesc3.get_cloexec().unwrap());
    }
}
