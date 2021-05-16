use core::fmt;

use crate::internal_prelude::*;
use crate::{AtFlag, TimeSpec};

/// Represents the file type mask from a `Stat` structure. Can be used to determine the file type.
///
/// Note: This is derived from [`Stat::mode()`] such that:
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// # use slibc::*;
/// let st = stat(".").unwrap();
/// assert_eq!(st.file_type().mask, st.mode() & (libc::S_IFMT as u32));
/// # }
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct StatFileType {
    pub mask: u32,
}

impl StatFileType {
    #[inline]
    pub fn is_dir(&self) -> bool {
        self.mask == libc::S_IFDIR as u32
    }

    #[inline]
    pub fn is_file(&self) -> bool {
        self.mask == libc::S_IFREG as u32
    }

    #[inline]
    pub fn is_symlink(&self) -> bool {
        self.mask == libc::S_IFLNK as u32
    }

    #[inline]
    pub fn is_block_device(&self) -> bool {
        self.mask == libc::S_IFBLK as u32
    }

    #[inline]
    pub fn is_char_device(&self) -> bool {
        self.mask == libc::S_IFCHR as u32
    }

    #[inline]
    pub fn is_fifo(&self) -> bool {
        self.mask == libc::S_IFIFO as u32
    }

    #[inline]
    pub fn is_socket(&self) -> bool {
        self.mask == libc::S_IFSOCK as u32
    }
}

impl fmt::Debug for StatFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = if self.is_dir() {
            "Directory"
        } else if self.is_file() {
            "File"
        } else if self.is_symlink() {
            "Symlink"
        } else if self.is_block_device() {
            "BlockDevice"
        } else if self.is_char_device() {
            "CharacterDevice"
        } else if self.is_fifo() {
            "Fifo"
        } else if self.is_socket() {
            "Socket"
        } else {
            "Unknown"
        };

        f.write_str(s)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Stat(libc::stat);

impl Stat {
    /// Get the device ID of the device containing this file.
    #[inline]
    pub fn dev(&self) -> u64 {
        self.0.st_dev as u64
    }

    /// Get this file's inode number.
    #[inline]
    pub fn ino(&self) -> u64 {
        self.0.st_ino as u64
    }

    /// Get this file's mode.
    ///
    /// This embeds the file type and the access mode (see [`Stat::file_type()`] and
    /// [`Stat::access_mode()`]). It also embeds several "flags" (see [`Stat::is_suid()`],
    /// [`Stat::is_sgid()`], and [`Stat::is_sticky()`]).
    #[inline]
    pub fn mode(&self) -> u32 {
        self.0.st_mode as u32
    }

    /// Get the file type information associated with this `Stat` structure.
    ///
    /// See [`StatFileType`] for more information.
    #[inline]
    pub fn file_type(&self) -> StatFileType {
        StatFileType {
            mask: self.mode() & (libc::S_IFMT as u32),
        }
    }

    /// Check whether this file is set-user-ID.
    #[inline]
    pub fn is_suid(&self) -> bool {
        self.mode() & libc::S_ISUID as u32 == libc::S_ISUID as u32
    }

    /// Check whether this file is set-group-ID.
    #[inline]
    pub fn is_sgid(&self) -> bool {
        self.mode() & libc::S_ISGID as u32 == libc::S_ISGID as u32
    }

    /// Check whether this file is sticky.
    ///
    /// For directories, this means users can only create files in the directory if they own the
    /// files.
    #[inline]
    pub fn is_sticky(&self) -> bool {
        self.mode() & libc::S_ISVTX as u32 == libc::S_ISVTX as u32
    }

    /// Get the access mode associated with this `Stat` structure.
    ///
    /// This is the portion of [`Stat::mode()`] that does not embed the file type.
    #[inline]
    pub fn access_mode(&self) -> u32 {
        self.mode() & 0o777
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
    ///
    /// On Linux, this can be retrieved using [`statx()`](./fn.statx.html) (see
    /// [`Statx::btime`](./struct.Statx.html#structfield.btime)).
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

impl AsRef<libc::stat> for Stat {
    #[inline]
    fn as_ref(&self) -> &libc::stat {
        &self.0
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

#[inline]
pub fn mkdir<P: AsPath>(path: P, mode: u32) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::mkdir(path.as_ptr(), mode as _) }))
}

