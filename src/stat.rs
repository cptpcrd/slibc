use crate::internal_prelude::*;
use crate::TimeSpec;

#[derive(Copy, Clone, Debug)]
pub struct Stat(libc::stat);

impl Stat {
    #[inline]
    pub fn dev(&self) -> u64 {
        self.0.st_dev as u64
    }

    #[inline]
    pub fn ino(&self) -> u64 {
        self.0.st_ino as u64
    }

    #[inline]
    pub fn mode(&self) -> u32 {
        self.0.st_mode as u32
    }

    #[inline]
    pub fn nlink(&self) -> u64 {
        self.0.st_nlink as u64
    }

    #[inline]
    pub fn uid(&self) -> libc::uid_t {
        self.0.st_uid
    }

    #[inline]
    pub fn gid(&self) -> libc::gid_t {
        self.0.st_gid
    }

    #[inline]
    pub fn rdev(&self) -> u64 {
        self.0.st_dev
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.0.st_size as u64
    }

    #[inline]
    pub fn atime(&self) -> Option<TimeSpec> {
        if self.0.st_atime == 0 && self.0.st_atime_nsec == 0 {
            None
        } else {
            Some(TimeSpec {
                tv_sec: self.0.st_atime,
                tv_nsec: self.0.st_atime_nsec as _,
            })
        }
    }

    #[inline]
    pub fn ctime(&self) -> Option<TimeSpec> {
        if self.0.st_ctime == 0 && self.0.st_ctime_nsec == 0 {
            None
        } else {
            Some(TimeSpec {
                tv_sec: self.0.st_ctime,
                tv_nsec: self.0.st_ctime_nsec as _,
            })
        }
    }

    #[inline]
    pub fn mtime(&self) -> Option<TimeSpec> {
        if self.0.st_mtime == 0 && self.0.st_mtime_nsec == 0 {
            None
        } else {
            Some(TimeSpec {
                tv_sec: self.0.st_mtime,
                tv_nsec: self.0.st_mtime_nsec as _,
            })
        }
    }

    #[inline]
    pub fn birthtime(&self) -> Option<TimeSpec> {
        #[cfg(any(
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "ios",
        ))]
        if self.0.st_birthtime != 0 && self.0.st_birthtime_nsec != 0 {
            return Some(TimeSpec {
                tv_sec: self.0.st_mtime,
                tv_nsec: self.0.st_mtime_nsec as _,
            });
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
