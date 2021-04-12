use crate::internal_prelude::*;

/// This ensures that `sys::statfs` is the same size as `libc::statfs`.
const _STATFS_SIZE_CHECK: sys::statfs =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<libc::statfs>()]) };

#[allow(non_camel_case_types)]
#[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
#[cfg(target_os = "android")]
pub type fsid_t = libc::__fsid_t;
#[allow(non_camel_case_types)]
#[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
#[cfg(not(target_os = "android"))]
pub type fsid_t = libc::fsid_t;

/// Filesystem statistics.
///
/// This is returned by [`statfs()`] or [`fstatfs()`].
///
/// The contents of this structure are platform-specific.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Statfs(sys::statfs);

impl Statfs {
    /// Create a `Statfs` structure with the contents zeroed.
    ///
    /// This means most of the methods will return zero or an "empty" value.
    ///
    /// This is intended to be used for `getfsstat()` on macOS/\*BSD.
    #[inline]
    pub fn zeroed() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }

    /// Filesystem mount flags
    #[inline]
    pub fn flags(&self) -> StatfsFlags {
        StatfsFlags::from_bits_truncate(self.0.f_flags as u64)
    }

    /// Total data blocks in filesystem
    #[inline]
    pub fn blocks(&self) -> u64 {
        self.0.f_blocks as u64
    }

    /// Filesystem block size
    #[inline]
    pub fn bsize(&self) -> u64 {
        self.0.f_bsize as u64
    }

    /// Free blocks in filesystem
    #[inline]
    pub fn bfree(&self) -> u64 {
        self.0.f_bfree as u64
    }

    /// Free blocks available to unprivileged users
    #[inline]
    pub fn bavail(&self) -> u64 {
        self.0.f_bavail as u64
    }

    /// Total inodes in filesystem
    #[inline]
    pub fn files(&self) -> u64 {
        self.0.f_files as u64
    }

    /// Free inodes in filesystem
    #[inline]
    pub fn ffree(&self) -> u64 {
        self.0.f_ffree as u64
    }

    /// The filesystem ID
    #[inline]
    pub fn fsid(&self) -> fsid_t {
        self.0.f_fsid
    }

    /// The filesystem type
    #[cfg_attr(docsrs, doc(cfg(not(target_os = "openbsd"))))]
    #[cfg(not(target_os = "openbsd"))]
    #[inline]
    pub fn fstype(&self) -> FsType {
        FsType(self.0.f_type)
    }

    /// The maximum length of filenames on this filesystem
    #[cfg(not(any(target_os = "dragonfly", apple)))]
    #[cfg_attr(
        docsrs,
        doc(cfg(not(any(target_os = "dragonfly", target_os = "macos", target_os = "ios"))))
    )]
    #[inline]
    pub fn namelen(&self) -> usize {
        #[cfg(linuxlike)]
        let len = self.0.f_namelen;
        #[cfg(bsd)]
        let len = self.0.f_namemax;

        len as usize
    }

    #[cfg(target_os = "openbsd")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "openbsd")))]
    #[inline]
    pub fn ctime(&self) -> u64 {
        self.0.f_ctime
    }

    #[cfg(target_os = "openbsd")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "openbsd")))]
    #[inline]
    pub fn mntfromspec(&self) -> &OsStr {
        util::osstr_from_buf(util::cvt_char_buf(&self.0.f_mntfromspec))
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
impl Statfs {
    /// The fragment size of this filesystem
    #[inline]
    pub fn frsize(&self) -> usize {
        self.0.f_frsize as usize
    }
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "macos",
    target_os = "ios"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos",
        target_os = "ios"
    )))
)]
impl Statfs {
    #[inline]
    pub fn iosize(&self) -> u64 {
        self.0.f_iosize as u64
    }

    /// User who mounted the filesystem
    #[inline]
    pub fn owner(&self) -> libc::uid_t {
        self.0.f_owner
    }

    #[inline]
    pub fn fstypename(&self) -> &OsStr {
        util::osstr_from_buf(util::cvt_char_buf(&self.0.f_fstypename))
    }

    #[inline]
    pub fn mnttoname(&self) -> &OsStr {
        util::osstr_from_buf(util::cvt_char_buf(&self.0.f_mntonname))
    }

    #[inline]
    pub fn mntfromname(&self) -> &OsStr {
        util::osstr_from_buf(util::cvt_char_buf(&self.0.f_mntfromname))
    }
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd")))
)]
impl Statfs {
    /// Synchronous writes since mount
    #[inline]
    pub fn syncwrites(&self) -> u64 {
        self.0.f_syncwrites as u64
    }

    /// Synchronous reads since mount
    #[inline]
    pub fn syncreads(&self) -> u64 {
        self.0.f_syncreads as u64
    }