#[inline]
pub fn mkdirat<P: AsPath>(dfd: RawFd, path: P, mode: u32) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::mkdirat(dfd, path.as_ptr(), mode as _) }))
}

#[inline]
pub fn umask(mask: u32) -> u32 {
    unsafe { libc::umask(mask as _) as u32 }
}

pub const UTIMENS_NOW: TimeSpec = TimeSpec {
    tv_sec: 0,
    tv_nsec: libc::UTIME_NOW,
};
pub const UTIMENS_OMIT: TimeSpec = TimeSpec {
    tv_sec: 0,
    tv_nsec: libc::UTIME_OMIT,
};

/// Update the timestamps of the specified file.
///
/// The file is identified by the combination of `dfd`, `path`, and `flags` in the same manner as
/// the other `*at()` syscalls.
///
/// The new access timestamp is specified by `atime`, and the new modification timestamp is
/// specified by `mtime`.
///
/// To set either timestamp to the current time, specify [`UTIMENS_NOW`] for the corresponding
/// argument. To leave either timestamp unchanged, specify [`UTIMENS_OMIT`].
#[inline]
pub fn utimensat<P: AsPath>(
    dfd: RawFd,
    path: P,
    atime: TimeSpec,
    mtime: TimeSpec,
    flags: AtFlag,
) -> Result<()> {
    let times = [atime, mtime];

    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::utimensat(dfd, path.as_ptr(), times.as_ptr() as *const _, flags.bits())
        })
    })
}

