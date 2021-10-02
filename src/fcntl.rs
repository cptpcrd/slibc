use crate::internal_prelude::*;

macro_rules! define_oflag {
    (
        $(
            #[cfg($cfg:meta)]
            $($(#[doc = $doc:literal])* $name:ident,)*
        )*
        @sys,
        $(
            #[cfg($cfg2:meta)]
            $($(#[doc = $doc2:literal])* $name2:ident,)*
        )*
    ) => {
        bitflags::bitflags! {
            /// Flags for [`open()`] and [`openat()`].
            ///
            /// See open(2) for more details.
            ///
            /// These flags may also be used for other functions, like `pipe2()` and `dup3()`. See
            /// those functions' documentation for details.
            pub struct OFlag: libc::c_int {
                $($(
                    $(#[doc = $doc])*
                    #[cfg($cfg)]
                    #[cfg_attr(docsrs, doc(cfg($cfg)))]
                    const $name = libc::$name;
                )*)*

                $($(
                    $(#[doc = $doc2])*
                    #[cfg($cfg2)]
                    #[cfg_attr(docsrs, doc(cfg($cfg2)))]
                    const $name2 = sys::$name2;
                )*)*
            }
        }
    };
}

define_oflag! {
    #[cfg(all())]
    /// Open the file for reading only.
    O_RDONLY,
    /// Open the file for writing only.
    O_WRONLY,
    /// Open the file for reading and writing.
    ///
    /// This is not guaranteed to be the same as `O_RDONLY | O_WRONLY`! Always use this flag if you
    /// need to open a file for both reading and writing.
    O_RDWR,
    /// Set the file offset to the end of the file before each write.
    ///
    /// This is done atomically, so multiple processes can write to the same file with `O_APPEND`
    /// (except over network filesystems, where this may be emulated and may not work properly).
    O_APPEND,
    /// Truncate the file to length 0 when opening.
    O_TRUNC,
    /// Set the close-on-exec flag on the new file descriptor.
    O_CLOEXEC,
    /// Create the file if it does not already exist.
    ///
    /// The `mode` argument to `open()`/`openat()`, with the process's umask bits removed, is used
    /// to set the mode of the newly created file.
    O_CREAT,
    /// When used with `O_CREAT`, fail with `EEXIST` (atomically) if the file already exists.
    ///
    /// This can only be portably used together with `O_CREAT`.
    O_EXCL,
    /// Fail with `EISDIR` if the specified file is not a directory.
    O_DIRECTORY,
    /// Synchronize file data and metadata after each write (often roughly equivalent to an
    /// `fsync()` after each write).
    O_SYNC,
    O_ASYNC,
    /// If the specified file is a terminal device, do not make it the process's controlling
    /// terminal.
    O_NOCTTY,
    /// If the final component of the given path refers to a symbolic link, fail with `ELOOP`.
    O_NOFOLLOW,
    /// Open the file in nonblocking mode.
    ///
    /// This has no effect for regular files. When opening FIFOs, it prevents the (normal) blocking on
    /// `open()` until another process opens the file (exact behavior is described in open(3p)).
    /// Behavior for other file types is often OS-dependent.
    O_NONBLOCK,
    /// Usually an alias for `O_NONBLOCK`.
    O_NDELAY,

    // All except macOS/OpenBSD
    #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "openbsd")))]
    O_DIRECT,

    // All except FreeBSD/DragonFlyBSD
    #[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
    /// Synchronize file data (not metadata) after each write (often roughly equivalent to an
    /// `fdatasync()` after each write).
    O_DSYNC,

    // macOS/*BSD-specific
    #[cfg(any(
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    O_SHLOCK,
    O_EXLOCK,

    // Linux-specific
    #[cfg(target_os = "linux")]
    O_NOATIME,
    O_TMPFILE,
    O_PATH,

    // FreeBSD-specific
    #[cfg(target_os = "freebsd")]
    O_TTY_INIT,

    // NetBSD-specific
    #[cfg(target_os = "netbsd")]
    O_NOSIGPIPE,

    // O_SEARCH and O_EXEC are defined on some systems

    #[cfg(any(all(target_os = "linux", target_env = "musl"), target_os = "freebsd"))]
    /// When opening a non-directory file (behavior is undefined for other file types), open it for
    /// execution (with `fexecve()`) only.
    O_EXEC,

    @sys,

    #[cfg(any(all(target_os = "linux", target_env = "musl"), target_os = "freebsd"))]
    /// When opening a directory (behavior is undefined for other file types), open it for
    /// searching only.
    ///
    /// The resulting file descriptor cannot be used to list the contents of the directory; only to
    /// access files within it using [`openat()`] and the other `*at()` functions.
    O_SEARCH,
}

bitflags::bitflags! {
    pub struct AtFlag: libc::c_int {
        const AT_REMOVEDIR = libc::AT_REMOVEDIR;
        const AT_SYMLINK_NOFOLLOW = libc::AT_SYMLINK_NOFOLLOW;
        const AT_SYMLINK_FOLLOW = libc::AT_SYMLINK_FOLLOW;

        #[cfg(not(target_os = "android"))]
        const AT_EACCESS = libc::AT_EACCESS;

        #[cfg(target_os = "linux")]
        const AT_EMPTY_PATH = libc::AT_EMPTY_PATH;
        #[cfg(target_os = "linux")]
        const AT_NO_AUTOMOUNT = libc::AT_NO_AUTOMOUNT;

        /// When using [`statx()`] to query a remote filesystem, match the behavior of `stat()`
        /// when deciding whether to synchronize (this is the default).
        #[cfg(target_os = "linux")]
        const AT_STATX_SYNC_AS_STAT = 0x0;
        /// When using [`statx()`] to query a remote filesystem, force synchronizing the latest
        /// attributes.
        #[cfg(target_os = "linux")]
        const AT_STATX_FORCE_SYNC = 0x2000;
        /// When using [`statx()`] to query a remote filesystem, use the latest cached information
        /// if possible (may be out of date).
        #[cfg(target_os = "linux")]
        const AT_STATX_DONT_SYNC = 0x4000;
    }
}

pub const AT_FDCWD: RawFd = libc::AT_FDCWD;

#[inline]
pub fn open<P: AsPath>(path: P, flags: OFlag, mode: u32) -> Result<FileDesc> {
    path.with_cstr(|path| unsafe {
        Error::unpack_fdesc(libc::open(path.as_ptr(), flags.bits(), mode))
    })
}

#[inline]
pub fn openat<P: AsPath>(dirfd: RawFd, path: P, flags: OFlag, mode: u32) -> Result<FileDesc> {
    path.with_cstr(|path| unsafe {
        Error::unpack_fdesc(libc::openat(dirfd, path.as_ptr(), flags.bits(), mode))
    })
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn readahead(fd: RawFd, offset: u64, count: usize) -> Result<()> {
    if unsafe { libc::readahead(fd, offset as i64, count) } < 0 {
        Err(Error::last())
    } else {
        Ok(())
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn sendfile(
    out_fd: RawFd,
    in_fd: RawFd,
    offset: Option<&mut libc::off_t>,
    count: usize,
) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::sendfile(
            out_fd,
            in_fd,
            offset.map_or_else(core::ptr::null_mut, |o| o),
            count,
        )
    })
}

#[cfg(linuxlike)]
bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct SpliceFlags: libc::c_uint {
        const MOVE = sys::SPLICE_F_MOVE;
        const NONBLOCK = sys::SPLICE_F_NONBLOCK;
        const MORE = sys::SPLICE_F_MORE;
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn splice(
    fd_in: RawFd,
    off_in: Option<&mut u64>,
    fd_out: RawFd,
    off_out: Option<&mut u64>,
    len: usize,
    flags: SpliceFlags,
) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::splice(
            fd_in,
            off_in.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
            fd_out,
            off_out.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
            len,
            flags.bits(),
        )
    })
}

#[cfg(linuxlike)]
bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct CopyFileFlags: libc::c_uint {
        #[doc(hidden)]
        const __RESERVED = 0;
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn copy_file_range(
    fd_in: RawFd,
    off_in: Option<&mut u64>,
    fd_out: RawFd,
    off_out: Option<&mut u64>,
    len: usize,
    flags: CopyFileFlags,
) -> Result<usize> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            let n = Error::unpack_size(unsafe {
                sys::copy_file_range(
                    fd_in,
                    off_in.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
                    fd_out,
                    off_out.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
                    len,
                    flags.bits(),
                )
            })?;
        } else {
            let n = Error::unpack_size(unsafe {
                libc::syscall(
                    libc::SYS_copy_file_range,
                    fd_in,
                    off_in.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
                    fd_out,
                    off_out.map_or_else(core::ptr::null_mut, |o| o) as *mut i64,
                    len,
                    flags.bits(),
                ) as isize
            })?;
        }
    }

    Ok(n)
}

#[cfg(any(freebsdlike, apple))]
bitflags::bitflags! {
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "macos",
            target_os = "ios",
        )))
    )]
    pub struct SendFileFlags: libc::c_int {
        #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
        #[cfg(target_os = "freebsd")]
        const NODISKIO = libc::SF_NODISKIO;
        #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
        #[cfg(target_os = "freebsd")]
        const NOCACHE = libc::SF_NOCACHE;
        #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
        #[cfg(target_os = "freebsd")]
        const SYNC = libc::SF_SYNC;
        #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
        #[cfg(target_os = "freebsd")]
        const USER_READAHEAD = libc::SF_USER_READAHEAD;

        #[doc(hidden)]
        #[cfg(not(target_os = "freebsd"))]
        const __RESERVED = 0;
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(any(freebsdlike, apple))]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct SendHdtr<'a, 'b> {
    inner: libc::sf_hdtr,
    headers: core::marker::PhantomData<&'a [crate::IoVec<'b>]>,
    trailers: core::marker::PhantomData<&'a [crate::IoVec<'b>]>,
}

