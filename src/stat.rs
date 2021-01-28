use crate::internal_prelude::*;
use crate::{AtFlag, TimeSpec};

#[derive(Copy, Clone, Debug)]
pub struct Stat(libc::stat);

impl Stat {
    /// Get the device ID of the device containing this file.
    #[inline]
    pub fn dev(&self) -> u64 {
        self.0.st_dev as u64
    }

    /// Get his file's inode number.
    #[inline]
    pub fn ino(&self) -> u64 {
        self.0.st_ino as u64
    }

    /// Get this file's mode.
    ///
    /// This embeds the file type and the access mode.
    #[inline]
    pub fn mode(&self) -> u32 {
        self.0.st_mode as u32
    }

    /// Get the number of hardlinks to this file.
    #[inline]
    pub fn nlink(&self) -> u64 {
        self.0.st_nlink as u64
    }

    /// Get the user ID of the owner of this file.
    #[inline]
    pub fn uid(&self) -> libc::uid_t {
        self.0.st_uid
    }

    /// Get the group ID of the owning group of this file.
    #[inline]
    pub fn gid(&self) -> libc::gid_t {
        self.0.st_gid
    }

    /// Get the device ID of this file (if it is a special file).
    #[inline]
    pub fn rdev(&self) -> u64 {
        self.0.st_rdev as u64
    }

    /// Get the size of this file.
    #[inline]
    pub fn size(&self) -> u64 {
        self.0.st_size as u64
    }

    /// Get the last access time of this file (if available).
    #[inline]
    pub fn atime(&self) -> TimeSpec {
        TimeSpec {
            tv_sec: self.0.st_atime,
            #[cfg(target_os = "netbsd")]
            tv_nsec: self.0.st_atimensec as _,
            #[cfg(not(target_os = "netbsd"))]
            tv_nsec: self.0.st_atime_nsec as _,
        }
    }

    /// Get the last status change time of this file (if available).
    #[inline]
    pub fn ctime(&self) -> TimeSpec {
        TimeSpec {
            tv_sec: self.0.st_ctime,
            #[cfg(target_os = "netbsd")]
            tv_nsec: self.0.st_ctimensec as _,
            #[cfg(not(target_os = "netbsd"))]
            tv_nsec: self.0.st_ctime_nsec as _,
        }
    }

    /// Get the last modification time of this file (if available).
    #[inline]
    pub fn mtime(&self) -> TimeSpec {
        TimeSpec {
            tv_sec: self.0.st_mtime,
            #[cfg(target_os = "netbsd")]
            tv_nsec: self.0.st_mtimensec as _,
            #[cfg(not(target_os = "netbsd"))]
            tv_nsec: self.0.st_mtime_nsec as _,
        }
    }

    /// Get the creation time of this file (if available).
    ///
    /// This is currently only available on the following platforms:
    ///
    /// - macOS
    /// - FreeBSD
    /// - OpenBSD
    /// - NetBSD
    #[inline]
    pub fn birthtime(&self) -> Option<TimeSpec> {
        #[cfg(any(
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "ios",
        ))]
        {
            let tv_sec = self.0.st_birthtime;

            #[cfg(target_os = "netbsd")]
            let tv_nsec = self.0.st_birthtimensec as _;
            #[cfg(not(target_os = "netbsd"))]
            let tv_nsec = self.0.st_birthtime_nsec as _;

            if tv_sec > 0 {
                return Some(TimeSpec { tv_sec, tv_nsec });
            }
        }

        None
    }
}

impl From<Stat> for libc::stat {
    #[inline]
    fn from(s: Stat) -> libc::stat {
        s.0
    }
}

#[inline]
pub fn stat<P: AsPath>(path: P) -> Result<Stat> {
    let mut buf = MaybeUninit::uninit();
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { libc::stat(path.as_ptr(), buf.as_mut_ptr()) })
    })?;
    Ok(Stat(unsafe { buf.assume_init() }))
}

#[inline]
pub fn fstat(fd: RawFd) -> Result<Stat> {
    let mut buf = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::fstat(fd, buf.as_mut_ptr()) })?;
    Ok(Stat(unsafe { buf.assume_init() }))
}

#[inline]
pub fn lstat<P: AsPath>(path: P) -> Result<Stat> {
    let mut buf = MaybeUninit::uninit();
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { libc::lstat(path.as_ptr(), buf.as_mut_ptr()) })
    })?;
    Ok(Stat(unsafe { buf.assume_init() }))
}

#[inline]
pub fn fstatat<P: AsPath>(dfd: RawFd, path: P, flags: AtFlag) -> Result<Stat> {
    let mut buf = MaybeUninit::uninit();
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::fstatat(dfd, path.as_ptr(), buf.as_mut_ptr(), flags.bits())
        })
    })?;
    Ok(Stat(unsafe { buf.assume_init() }))
}