    /// Asynchronous writes since mount
    #[inline]
    pub fn asyncwrites(&self) -> u64 {
        self.0.f_asyncwrites as u64
    }

    /// Asynchronous reads since mount
    #[inline]
    pub fn asyncreads(&self) -> u64 {
        self.0.f_asyncreads as u64
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "linux", any(target_env = "gnu", target_env = "")))] {
        #[allow(non_camel_case_types)]
        type fstype_t = libc::__fsword_t;
    } else if #[cfg(linuxlike)] {
        #[allow(non_camel_case_types)]
        type fstype_t = libc::c_ulong;
    } else if #[cfg(any(target_os = "freebsd", apple))] {
        #[allow(non_camel_case_types)]
        type fstype_t = u32;
    } else if #[cfg(target_os = "dragonfly")] {
        #[allow(non_camel_case_types)]
        type fstype_t = i32;
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(not(any(target_os = "netbsd", target_os = "openbsd"))))
)]
#[cfg(not(target_os = "openbsd"))]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct FsType(pub fstype_t);

#[cfg(linuxlike)]
bitflags::bitflags! {
    /// Flags returned by [`Statfs::flags()`].
    ///
    /// These are very platform-specific (though some flags like `RDONLY`, `NOEXEC`, and
    /// `NOSUID` are common).
    #[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
    pub struct StatfsFlags: u64 {
        const MANDLOCK = libc::ST_MANDLOCK as _;
        const NOATIME = libc::ST_NOATIME as _;
        const NODEV = libc::ST_NODEV as _;
        const NODIRATIME = libc::ST_NODIRATIME as _;
        const NOEXEC = libc::ST_NOEXEC as _;
        const NOSUID = libc::ST_NOSUID as _;
        const RDONLY = libc::ST_RDONLY as _;
        const SYNCHRONOUS = libc::ST_SYNCHRONOUS as _;
        const RELATIME = 4096;
    }
}

#[cfg(bsd)]
macro_rules! bsd_declare_statfs_flags {
    ($(
        #[cfg($cfg:meta)]
        $(
            $(#[doc = $doc:literal])*
            $name:ident = $libc_name:ident,
        )+
    )*) => {
        bitflags::bitflags! {
            /// Flags returned by [`Statfs::flags()`].
            ///
            /// These are very platform-specific (though some flags like `RDONLY`, `NOEXEC`, and
            /// `NOSUID` are common).
            #[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
            pub struct StatfsFlags: u64 {
                $($(
                    #[cfg($cfg)]
                    #[cfg_attr(docsrs, doc(cfg($cfg)))]
                    $(#[doc = $doc])*
                    const $name = sys::$libc_name as _;
                )*)*
            }
        }
    };
}

#[cfg(bsd)]
bsd_declare_statfs_flags! {
    #[cfg(all())]
    RDONLY = MNT_RDONLY,
    NOEXEC = MNT_NOEXEC,
    NOSUID = MNT_NOSUID,
    NOATIME = MNT_NOATIME,
    SYNCHRONOUS = MNT_SYNCHRONOUS,
    ASYNC = MNT_ASYNC,
    LOCAL = MNT_LOCAL,
    QUOTA = MNT_QUOTA,
    ROOTFS = MNT_ROOTFS,
    EXPORTED = MNT_EXPORTED,

    #[cfg(target_os = "openbsd")]
    NOPERM = MNT_NOPERM,
    WXALLOWED = MNT_WXALLOWED,

    #[cfg(target_os = "freebsd")]
    GJOURNAL = MNT_GJOURNAL,
    ACLS = MNT_ACLS,
    SUJ = MNT_SUJ,
    NFS4ACLS = MNT_NFS4ACLS,
    UNTRUSTED = MNT_UNTRUSTED,
    VERIFIED = MNT_VERIFIED,

    #[cfg(target_os = "dragonfly")]
    TRIM = MNT_TRIM,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    DOVOLFS = MNT_DOVOLFS,
    DONTBROWSE = MNT_DONTBROWSE,
    IGNORE_OWNERSHIP = MNT_IGNORE_OWNERSHIP,
    QUARANTINE = MNT_QUARANTINE,
    NOUSERXATTR = MNT_NOUSERXATTR,
    JOURNALED = MNT_JOURNALED,
    DEFWRITE = MNT_DEFWRITE,
    CPROTECT = MNT_CPROTECT,
    REMOVABLE = MNT_REMOVABLE,
    SNAPSHOT = MNT_SNAPSHOT,
    STRICTATIME = MNT_STRICTATIME,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    SOFTDEP = MNT_SOFTDEP,
    USER = MNT_USER,
    EXKERB = MNT_EXKERB,
    SUIDDIR = MNT_SUIDDIR,
    NOSYMFOLLOW = MNT_NOSYMFOLLOW,
    EXPUBLIC = MNT_EXPUBLIC,
    IGNORE = MNT_IGNORE,
    NOCLUSTERR = MNT_NOCLUSTERR,
    NOCLUSTERW = MNT_NOCLUSTERW,

    #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "ios"))]
    UNION = MNT_UNION,
    MULTILABEL = MNT_MULTILABEL,

    #[cfg(not(target_os = "freebsd"))]
    NODEV = MNT_NODEV,

    #[cfg(not(target_os = "openbsd"))]
    AUTOMOUNTED = MNT_AUTOMOUNTED,

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    EXRDONLY = MNT_EXRDONLY,
    DEFEXPORTED = MNT_DEFEXPORTED,
    EXPORTANON = MNT_EXPORTANON,
}

#[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
#[inline]
pub fn statfs<P: AsPath>(path: P) -> Result<Statfs> {
    path.with_cstr(|path| {
        let mut buf = MaybeUninit::uninit();
        Error::unpack_nz(unsafe { libc::statfs(path.as_ptr(), buf.as_mut_ptr()) })?;
        Ok(Statfs(unsafe { core::mem::transmute(buf.assume_init()) }))
    })
}

#[cfg_attr(docsrs, doc(cfg(not(target_os = "netbsd"))))]
#[inline]
pub fn fstatfs(fd: RawFd) -> Result<Statfs> {
    let mut buf = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::fstatfs(fd, buf.as_mut_ptr()) })?;
    Ok(Statfs(unsafe { core::mem::transmute(buf.assume_init()) }))
}