#[cfg(any(freebsdlike, apple))]
impl<'a, 'b> SendHdtr<'a, 'b> {
    #[inline]
    pub fn new(headers: &'a [crate::IoVec<'b>], trailers: &'a [crate::IoVec<'b>]) -> Self {
        Self {
            inner: libc::sf_hdtr {
                headers: headers.as_ptr() as _,
                hdr_cnt: headers.len().try_into().unwrap(),
                trailers: trailers.as_ptr() as _,
                trl_cnt: trailers.len().try_into().unwrap(),
            },
            headers: core::marker::PhantomData,
            trailers: core::marker::PhantomData,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "freebsd", target_os = "dragonfly"))))]
#[cfg(freebsdlike)]
pub fn sendfile(
    fd: RawFd,
    s: RawFd,
    offset: u64,
    nbytes: usize,
    hdtr: Option<&SendHdtr>,
    sbytes: Option<&mut u64>,
    flags: SendFileFlags,
) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::sendfile(
            fd,
            s,
            offset as i64,
            nbytes,
            hdtr.map_or_else(core::ptr::null, |ht| ht) as _,
            sbytes.map_or_else(core::ptr::null, |s| s) as *mut i64,
            flags.bits(),
        )
    })
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
#[cfg(apple)]
pub fn sendfile(
    fd: RawFd,
    s: RawFd,
    offset: u64,
    len: &mut u64,
    hdtr: Option<&SendHdtr>,
    flags: SendFileFlags,
) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::sendfile(
            fd,
            s,
            offset as i64,
            len as *mut u64 as *mut i64,
            hdtr.map_or_else(core::ptr::null, |ht| ht) as _,
            flags.bits(),
        )
    })
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd"
    )))
)]
#[cfg(any(linuxlike, freebsdlike, target_os = "netbsd"))]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum PosixFAdvice {
    NORMAL = sys::POSIX_FADV_NORMAL,
    SEQUENTIAL = sys::POSIX_FADV_SEQUENTIAL,
    RANDOM = sys::POSIX_FADV_RANDOM,
    NOREUSE = sys::POSIX_FADV_NOREUSE,
    WILLNEED = sys::POSIX_FADV_WILLNEED,
    DONTNEED = sys::POSIX_FADV_DONTNEED,
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd"
    )))
)]
#[cfg(any(linuxlike, freebsdlike, target_os = "netbsd"))]
#[inline]
pub fn posix_fadvise(fd: RawFd, offset: u64, len: u64, advice: PosixFAdvice) -> Result<()> {
    Error::unpack_eno(unsafe { sys::posix_fadvise(fd, offset as _, len as _, advice as _) })
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd"
    )))
)]
#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
#[inline]
pub fn posix_fallocate(fd: RawFd, offset: u64, len: u64) -> Result<()> {
    Error::unpack_eno(unsafe { sys::posix_fallocate(fd, offset as _, len as _) })
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
bitflags::bitflags! {
    #[derive(Default)]
    pub struct FallocMode: libc::c_int {
        const KEEP_SIZE = libc::FALLOC_FL_KEEP_SIZE;
        const PUNCH_HOLE = libc::FALLOC_FL_PUNCH_HOLE;
        const COLLAPSE_RANGE = libc::FALLOC_FL_COLLAPSE_RANGE;
        const ZERO_RANGE = libc::FALLOC_FL_ZERO_RANGE;
        const INSERT_RANGE = libc::FALLOC_FL_INSERT_RANGE;
        const UNSHARE_RANGE = libc::FALLOC_FL_UNSHARE_RANGE;
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn fallocate(fd: RawFd, mode: FallocMode, offset: u64, len: u64) -> Result<()> {
    Error::unpack_nz(unsafe { libc::fallocate(fd, mode.bits(), offset as _, len as _) })
}

/// Call `fcntl()` with an `int` argument.
///
/// # Safety
///
/// 1. The given `arg` must be appropriate for the given `cmd` (from the constants defined in
///    `libc`).
/// 2. This must not be used to violate other invariants.
#[inline]
pub unsafe fn fcntl_arg(fd: RawFd, cmd: libc::c_int, arg: libc::c_int) -> Result<libc::c_int> {
    Error::unpack(libc::fcntl(fd, cmd, arg))
}

/// Call `fcntl()` with an pointer argument.
///
/// # Safety
///
/// 1. The given `arg` must be a valid pointer to a buffer that is large enough for the given
///    `cmd`.
/// 2. This must not be used to violate other invariants.
#[inline]
pub unsafe fn fcntl_ptr(
    fd: RawFd,
    cmd: libc::c_int,
    arg: *mut libc::c_void,
) -> Result<libc::c_int> {
    Error::unpack(libc::fcntl(fd, cmd, arg))
}

/// Duplicate the given file descriptor.
///
/// `fd` specifies the file descriptor to duplicate; the new file descriptor will be the lowest
/// available file descriptor greater than or equal to `minfd`.
///
/// `slibc::fcntl_dupfd(fd, 0)` is equivalent to `slibc::dup(fd)`.
#[inline]
pub fn fcntl_dupfd(fd: RawFd, minfd: RawFd) -> Result<FileDesc> {
    unsafe { Ok(FileDesc::new(fcntl_arg(fd, libc::F_DUPFD, minfd)?)) }
}

/// Duplicate the given file descriptor, setting the close-on-exec flag on the new file descriptor.
///
/// This is equivalent to [`fcntl_dupfd()`], except that the close-on-exec flag is also set.
#[inline]
pub fn fcntl_dupfd_cloexec(fd: RawFd, minfd: RawFd) -> Result<FileDesc> {
    unsafe { Ok(FileDesc::new(fcntl_arg(fd, libc::F_DUPFD_CLOEXEC, minfd)?)) }
}

/// Get the flags associated with the given file descriptor.
///
/// See [`fcntl_setfd()`].
#[inline]
pub fn fcntl_getfd(fd: RawFd) -> Result<libc::c_int> {
    unsafe { fcntl_arg(fd, libc::F_GETFD, 0) }
}

/// Set the flags associated with the given file descriptor.
///
/// Currently there is only one flag: `FD_CLOEXEC`, the close-on-exec flag. (It isn't exported by
/// this crate, but can be accessed through `libc`.) If this flag is set on a file descriptor, the
/// file descriptor will be closed when replacing the process via `exec()`.
///
/// See `fcntl(2)` for more information.
#[inline]
pub fn fcntl_setfd(fd: RawFd, flags: libc::c_int) -> Result<()> {
    unsafe {
        fcntl_arg(fd, libc::F_SETFD, flags)?;
    }
    Ok(())
}

#[inline]
pub fn fcntl_getfl(fd: RawFd) -> Result<libc::c_int> {
    unsafe { fcntl_arg(fd, libc::F_GETFL, 0) }
}

#[inline]
pub fn fcntl_setfl(fd: RawFd, flags: libc::c_int) -> Result<()> {
    unsafe {
        fcntl_arg(fd, libc::F_SETFL, flags)?;
    }
    Ok(())
}

/// Get the capacity of the specified pipe.
///
/// See [`fcntl_setpipe_sz()`].
#[cfg(target_os = "linux")]
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn fcntl_getpipe_sz(fd: RawFd) -> Result<libc::c_int> {
    unsafe { fcntl_arg(fd, libc::F_GETPIPE_SZ, 0) }
}

/// Set the capacity of the specified pipe.
///
/// The minimum capacity is the system page size (see [`getpagesize()`]), and the maximum size (for
/// unprivileged users) is defined by `/proc/sys/fs/pipe-max-size` (see `proc(5)`). A process with
/// the CAP_SYS_RESOURCE capability can override the maximum limit.
///
/// - Attempts to set the capacity to a value smaller than the system page size will result in it
///   being silently rounded up to the page size.
/// - Attempts (by an unprivileged process) to set the capacity to a value larger than the upper
///   limit will fail with the error EPERM.
/// - Attempts to set the capacity to a value smaller than the amount of buffer space currently in
///   use to store data will fail with the error EBUSY.
///
/// More information can be found in `fcntl(2)`.
///
/// [`getpagesize()`]: ./fn.getpagesize.html
#[cfg(target_os = "linux")]
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn fcntl_setpipe_sz(fd: RawFd, cap: libc::c_int) -> Result<()> {
    unsafe {
        fcntl_arg(fd, libc::F_SETPIPE_SZ, cap)?;
    }
    Ok(())
}

/// Get the path to which the given file descriptor is open.
///
/// `buf` must be an array [`PATH_MAX`](./constant.PATH_MAX.html) bytes long.
///
/// To use a dynamically allocated buffer, see [`fcntl_getpath_unchecked()`].
#[cfg(target_os = "macos")]
#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
pub fn fcntl_getpath(fd: RawFd, buf: &mut [u8; crate::PATH_MAX]) -> Result<&CStr> {
    unsafe {
        fcntl_ptr(fd, libc::F_GETPATH, buf.as_mut_ptr() as *mut _)?;
    }

    Ok(util::cstr_from_buf(buf).unwrap())
}

/// Get the path to which the given file descriptor is open.
///
/// # Safety
///
/// `buf` must be at least [`PATH_MAX`](./constant.PATH_MAX.html) bytes long. (This is
/// verified if debug assertions are enabled.)
#[cfg(target_os = "macos")]
#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
pub unsafe fn fcntl_getpath_unchecked(fd: RawFd, buf: &mut [u8]) -> Result<&CStr> {
    debug_assert!(buf.len() >= libc::PATH_MAX as usize);
    fcntl_ptr(fd, libc::F_GETPATH, buf.as_mut_ptr() as *mut _)?;

    Ok(util::cstr_from_buf(buf).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_fdesc() -> FileDesc {
        crate::pipe().unwrap().0
    }

    #[test]
    fn test_dupfd() {
        let fdesc = get_fdesc();

        let fdesc2 = fcntl_dupfd(fdesc.fd(), 100).unwrap();
        assert!(fdesc2.fd() >= 100);
        assert!(!fdesc2.get_cloexec().unwrap());

        let fdesc3 = fcntl_dupfd_cloexec(fdesc.fd(), 200).unwrap();
        assert!(fdesc3.fd() >= 200);
        assert!(fdesc3.get_cloexec().unwrap());
    }

    #[test]
    fn test_open() {
        let devnull = unsafe { CStr::from_bytes_with_nul_unchecked(b"/dev/null\0") };
        let devzero = unsafe { CStr::from_bytes_with_nul_unchecked(b"/dev/zero\0") };

        let mut buf = [0; 1024];

        let f = open(devnull, OFlag::O_RDWR, 0).unwrap();

        // Empty reads
        assert_eq!(f.read(&mut buf).unwrap(), 0);

        // Writes are accepted but ignored
        assert!(f.write(b"ignored").unwrap() > 0);

        let f = open(devzero, OFlag::O_RDWR | OFlag::O_CLOEXEC, 0).unwrap();

        // Reads all zeroes
        let n = f.read(&mut buf).unwrap();
        assert_ne!(n, 0);
        for &ch in buf[..n].iter() {
            assert_eq!(ch, 0);
        }

        // Writes are accepted but ignored
        assert!(f.write(b"ignored").unwrap() > 0);
    }

    #[test]
    fn test_getset_fl() {
        assert_eq!(fcntl_getfl(-1).unwrap_err(), Errno::EBADF);
        assert_eq!(fcntl_setfl(-1, 0).unwrap_err(), Errno::EBADF);

        let fdesc = get_fdesc();

        assert!(!fdesc.get_nonblocking().unwrap());
        let mut flags = fcntl_getfl(fdesc.fd()).unwrap();
        assert_eq!(flags & libc::O_NONBLOCK, 0);

        flags |= libc::O_NONBLOCK;
        fcntl_setfl(fdesc.fd(), flags).unwrap();
        assert!(fdesc.get_nonblocking().unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_pipe_sz() {
        let (r, w) = crate::pipe().unwrap();

        let mut size = fcntl_getpipe_sz(r.fd()).unwrap();
        assert_eq!(size, fcntl_getpipe_sz(w.fd()).unwrap());

        size /= 2;
        fcntl_setpipe_sz(w.fd(), size).unwrap();
        assert_eq!(size, fcntl_getpipe_sz(r.fd()).unwrap());

        size *= 4;
        fcntl_setpipe_sz(w.fd(), size).unwrap();
        assert_eq!(size, fcntl_getpipe_sz(r.fd()).unwrap());
    }

    #[cfg(all(
        any(linuxlike, target_os = "freebsd", target_os = "netbsd"),
        feature = "std"
    ))]
    #[test]
    fn test_posix_fallocate() {
        let file: crate::FileDesc = tempfile::tempfile().unwrap().into();
        assert_eq!(file.stat().unwrap().size(), 0);

        posix_fallocate(file.fd(), 0, 1024).unwrap();
        assert_eq!(file.stat().unwrap().size(), 1024);

        posix_fallocate(file.fd(), 0, 100).unwrap();
        assert_eq!(file.stat().unwrap().size(), 1024);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_getpath() {
        fn check_path(path: &CStr) {
            let f = open(path, OFlag::O_RDONLY, 0).unwrap();

            let mut buf = [0; crate::PATH_MAX];
            assert_eq!(fcntl_getpath(f.fd(), &mut buf).unwrap(), path);
            assert_eq!(
                unsafe { fcntl_getpath_unchecked(f.fd(), &mut buf[..]) }.unwrap(),
                path,
            );
        }

        check_path(crate::c_paths::slash());
        check_path(CStr::from_bytes_with_nul(b"/dev/null\0").unwrap());

        let (r, w) = crate::pipe().unwrap();
        for &fd in [-1, r.fd(), w.fd()].iter() {
            assert_eq!(
                fcntl_getpath(fd, &mut [0; crate::PATH_MAX]).unwrap_err(),
                Errno::EBADF
            );
            assert_eq!(
                unsafe { fcntl_getpath_unchecked(fd, &mut [0; crate::PATH_MAX]) }.unwrap_err(),
                Errno::EBADF
            );
        }
    }
}
