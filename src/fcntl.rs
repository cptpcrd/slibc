use crate::internal_prelude::*;

macro_rules! define_oflag {
    ($(
        $(#[doc = $doc:literal])*
        $(#[cfg($($args:tt)*)])*
        $name:ident,
    )+) => {
        bitflags::bitflags! {
            /// Flags for [`open()`] and [`openat()`].
            ///
            /// See open(2) for more details.
            ///
            /// These flags may also be used for other functions, like `pipe2()` and `dup3()`. See
            /// those functions' documentation for details.
            pub struct OFlag: libc::c_int {
                $(
                    $(#[doc = $doc])*
                    $(
                        #[cfg($($args)*)]
                        #[cfg_attr(docsrs, doc(cfg($($args)*)))]
                    )*
                    const $name = libc::$name;
                )+
            }
        }
    };
}

define_oflag! {
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
    /// Open the file in nonblocking mode.
    ///
    /// This has no effect for regular files. When opening FIFOs, it prevents the (normal) blocking on
    /// `open()` until another process opens the file (exact behavior is described in open(3p)).
    /// Behavior for other file types is often OS-dependent.
    O_NONBLOCK,
    /// Usually an alias for `O_NONBLOCK`.
    O_NDELAY,

    // Present on most systems, but not all

    #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "openbsd")))]
    O_DIRECT,
    /// Synchronize file data (not metadata) after each write (often roughly equivalent to an
    /// `fdatasync()` after each write).
    #[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
    O_DSYNC,

    // Present on the BSDs and macOS

    #[cfg(any(
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    O_SHLOCK,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    O_EXLOCK,

    // Linux-specific flags

    #[cfg(target_os = "linux")]
    O_NOATIME,
    #[cfg(target_os = "linux")]
    O_TMPFILE,
    #[cfg(target_os = "linux")]
    O_PATH,

    // FreeBSD-specific flags

    #[cfg(target_os = "freebsd")]
    O_TTY_INIT,

    // O_SEARCH and O_EXEC are defined on some systems

    /// When opening a directory (behavior is undefined for other file types), open it for
    /// searching only.
    ///
    /// The resulting file descriptor cannot be used to list the contents of the directory; only to
    /// access files within it using [`openat()`] and the other `*at()` functions.
    #[cfg(all(target_os = "linux", target_env = "musl"))]
    O_SEARCH,
    /// When opening a non-directory file (behavior is undefined for other file types), open it for
    /// execution (with `fexecve()`) only.
    #[cfg(any(all(target_os = "linux", target_env = "musl"), target_os = "freebsd"))]
    O_EXEC,
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
        Error::unpack(libc::open(path.as_ptr(), flags.bits(), mode)).map(|fd| FileDesc::new(fd))
    })
}

#[inline]
pub fn openat<P: AsPath>(dirfd: RawFd, path: P, flags: OFlag, mode: u32) -> Result<FileDesc> {
    path.with_cstr(|path| unsafe {
        Error::unpack(libc::openat(dirfd, path.as_ptr(), flags.bits(), mode))
            .map(|fd| FileDesc::new(fd))
    })
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

#[inline]
pub fn fcntl_dupfd(fd: RawFd, minfd: RawFd) -> Result<FileDesc> {
    unsafe { Ok(FileDesc::new(fcntl_arg(fd, libc::F_DUPFD, minfd)?)) }
}

#[inline]
pub fn fcntl_dupfd_cloexec(fd: RawFd, minfd: RawFd) -> Result<FileDesc> {
    unsafe { Ok(FileDesc::new(fcntl_arg(fd, libc::F_DUPFD_CLOEXEC, minfd)?)) }
}

#[inline]
pub fn fcntl_getfd(fd: RawFd) -> Result<libc::c_int> {
    unsafe { fcntl_arg(fd, libc::F_GETFD, 0) }
}

#[inline]
pub fn fcntl_setfd(fd: RawFd, flags: libc::c_int) -> Result<()> {
    unsafe {
        fcntl_arg(fd, libc::F_SETFD, flags)?;
    }
    Ok(())
}

#[inline]
pub fn fcntl_getfl(fd: RawFd) -> Result<RawFd> {
    unsafe { fcntl_arg(fd, libc::F_GETFL, 0) }
}

#[inline]
pub fn fcntl_setfl(fd: RawFd, flags: libc::c_int) -> Result<()> {
    unsafe {
        fcntl_arg(fd, libc::F_SETFL, flags)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[inline]
pub fn fcntl_getpipe_sz(fd: RawFd) -> Result<RawFd> {
    unsafe { fcntl_arg(fd, libc::F_GETPIPE_SZ, 0) }
}

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
/// `buf` must be an array [`PATH_MAX`] bytes long.
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
/// `buf` must be at least [`PATH_MAX`](../limits/constant.PATH_MAX.html) bytes long. (This is
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

    #[cfg(target_os = "linux")]
    #[test]
    fn test_pipe_sz() {
        let (r, w) = crate::pipe().unwrap();

        let mut size = fcntl_getpipe_sz(r.fd()).unwrap();
        assert_eq!(size, fcntl_getpipe_sz(w.fd()).unwrap());

        size /= 2;
        fcntl_setpipe_sz(w.fd(), size).unwrap();
        assert_eq!(size, fcntl_getpipe_sz(r.fd()).unwrap());
    }
}