/// Retrieve a list of mounted filesystems.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "macos",
        target_os = "ios"
    )))
)]
#[cfg(bsd)]
#[inline]
pub fn getfsstat(buf: Option<&mut [Statfs]>, nowait: bool) -> Result<usize> {
    use core::convert::TryInto;

    let (ptr, len) = match buf {
        Some(buf) => (
            buf.as_mut_ptr() as *mut _,
            buf.len().try_into().unwrap_or(i32::MAX as _),
        ),
        None => (core::ptr::null_mut(), 0),
    };

    let n = Error::unpack(unsafe {
        sys::getfsstat(
            ptr,
            len,
            if nowait {
                sys::MNT_NOWAIT
            } else {
                sys::MNT_WAIT
            },
        )
    })?;

    Ok(n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_statfs_same(sfs1: &Statfs, sfs2: &Statfs) {
        assert_eq!(sfs1.flags(), sfs2.flags());
        assert_eq!(sfs1.blocks(), sfs2.blocks());
        assert_eq!(sfs1.bsize(), sfs2.bsize());
        assert_eq!(sfs1.fsid(), sfs2.fsid());

        #[cfg(not(target_os = "openbsd"))]
        assert_eq!(sfs1.fstype(), sfs2.fstype());

        #[cfg(not(any(target_os = "dragonfly", apple)))]
        assert_eq!(sfs1.namelen(), sfs2.namelen());

        #[cfg(bsd)]
        {
            assert_eq!(sfs1.iosize(), sfs2.iosize());
            assert_eq!(sfs1.owner(), sfs2.owner());
            assert_eq!(sfs1.fstypename(), sfs2.fstypename());
            assert_eq!(sfs1.mnttoname(), sfs2.mnttoname());
            assert_eq!(sfs1.mntfromname(), sfs2.mntfromname());
        }
    }

    #[test]
    fn test_statfs_fstatfs() {
        for &path in [
            crate::c_paths::slash(),
            CStr::from_bytes_with_nul(b"/bin\0").unwrap(),
            CStr::from_bytes_with_nul(b"/tmp\0").unwrap(),
        ].iter() {
            let sfs1 = statfs(path).unwrap();

            let f2 =
                crate::open(path, crate::OFlag::O_RDONLY | crate::OFlag::O_CLOEXEC, 0).unwrap();
            let sfs2 = fstatfs(f2.fd()).unwrap();

            check_statfs_same(&sfs1, &sfs2);
        }
    }

    #[cfg(all(bsd, feature = "alloc"))]
    #[test]
    fn test_getfsstat() {
        let len = getfsstat(None, false).unwrap();
        let mut buf = vec![Statfs::zeroed(); len];
        let len = getfsstat(Some(&mut buf), false).unwrap();
        let buf = &buf[..len];

        for sfs1 in buf {
            let sfs2 = statfs(sfs1.mnttoname()).unwrap();
            check_statfs_same(&sfs1, &sfs2);
        }
    }
}