/// Update the timestamps of file specified by the given file descriptor
///
/// This is identical to [`utimensat()`], except that the file is specified by an open file
/// descriptor.
#[inline]
pub fn futimens(fd: RawFd, atime: TimeSpec, mtime: TimeSpec) -> Result<()> {
    let times = [atime, mtime];
    Error::unpack_nz(unsafe { libc::futimens(fd, times.as_ptr() as *const _) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filetype_is() {
        macro_rules! check {
            ($meth:ident, $true_mask:ident, $($false_mask:ident),+ $(,)?) => {{
                assert!(StatFileType { mask: libc::$true_mask as u32 }.$meth());

                $(
                    assert!(!StatFileType { mask: libc::$false_mask as u32 }.$meth());
                )+
            }};
        }

        check!(is_file, S_IFREG, S_IFDIR, S_IFLNK, S_IFBLK, S_IFCHR, S_IFIFO, S_IFSOCK);
        check!(is_dir, S_IFDIR, S_IFREG, S_IFLNK, S_IFBLK, S_IFCHR, S_IFIFO, S_IFSOCK);
        check!(is_symlink, S_IFLNK, S_IFREG, S_IFDIR, S_IFBLK, S_IFCHR, S_IFIFO, S_IFSOCK);
        check!(
            is_block_device,
            S_IFBLK,
            S_IFREG,
            S_IFDIR,
            S_IFLNK,
            S_IFCHR,
            S_IFIFO,
            S_IFSOCK
        );
        check!(
            is_char_device,
            S_IFCHR,
            S_IFREG,
            S_IFDIR,
            S_IFLNK,
            S_IFBLK,
            S_IFIFO,
            S_IFSOCK
        );
        check!(is_fifo, S_IFIFO, S_IFREG, S_IFDIR, S_IFLNK, S_IFBLK, S_IFCHR, S_IFSOCK);
        check!(is_socket, S_IFSOCK, S_IFREG, S_IFDIR, S_IFLNK, S_IFBLK, S_IFCHR, S_IFIFO);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_filetype_debug() {
        macro_rules! check {
            ($mask:expr, $s:expr) => {
                assert_eq!(format!("{:?}", StatFileType { mask: $mask as u32 }), $s);
            };
        }

        check!(libc::S_IFDIR, "Directory");
        check!(libc::S_IFREG, "File");
        check!(libc::S_IFLNK, "Symlink");
        check!(libc::S_IFBLK, "BlockDevice");
        check!(libc::S_IFCHR, "CharacterDevice");
        check!(libc::S_IFIFO, "Fifo");
        check!(libc::S_IFSOCK, "Socket");
        check!(u32::MAX, "Unknown");
    }

    #[test]
    fn test_stats_same() {
        macro_rules! check_stat_eq {
            ($st1:expr, $st2:expr) => {
                check_stat_eq!(
                    $st1,
                    $st2,
                    @,
                    dev,
                    ino,
                    mode,
                    file_type,
                    is_suid,
                    is_sgid,
                    is_sticky,
                    access_mode,
                    nlink,
                    uid,
                    gid,
                    rdev,
                    size,
                    atime,
                    ctime,
                    mtime,
                    birthtime,
                )
            };

            ($st1:expr, $st2:expr, @, $($name:ident),+ $(,)?) => {{
                $(
                    assert_eq!(
                        $st1.$name(), $st2.$name(), concat!(stringify!($name), " mismatch")
                    );
                )+
            }};
        }

        fn check_same_stats<P: AsPath + Clone>(path: P, mask: libc::mode_t) {
            let st1 = crate::stat(path.clone()).unwrap();
            let st2 = crate::lstat(path.clone()).unwrap();
            let st3 =
                crate::fstatat(crate::AT_FDCWD, path.clone(), crate::AtFlag::empty()).unwrap();
            let st4 = crate::open(path, crate::OFlag::O_RDONLY, 0)
                .unwrap()
                .stat()
                .unwrap();

            assert_eq!(st1.file_type(), StatFileType { mask: mask as _ });

            check_stat_eq!(st1, st2);
            check_stat_eq!(st1, st3);
            check_stat_eq!(st1, st4);
        }

        check_same_stats(CStr::from_bytes_with_nul(b"/\0").unwrap(), libc::S_IFDIR);
        check_same_stats(CStr::from_bytes_with_nul(b".\0").unwrap(), libc::S_IFDIR);
        check_same_stats(
            CStr::from_bytes_with_nul(b"/bin/\0").unwrap(),
            libc::S_IFDIR,
        );

        #[cfg(feature = "std")]
        check_same_stats(std::env::current_exe().unwrap(), libc::S_IFREG);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_umask_create() {
        let old_mask = umask(0o077);

        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdir_fd = crate::open(
            tmpdir.path(),
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
            0,
        )
        .unwrap();

        let tmpdir_a = tmpdir.path().join("a");
        mkdir(&tmpdir_a, 0o777).unwrap();
        assert_eq!(stat(&tmpdir_a).unwrap().access_mode(), 0o700);

        mkdirat(tmpdir_fd.fd(), "b", 0o777).unwrap();
        assert_eq!(
            fstatat(tmpdir_fd.fd(), "b", crate::AtFlag::empty())
                .unwrap()
                .access_mode(),
            0o700
        );

        drop(
            crate::openat(
                tmpdir_fd.fd(),
                "c",
                OFlag::O_WRONLY | OFlag::O_CLOEXEC | OFlag::O_CREAT | OFlag::O_EXCL,
                0o666,
            )
            .unwrap(),
        );
        assert_eq!(
            fstatat(tmpdir_fd.fd(), "c", crate::AtFlag::empty())
                .unwrap()
                .access_mode(),
            0o600
        );

        assert_eq!(umask(old_mask), 0o077);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_utimensat_futimens() {
        use std::os::unix::prelude::*;

        let tmpfile_open = tempfile::NamedTempFile::new().unwrap();
        let tmpfile = tmpfile_open.as_ref();

        let t1 = TimeSpec {
            tv_sec: 1,
            tv_nsec: 1,
        };
        let t2 = TimeSpec {
            tv_sec: 2,
            tv_nsec: 2,
        };

        utimensat(libc::AT_FDCWD, tmpfile, t1, t2, AtFlag::empty()).unwrap();
        let st = crate::stat(tmpfile).unwrap();
        assert_eq!(st.atime(), t1);
        assert_eq!(st.mtime(), t2);

        utimensat(libc::AT_FDCWD, tmpfile, UTIMENS_OMIT, t1, AtFlag::empty()).unwrap();
        let st = crate::stat(tmpfile).unwrap();
        assert_eq!(st.atime(), t1);
        assert_eq!(st.mtime(), t1);

        utimensat(libc::AT_FDCWD, tmpfile, t2, UTIMENS_OMIT, AtFlag::empty()).unwrap();
        let st = crate::stat(tmpfile).unwrap();
        assert_eq!(st.atime(), t2);
        assert_eq!(st.mtime(), t1);

        futimens(tmpfile_open.as_raw_fd(), t1, t2).unwrap();
        let st = crate::stat(tmpfile).unwrap();
        assert_eq!(st.atime(), t1);
        assert_eq!(st.mtime(), t2);
    }
}
