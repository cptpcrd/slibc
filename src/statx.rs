use crate::internal_prelude::*;

use core::fmt;

/// A timestamp from [`Statx`].
///
/// You can convert this to an `std::time::SystemTime` like so:
///
/// ```
/// # #[cfg(feature = "std")]
/// # {
/// # use slibc::*;
/// # use std::time::SystemTime;
/// assert_eq!(SystemTime::from(StatxTstamp::new(0, 0)), SystemTime::UNIX_EPOCH);
/// # }
/// ```
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[repr(C)]
#[non_exhaustive]
pub struct StatxTstamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    __reserved: i32,
}

impl StatxTstamp {
    /// Construct a new `StatxTstamp`.
    #[inline]
    pub fn new(tv_sec: i64, tv_nsec: u32) -> Self {
        Self {
            tv_sec,
            tv_nsec,
            __reserved: 0,
        }
    }
}

impl fmt::Debug for StatxTstamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StatxTstamp")
            .field("tv_sec", &self.tv_sec)
            .field("tv_nsec", &self.tv_nsec)
            .finish()
    }
}

impl From<StatxTstamp> for crate::TimeSpec {
    #[inline]
    fn from(t: StatxTstamp) -> Self {
        crate::TimeSpec {
            tv_sec: t.tv_sec as _,
            tv_nsec: t.tv_nsec as _,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<StatxTstamp> for std::time::SystemTime {
    #[inline]
    fn from(t: StatxTstamp) -> Self {
        Self::UNIX_EPOCH + std::time::Duration::new(t.tv_sec as u64, t.tv_nsec)
    }
}

bitflags::bitflags! {
    /// Represents the fields that may be returned in a [`Statx`] struct.
    ///
    /// This is used as the `mask` argument to [`statx()`] to tell the kernel which fields the
    /// caller wants, and the kernel sets [`Statx::mask`] to indicate which information has been
    /// returned. For example:
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # use slibc::*;
    /// let stx = statx(AT_FDCWD, ".", AtFlag::empty(), StatxMask::TYPE).unwrap();
    /// if stx.mask.contains(StatxMask::TYPE) {
    ///     // The file type information in `stx.mode` is correct
    /// }
    /// # }
    /// ```
    ///
    /// The implementation of `Default::default()` for this struct returns [`Self::BASIC_STATS`]
    /// (which is probably what most users will want):
    /// ```
    /// # use slibc::StatxMask;
    /// assert_eq!(StatxMask::default(), StatxMask::BASIC_STATS);
    /// ```
    ///
    /// See `statx(2)` for more information.
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[repr(transparent)]
    pub struct StatxMask: u32 {
        /// Retrieve the file type information in [`Statx::mode`].
        const TYPE = 0x1;
        /// Retrieve the access mode information in [`Statx::mode`].
        const MODE = 0x2;
        /// Retrieve [`Statx::nlink`].
        const NLINK = 0x4;
        /// Retrieve [`Statx::uid`].
        const UID = 0x8;
        /// Retrieve [`Statx::gid`].
        const GID = 0x10;
        /// Retrieve [`Statx::atime`].
        const ATIME = 0x20;
        /// Retrieve [`Statx::mtime`].
        const MTIME = 0x40;
        /// Retrieve [`Statx::ctime`].
        const CTIME = 0x80;
        /// Retrieve [`Statx::ino`].
        const INO = 0x100;
        /// Retrieve [`Statx::size`].
        const SIZE = 0x200;
        /// Retrieve [`Statx::blocks`].
        const BLOCKS = 0x400;

        /// Retrieve the following:
        ///
        /// [`Self::TYPE`], [`Self::MODE`], [`Self::NLINK`], [`Self::UID`], [`Self::GID`],
        /// [`Self::ATIME`], [`Self::MTIME`], [`Self::CTIME`], [`Self::INO`], [`Self::SIZE`],
        /// [`Self::BLOCKS`]
        const BASIC_STATS =
            Self::TYPE.bits
            | Self::MODE.bits
            | Self::NLINK.bits
            | Self::UID.bits
            | Self::GID.bits
            | Self::ATIME.bits
            | Self::MTIME.bits
            | Self::CTIME.bits
            | Self::INO.bits
            | Self::SIZE.bits
            | Self::BLOCKS.bits;

        /// Retrieve [`Statx::btime`].
        const BTIME = 0x800;
        /// Retrieve [`Statx::mnt_id`].
        const MNT_ID = 0x1000;
    }
}

impl Default for StatxMask {
    #[inline]
    fn default() -> Self {
        Self::BASIC_STATS
    }
}

bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[repr(transparent)]
    pub struct StatxAttr: u64 {
        /// The file is compressed by the filesystem.
        const COMPRESSED = 0x4;
        /// The file cannot be modified, deleted, renamed, or hard-linked to.
        const IMMUTABLE = 0x10;
        /// The file can only be opened in append mode.
        const APPEND = 0x20;
        /// The file is not a candidate for backup by programs such as `dump(8)`.
        const NODUMP = 0x40;
        /// The file is encrypted.
        const ENCRYPTED = 0x800;
        /// The file is an automount point.
        const AUTOMOUNT = 0x1000;
        /// The file is the root of a mountpoint.
        ///
        /// Added in Linux 5.8.
        const MOUNT_ROOT = 0x2000;
        /// The file is protected by "verity" (immutable, and all reads will be verified by a
        /// cryptographic hash).
        ///
        /// Added in Linux 5.5.
        const VERITY = 0x100000;
        /// The file is in DAX (CPU direct access) state.
        ///
        /// Added in Linux 5.8 (and fixed in Linux 5.10; it was accidentally given the same value
        /// as `MOUNT_ROOT`).
        const DAX = 0x200000;
    }
}

/// Extended information about a file.
///
/// **WARNING**: Before accessing any of the fields in this struct, check `self.mask` to ensure
/// they have been properly initialized!
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Statx {
    /// Indicates which fields are initialized (see [`StatxMask`] for more information).
    pub mask: StatxMask,
    /// The "preferred" block size for file I/O.
    pub blksize: u32,
    /// File attributes (see [`StatxAttr`] for more information).
    pub attributes: StatxAttr,
    /// The number of hard links to the file.
    pub nlink: u32,
    /// The UID of the file's owner.
    pub uid: u32,
    /// The GID of the file's owning group.
    pub gid: u32,
    /// The file's mode.
    ///
    /// This embeds the file type and the access mode (see [`Statx::file_type()`] and
    /// [`Statx::access_mode()`]). It also embeds several "flags" (see [`Statx::is_suid()`],
    /// [`Statx::is_sgid()`], and [`Statx::is_sticky()`]).
    pub mode: u16,
    __spare0: [u16; 1],
    /// The file's inode number.
    pub ino: u64,
    /// The file's size.
    pub size: u64,
    /// The number of blocks allocated to the file (in 512-byte units).
    pub blocks: u64,
    /// The attributes supported by the kernel and the filesystem containing the file.
    pub attributes_mask: StatxAttr,
    /// The last access time of the file.
    pub atime: StatxTstamp,
    /// The creation time ("birth time") of the file.
    ///
    /// This may not be supported by all filesystems.
    pub btime: StatxTstamp,
    /// The last status change time of the file.
    pub ctime: StatxTstamp,
    /// The last modification time of the file.
    pub mtime: StatxTstamp,
    /// The major ID of the device ID of this file (if it is a special file).
    pub rdev_major: u32,
    /// The minor ID of the device ID of this file (if it is a special file).
    pub rdev_minor: u32,
    /// The major ID of the device containing this file.
    pub dev_major: u32,
    /// The minor ID of the device containing this file.
    pub dev_minor: u32,
    /// The mount ID of the file.
    ///
    /// This was added in Linux 5.8; previous kernels will leave this field unset (and never
    /// include [`StatxMask::MNT_ID`] in `self.mask`).
    pub mnt_id: u64,
    __spare2: u64,
    __spare3: [u64; 12],
    // Note: To add a new field:
    // - Add it to this struct (removing padding fields as appropriate)
    // - Add the corresponding `StatxMask` flag (if applicable)
    // - Add it to the `Debug` implementation for this struct
    // - Add any necessary getters (like the `dev()` getter)
}

impl Statx {
    /// Equivalent to [`Stat::file_type()`](./struct.Stat.html#method.file_type).
    ///
    /// Note that this is only valid if the file type part of `self.mode` is initialized (i.e. if
    /// `self.mask` contains [`StatxMask::TYPE`]).
    #[inline]
    pub fn file_type(&self) -> crate::StatFileType {
        crate::StatFileType {
            mask: (self.mode as u32) & (libc::S_IFMT as u32),
        }
    }

    /// Equivalent to [`Stat::access_mode()`](./struct.Stat.html#method.access_mode).
    ///
    /// Note that this is only valid if the access mode part of `self.mode` is initialized (i.e. if
    /// `self.mask` contains [`StatxMask::MODE`]).
    #[inline]
    pub fn access_mode(&self) -> u32 {
        (self.mode & 0o777) as u32
    }

    /// Check whether this file is set-user-ID.
    ///
    /// Note that this is only valid if the access mode part of `self.mode` is initialized (i.e. if
    /// `self.mask` contains [`StatxMask::MODE`]).
    #[inline]
    pub fn is_suid(&self) -> bool {
        self.mode & libc::S_ISUID as u16 == libc::S_ISUID as u16
    }

    /// Check whether this file is set-group-ID.
    ///
    /// Note that this is only valid if the access mode part of `self.mode` is initialized (i.e. if
    /// `self.mask` contains [`StatxMask::MODE`]).
    #[inline]
    pub fn is_sgid(&self) -> bool {
        self.mode & libc::S_ISGID as u16 == libc::S_ISGID as u16
    }

    /// Check whether this file is sticky.
    ///
    /// For directories, this means users can only create files in the directory if they own the
    /// files.
    ///
    /// Note that this is only valid if the access mode part of `self.mode` is initialized (i.e. if
    /// `self.mask` contains [`StatxMask::MODE`]).
    #[inline]
    pub fn is_sticky(&self) -> bool {
        self.mode & libc::S_ISVTX as u16 == libc::S_ISVTX as u16
    }

    /// The device ID of this file (if it's a special file).
    ///
    /// This combines `self.rdev_major` and `self.rdev_minor` using `makedev()`.
    #[inline]
    pub fn rdev(&self) -> u64 {
        unsafe { libc::makedev(self.rdev_major as _, self.rdev_minor as _) as u64 }
    }

    /// The device ID of the device containing this file.
    ///
    /// This combines `self.dev_major` and `self.dev_minor` using `makedev()`.
    #[inline]
    pub fn dev(&self) -> u64 {
        unsafe { libc::makedev(self.dev_major as _, self.dev_minor as _) as u64 }
    }
}

impl fmt::Debug for Statx {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Statx")
            .field("mask", &self.mask)
            .field("blksize", &self.blksize)
            .field("attributes", &self.attributes)
            .field("nlink", &self.nlink)
            .field("uid", &self.uid)
            .field("gid", &self.gid)
            .field("mode", &self.mode)
            .field("ino", &self.ino)
            .field("size", &self.size)
            .field("blocks", &self.blocks)
            .field("attributes_mask", &self.attributes_mask)
            .field("atime", &self.atime)
            .field("btime", &self.btime)
            .field("ctime", &self.ctime)
            .field("mtime", &self.mtime)
            .field("rdev_major", &self.rdev_major)
            .field("rdev_minor", &self.rdev_minor)
            .field("dev_major", &self.dev_major)
            .field("dev_minor", &self.dev_minor)
            .field("mnt_id", &self.mnt_id)
            .finish()
    }
}

/// Retrieve extended information on the given file.
///
/// `dirfd` and `path` are the same as for [`fstatat()`](./fn.fstatat.html).
///
/// `flags` is also the same as for [`fstatat()`](./fn.fstatat.html), except that the following
/// flags can also be included: `AT_STATX_SYNC_AS_STAT`, `AT_STATX_FORCE_SYNC`,
/// `AT_STATX_DONT_SYNC`. See [`AtFlag`](./struct.AtFlag.html) and `statx(2)` for more information.
///
/// The purpose of `mask` is described in [`StatxMask`].
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
pub fn statx<P: AsPath>(
    dirfd: RawFd,
    path: P,
    flags: crate::AtFlag,
    mask: StatxMask,
) -> Result<Statx> {
    path.with_cstr(|path| {
        // The kernel will zero it out before copying in the data
        let mut buf = MaybeUninit::uninit();

        if unsafe {
            libc::syscall(
                libc::SYS_statx,
                dirfd,
                path.as_ptr(),
                flags.bits(),
                mask.bits() as libc::c_uint,
                buf.as_mut_ptr(),
            )
        } != 0
        {
            return Err(Error::last());
        }

        let mut buf: Statx = unsafe { buf.assume_init() };
        buf.mask.bits &= StatxMask::all().bits;
        buf.attributes.bits &= StatxAttr::all().bits;
        buf.attributes_mask.bits &= StatxAttr::all().bits;
        Ok(buf)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_same_statx_stat<P: AsPath + Clone>(path: P) {
        let st = crate::stat(path.clone()).unwrap();
        let stx = crate::statx(
            crate::AT_FDCWD,
            path,
            crate::AtFlag::empty(),
            StatxMask::BASIC_STATS,
        )
        .unwrap();

        assert!(stx.mask.contains(StatxMask::BASIC_STATS));

        assert_eq!(st.dev(), stx.dev());
        assert_eq!(st.ino(), stx.ino);
        assert_eq!(st.mode(), stx.mode as u32);
        assert_eq!(st.nlink(), stx.nlink as u64);
        assert_eq!(st.uid(), stx.uid);
        assert_eq!(st.gid(), stx.gid);
        assert_eq!(st.rdev(), stx.rdev());
        assert_eq!(st.size(), stx.size);
        assert_eq!(st.atime(), crate::TimeSpec::from(stx.atime));
        assert_eq!(st.ctime(), crate::TimeSpec::from(stx.ctime));
        assert_eq!(st.mtime(), crate::TimeSpec::from(stx.mtime));

        assert_eq!(st.file_type(), stx.file_type());
        assert_eq!(st.access_mode(), stx.access_mode());
        assert_eq!(st.is_suid(), stx.is_suid());
        assert_eq!(st.is_sgid(), stx.is_sgid());
        assert_eq!(st.is_sticky(), stx.is_sticky());
    }

    #[test]
    fn test_same_statx_stat() {
        check_same_statx_stat(CStr::from_bytes_with_nul(b"/\0").unwrap());
        check_same_statx_stat(CStr::from_bytes_with_nul(b".\0").unwrap());
        check_same_statx_stat(CStr::from_bytes_with_nul(b"/tmp\0").unwrap());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_same_statx_stat_current_exe() {
        check_same_statx_stat(
            &CString::new(std::env::current_exe().unwrap().into_os_string().into_vec()).unwrap(),
        );
    }

    #[test]
    fn test_statx_same() {
        macro_rules! check_statx_eq {
            ($stx1:expr, $stx2:expr) => {
                check_statx_eq!(
                    $stx1,
                    $stx2,
                    @,
                    blksize,
                    attributes,
                    nlink,
                    uid,
                    gid,
                    mode,
                    ino,
                    size,
                    blocks,
                    attributes_mask,
                    atime,
                    btime,
                    ctime,
                    mtime,
                    rdev_major,
                    rdev_minor,
                    dev_major,
                    dev_minor,
                    mnt_id,
                    @,
                    file_type,
                    access_mode,
                    is_suid,
                    is_sgid,
                    is_sticky,
                    rdev,
                    dev,
                )
            };

            ($stx1:expr, $stx2:expr, @, $($aname:ident,)+ @, $($fname:ident),+ $(,)?) => {{
                assert!($stx1.mask.contains(StatxMask::BASIC_STATS));
                assert!($stx2.mask.contains(StatxMask::BASIC_STATS));

                $(
                    assert_eq!(
                        $stx1.$aname, $stx2.$aname, concat!(stringify!($aname), " mismatch")
                    );
                )+

                $(
                    assert_eq!(
                        $stx1.$fname(), $stx2.$fname(), concat!(stringify!($fname), " mismatch")
                    );
                )+
            }};
        }

        fn check_same_stats<P: AsPath + Clone>(path: P, mask: libc::mode_t) {
            let stx1 = statx(
                crate::AT_FDCWD,
                path.clone(),
                crate::AtFlag::empty(),
                StatxMask::BASIC_STATS,
            )
            .unwrap();

            let stx2 = crate::open(path, crate::OFlag::O_RDONLY, 0)
                .unwrap()
                .statx(crate::AtFlag::empty(), StatxMask::BASIC_STATS)
                .unwrap();

            assert_eq!(stx1.file_type(), crate::StatFileType { mask: mask as _ });

            check_statx_eq!(stx1, stx2);
        }

        check_same_stats(CStr::from_bytes_with_nul(b"/\0").unwrap(), libc::S_IFDIR);
        check_same_stats(CStr::from_bytes_with_nul(b".\0").unwrap(), libc::S_IFDIR);
        check_same_stats(
            CStr::from_bytes_with_nul(b"/tmp/\0").unwrap(),
            libc::S_IFDIR,
        );

        #[cfg(feature = "std")]
        check_same_stats(std::env::current_exe().unwrap(), libc::S_IFREG);
    }

    #[test]
    fn test_statx_error() {
        macro_rules! check_err {
            ($dirfd:expr, $name:expr, $eno:expr) => {
                assert_eq!(
                    crate::statx(
                        $dirfd,
                        CStr::from_bytes_with_nul($name).unwrap(),
                        crate::AtFlag::empty(),
                        StatxMask::BASIC_STATS,
                    )
                    .unwrap_err(),
                    $eno,
                );
            };
        }

        check_err!(crate::AT_FDCWD, b"/NOEXIST\0", Errno::ENOENT);
        check_err!(crate::AT_FDCWD, b"/bin/sh/\0", Errno::ENOTDIR);
        check_err!(crate::AT_FDCWD, b"\0", Errno::ENOENT);
        check_err!(-1, b".\0", Errno::EBADF);
    }

    /// Tests the layout of `Statx` against `libc::statx` to make sure it's correct.
    #[cfg(target_env = "gnu")]
    #[test]
    fn test_statx_layout() {
        // Get random data
        let mut buf = [0u8; core::mem::size_of::<libc::statx>()];
        getrandom::getrandom(&mut buf).unwrap();

        let stx1: libc::statx = unsafe { core::mem::transmute(buf) };
        let stx2: Statx = unsafe { core::mem::transmute(buf) };

        assert_eq!(stx1.stx_mask, stx2.mask.bits);
        assert_eq!(stx1.stx_blksize, stx2.blksize);
        assert_eq!(stx1.stx_attributes, stx2.attributes.bits);
        assert_eq!(stx1.stx_nlink, stx2.nlink);
        assert_eq!(stx1.stx_uid, stx2.uid);
        assert_eq!(stx1.stx_gid, stx2.gid);
        assert_eq!(stx1.stx_mode, stx2.mode);
        assert_eq!(stx1.stx_ino, stx2.ino);
        assert_eq!(stx1.stx_size, stx2.size);
        assert_eq!(stx1.stx_blocks, stx2.blocks);
        assert_eq!(stx1.stx_attributes_mask, stx2.attributes_mask.bits);

        assert_eq!(stx1.stx_atime.tv_sec, stx2.atime.tv_sec);
        assert_eq!(stx1.stx_atime.tv_nsec, stx2.atime.tv_nsec);
        assert_eq!(stx1.stx_btime.tv_sec, stx2.btime.tv_sec);
        assert_eq!(stx1.stx_btime.tv_nsec, stx2.btime.tv_nsec);
        assert_eq!(stx1.stx_ctime.tv_sec, stx2.ctime.tv_sec);
        assert_eq!(stx1.stx_ctime.tv_nsec, stx2.ctime.tv_nsec);
        assert_eq!(stx1.stx_mtime.tv_sec, stx2.mtime.tv_sec);
        assert_eq!(stx1.stx_mtime.tv_nsec, stx2.mtime.tv_nsec);

        assert_eq!(stx1.stx_rdev_major, stx2.rdev_major);
        assert_eq!(stx1.stx_rdev_minor, stx2.rdev_minor);
        assert_eq!(stx1.stx_dev_major, stx2.dev_major);
        assert_eq!(stx1.stx_dev_minor, stx2.dev_minor);
    }

    #[test]
    fn test_statxmask_default() {
        assert_eq!(StatxMask::default(), StatxMask::BASIC_STATS);
    }

    #[test]
    fn test_from_statxtstamp() {
        let epoch = StatxTstamp::new(0, 0);
        let later = StatxTstamp::new(1234, 5678);

        assert_eq!(
            crate::TimeSpec::from(epoch),
            crate::TimeSpec {
                tv_sec: 0,
                tv_nsec: 0
            }
        );
        assert_eq!(
            crate::TimeSpec::from(later),
            crate::TimeSpec {
                tv_sec: 1234,
                tv_nsec: 5678
            }
        );

        #[cfg(feature = "std")]
        {
            use std::time::{Duration, SystemTime};

            assert_eq!(SystemTime::from(epoch), SystemTime::UNIX_EPOCH);
            assert_eq!(
                SystemTime::from(later),
                SystemTime::UNIX_EPOCH + Duration::new(1234, 5678)
            );
        }
    }
}
