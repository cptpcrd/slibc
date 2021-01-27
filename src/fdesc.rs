#[cfg(feature = "std")]
use std::io::prelude::*;
#[cfg(feature = "std")]
use std::os::unix::prelude::*;

use crate::ffi::RawFd;
use crate::internal_prelude::*;

/// A helper struct that wraps a file descriptor and provides useful methods.
///
/// The file descriptor is automatically closed when the `FileDesc` struct is dropped.
#[derive(Debug)]
pub struct FileDesc(RawFd);

impl FileDesc {
    /// Create a new `FileDesc` wrapper around a raw file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must be valid and not in use elsewhere.
    #[inline]
    pub const unsafe fn new(fd: RawFd) -> Self {
        Self(fd)
    }

    /// Access the inner file descriptor.
    ///
    /// This serves a similar purpose to `AsRawFd::as_raw_fd()` in `std`; its purpose is to
    /// expose similar functionality in `#![no_std]` crates.
    ///
    /// The file descriptor is only valid as long as this object is in scope. It should NOT be
    /// closed or "consumed" by other interfaces; use [`into_fd()`](#method.into_fd) if it is going
    /// to be used for those purposes.
    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0
    }

    /// Take ownership of the inner file descriptor.
    ///
    /// This serves a similar purpose to `IntoRawFd::into_raw_fd()` in `std`; its purpose is to
    /// expose similar functionality in `#![no_std]` crates.
    ///
    /// After this method is called, the caller is responsible for closing the file descriptor.
    /// Failing to do so may result in resource leaks.
    #[inline]
    pub fn into_fd(self) -> RawFd {
        let fd = self.0;
        core::mem::forget(self);
        fd
    }

    /// Read data from the file descriptor into a buffer.
    ///
    /// This is the equivalent of `io::Read::read()` for use in `#![no_std]` crates.
    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        crate::read(self.0, buf)
    }

    /// Write data into the file descriptor from a buffer.
    ///
    /// The number of bytes successfully written is returned.
    ///
    /// This is the equivalent of `io::Write::write()` for use in `#![no_std]` crates.
    #[inline]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        crate::write(self.0, buf)
    }

    /// Attempt to write an entire buffer into the file descriptor.
    ///
    /// This is the equivalent of `io::Write::write_all()` for use in `#![no_std]` crates.
    pub fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(Error::from_code(libc::EIO)),
                Ok(n) => buf = &buf[n..],

                Err(e) if e.code() == libc::EINTR => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Read data from the file descriptor at a given offset into a buffer.
    ///
    /// This is the equivalent of `FileExt::read_at()`.
    #[inline]
    pub fn pread(&mut self, buf: &mut [u8], offset: u64) -> Result<usize> {
        crate::pread(self.0, buf, offset)
    }

    /// Write data into the file descriptor at a given offset from a buffer.
    ///
    /// This is the equivalent of `FileExt::write_at()`.
    #[inline]
    pub fn pwrite(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        crate::pwrite(self.0, buf, offset)
    }

    /// Get the close-on-exec status of the given file descriptor.
    #[inline]
    pub fn get_cloexec(&self) -> Result<bool> {
        Ok(crate::fcntl_getfd(self.0)? & libc::FD_CLOEXEC != 0)
    }

    /// Set the close-on-exec status of the given file descriptor.
    #[inline]
    pub fn set_cloexec(&mut self, cloexec: bool) -> Result<()> {
        let mut flags = crate::fcntl_getfd(self.0)?;

        #[allow(clippy::collapsible_if)]
        if cloexec {
            if flags & libc::FD_CLOEXEC == 0 {
                flags |= libc::FD_CLOEXEC;
            } else {
                return Ok(());
            }
        } else {
            if flags & libc::FD_CLOEXEC != 0 {
                flags &= !libc::FD_CLOEXEC;
            } else {
                return Ok(());
            }
        }

        crate::fcntl_setfd(self.0, flags)
    }

    /// Check whether this file descriptor refers to a terminal.
    #[inline]
    pub fn isatty(&self) -> Result<bool> {
        crate::isatty(self.0)
    }

    #[inline]
    pub fn seek(&mut self, pos: crate::SeekPos) -> Result<u64> {
        crate::lseek(self.0, pos)
    }

    #[inline]
    pub fn tell(&mut self) -> Result<u64> {
        crate::lseek(self.0, crate::SeekPos::Current(0))
    }

    /// Duplicate the file descriptor.
    ///
    /// The new file descriptor will *not* have its close-on-exec flag set. Use
    /// [`dup_cloexec()`](#method.dup_cloexec).
    #[inline]
    pub fn dup(&self) -> Result<Self> {
        crate::dup(self.0)
    }

    /// Duplicate the file descriptor.
    ///
    /// The new file descriptor will have its close-on-exec flag set.
    #[inline]
    pub fn dup_cloexec(&self) -> Result<Self> {
        crate::fcntl_dupfd_cloexec(self.0, 0)
    }

    /// Sync all data and metadata associated with this file to the disk.
    #[inline]
    pub fn sync_all(&self) -> Result<()> {
        crate::fsync(self.0)
    }

    /// Sync all data (not metadata, if possible) associated with this file to the disk.
    #[inline]
    pub fn sync_data(&self) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "dragonfly", target_os = "macos", target_os = "ios"))] {
                // No fdatasync()
                self.sync_all()?;
            } else {
                crate::fdatasync(self.0)?;
            }
        }

        Ok(())
    }

    /// Sync all modifications to the filesystem containing this file to the disk.
    ///
    /// See syncfs(2) for more details.
    #[cfg(target_os = "linux")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[inline]
    pub fn syncfs(&self) -> Result<()> {
        crate::syncfs(self.0)
    }
}

impl Drop for FileDesc {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

#[cfg(feature = "std")]
impl Read for FileDesc {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(crate::read(self.0, buf)?)
    }
}

#[cfg(feature = "std")]
impl Write for FileDesc {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(crate::write(self.0, buf)?)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
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

        Ok(crate::lseek(self.0, pos)?)
    }
}

#[cfg(feature = "std")]
impl FromRawFd for FileDesc {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(fd)
    }
}

#[cfg(feature = "std")]
impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0
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
        Self(f.into_raw_fd())
    }
}

#[cfg(feature = "std")]
impl From<FileDesc> for std::process::Stdio {
    #[inline]
    fn from(f: FileDesc) -> Self {
        unsafe { Self::from_raw_fd(f.into_raw_fd()) }
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
    fn test_into_fd() {
        let fdesc = get_fdesc();
        let fd = fdesc.into_fd();
        assert!(fd_valid(fd));
        unsafe {
            FileDesc::new(fd);
        }
    }

    #[test]
    fn test_cloexec() {
        let mut fdesc = get_fdesc();
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
