use core::convert::TryInto;

use crate::internal_prelude::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum SysconfName {
    HOST_NAME_MAX = libc::_SC_HOST_NAME_MAX,
    LOGIN_NAME_MAX = libc::_SC_LOGIN_NAME_MAX,
    TTY_NAME_MAX = libc::_SC_TTY_NAME_MAX,
    NGROUPS_MAX = libc::_SC_NGROUPS_MAX,
    PAGESIZE = libc::_SC_PAGESIZE,
}

#[inline]
pub fn sysconf(name: SysconfName) -> Option<usize> {
    match unsafe { libc::sysconf(name as libc::c_int) } {
        -1 => None,
        res => Some(res as usize),
    }
}

/// Get the number of bytes in a memory page.
pub fn getpagesize() -> usize {
    unsafe { sys::getpagesize() as usize }
}

/// Synchronize all filesystem modifications to the underlying filesystems.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub fn sync() {
    unsafe {
        libc::sync();
    }
}

/// Synchronize all modifications to the filesystem containing the file referred to by `fd`.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn syncfs(fd: RawFd) -> Result<()> {
    Error::unpack_nz(unsafe { sys::syncfs(fd) })
}

/// Sync all modifications to the file referred to by `fd` to the underlying filesystem.
#[inline]
pub fn fsync(fd: RawFd) -> Result<()> {
    Error::unpack_nz(unsafe { libc::fsync(fd) })
}

/// Sync all modifications to the data of the file referred to by `fd` to the underlying
/// filesystem.
///
/// Unlike `fsync()`, this may not synchronize file *metadata*.
#[cfg_attr(
    docsrs,
    doc(cfg(not(any(target_os = "macos", target_os = "ios", target_os = "dragonfly"))))
)]
#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "dragonfly")))]
#[inline]
pub fn fdatasync(fd: RawFd) -> Result<()> {
    Error::unpack_nz(unsafe { libc::fdatasync(fd) })
}

/// Change this process's working directory to the directory specified by the given file descriptor.
#[inline]
pub fn fchdir(fd: RawFd) -> Result<()> {
    Error::unpack_nz(unsafe { libc::fchdir(fd) })
}

/// Change this process's working directory to the directory specified by the given path.
///
/// See [String/path handling](../index.html#stringpath-handling) at the crate root for more
/// information regarding how string arguments are handled.
#[inline]
pub fn chdir<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|s| unsafe { Error::unpack_nz(libc::chdir(s.as_ptr())) })
}

/// Change this process's root directory to the directory specified by the given path.
///
/// See [String/path handling](../index.html#stringpath-handling) at the crate root for more
/// information regarding how string arguments are handled.
///
/// # Use with `chdir()`
///
/// To avoid potential for later escapes, a common idiom is to change the working directory,
/// then immediately `chroot(".")`.
/// example:
///
/// ```no_run
/// # #[cfg(feature = "alloc")]
/// # {
/// # use slibc::{chdir, chroot};
/// chdir("/a/b/c").unwrap();
/// chroot(".").unwrap();
/// # }
/// ```
///
/// Or, using `fchdir()` (notably, this allows `chroot()`ing to a directory specified by a file
/// descriptor even though there is no `fchroot()` function):
///
/// ```ignore
/// # use slibc::unistd::{fchdir, chroot};
/// fchdir(fd).unwrap();
/// chroot(".").unwrap();
/// ```
#[inline]
pub fn chroot<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|s| unsafe { Error::unpack_nz(libc::chroot(s.as_ptr())) })
}

/// Fork the current process.
///
/// On success, this returns `Ok(Some(pid))` in the parent and `Ok(None)` in the child. On failure,
/// this returns an error in the parent.
///
/// # Safety
///
/// This function is highly unsafe. Basic operations such as allocating memory are not guaranteed
/// to work in the child. Use extreme caution, and carefully evaluate each function you plan to
/// call.
///
/// You may also want to take steps to ensure that the child will not panic, or that if it panics,
/// the panic will not unwind into the parent.
#[inline]
pub unsafe fn fork() -> Result<Option<libc::pid_t>> {
    match libc::fork() {
        0 => Ok(None),
        -1 => Err(Error::last()),
        pid => Ok(Some(pid)),
    }
}

/// Get the current process's PID.
#[inline]
pub fn getpid() -> libc::pid_t {
    unsafe { libc::getpid() }
}

/// Get the current thread's TID.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[allow(clippy::needless_return)]
#[inline]
pub fn gettid() -> libc::pid_t {
    unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t }
}

/// Get the parent process's PID.
#[inline]
pub fn getppid() -> libc::pid_t {
    unsafe { libc::getppid() }
}

/// Get the given process's process group ID.
///
/// 0 specifies the current process, and is equivalent to [`getpgrp()`].
#[inline]
pub fn getpgid(pid: libc::pid_t) -> Result<libc::pid_t> {
    Error::unpack(unsafe { libc::getpgid(pid) })
}

/// Get the given process's session ID.
///
/// 0 specifies the current process.
#[inline]
pub fn getsid(pid: libc::pid_t) -> Result<libc::pid_t> {
    Error::unpack(unsafe { libc::getsid(pid) })
}

/// Get the current process's process group ID.
#[inline]
pub fn getpgrp() -> libc::pid_t {
    unsafe { libc::getpgrp() }
}

/// Set the given process's process group ID.
///
/// This function can be used to either join an existing process group or create a new process
/// group within the current process's session.
///
/// If either `pid` or `pgid` is 0, the current process's PID is used. Thus, for example,
/// `setpgid(0, 0)` will make the current process the process group leader of a new process group
/// (if it is not already).
#[inline]
pub fn setpgid(pid: libc::pid_t, pgid: libc::pid_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::setpgid(pid, pgid) })
}

/// Create a new session if this process is not a process group leader.
///
/// The new session ID of this process is returned.
///
/// If any process's process group ID is the PID of this process, this will fail with `EPERM`.
/// Usually this is not significant, so the result of `setsid()` is often ignored.
#[inline]
pub fn setsid() -> Result<libc::pid_t> {
    Error::unpack(unsafe { libc::setsid() })
}

/// Returns the current real user ID.
#[inline]
pub fn getuid() -> libc::uid_t {
    unsafe { libc::getuid() }
}

/// Returns the current effective user ID.
#[inline]
pub fn geteuid() -> libc::uid_t {
    unsafe { libc::geteuid() }
}

/// Returns the current real group ID.
#[inline]
pub fn getgid() -> libc::gid_t {
    unsafe { libc::getgid() }
}

/// Returns the current effective group ID.
#[inline]
pub fn getegid() -> libc::gid_t {
    unsafe { libc::getegid() }
}

#[inline]
pub fn setuid(uid: libc::uid_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::setuid(uid) })
}

#[inline]
pub fn setgid(gid: libc::gid_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::setgid(gid) })
}

#[inline]
pub fn seteuid(uid: libc::uid_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::seteuid(uid) })
}

#[inline]
pub fn setegid(gid: libc::gid_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::setegid(gid) })
}

#[inline]
pub fn setreuid(ruid: Option<libc::uid_t>, euid: Option<libc::uid_t>) -> Result<()> {
    Error::unpack_nz(unsafe {
        sys::setreuid(
            ruid.unwrap_or(libc::uid_t::MAX),
            euid.unwrap_or(libc::uid_t::MAX),
        )
    })
}

#[inline]
pub fn setregid(rgid: Option<libc::gid_t>, egid: Option<libc::gid_t>) -> Result<()> {
    Error::unpack_nz(unsafe {
        sys::setregid(
            rgid.unwrap_or(libc::gid_t::MAX),
            egid.unwrap_or(libc::gid_t::MAX),
        )
    })
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "dragonfly",
))]
mod resids {
    use super::*;

    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly",
        )))
    )]
    #[inline]
    pub fn getresuid() -> (libc::uid_t, libc::uid_t, libc::uid_t) {
        let mut ruid = MaybeUninit::uninit();
        let mut euid = MaybeUninit::uninit();
        let mut suid = MaybeUninit::uninit();

        let ret =
            unsafe { sys::getresuid(ruid.as_mut_ptr(), euid.as_mut_ptr(), suid.as_mut_ptr()) };
        debug_assert_eq!(ret, 0);

        unsafe { (ruid.assume_init(), euid.assume_init(), suid.assume_init()) }
    }

    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly",
        )))
    )]
    #[inline]
    pub fn getresgid() -> (libc::gid_t, libc::gid_t, libc::gid_t) {
        let mut rgid = MaybeUninit::uninit();
        let mut egid = MaybeUninit::uninit();
        let mut sgid = MaybeUninit::uninit();

        let ret =
            unsafe { sys::getresgid(rgid.as_mut_ptr(), egid.as_mut_ptr(), sgid.as_mut_ptr()) };
        debug_assert_eq!(ret, 0);

        unsafe { (rgid.assume_init(), egid.assume_init(), sgid.assume_init()) }
    }

    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly",
        )))
    )]
    #[inline]
    pub fn setresuid(
        ruid: Option<libc::uid_t>,
        euid: Option<libc::uid_t>,
        suid: Option<libc::uid_t>,
    ) -> Result<()> {
        Error::unpack_nz(unsafe {
            libc::setresuid(
                ruid.unwrap_or(libc::uid_t::MAX),
                euid.unwrap_or(libc::uid_t::MAX),
                suid.unwrap_or(libc::uid_t::MAX),
            )
        })
    }

    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "dragonfly",
        )))
    )]
    #[inline]
    pub fn setresgid(
        rgid: Option<libc::gid_t>,
        egid: Option<libc::gid_t>,
        sgid: Option<libc::gid_t>,
    ) -> Result<()> {
        Error::unpack_nz(unsafe {
            libc::setresgid(
                rgid.unwrap_or(libc::gid_t::MAX),
                egid.unwrap_or(libc::gid_t::MAX),
                sgid.unwrap_or(libc::gid_t::MAX),
            )
        })
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "dragonfly",
))]
pub use resids::*;

/// Low-level interface to the C `getgroups()` function.
///
/// This attempts to store the current list of supplementary group IDs in the provided slice. It is
/// a very thin wrapper around C's `getgroups()` function, so the semantics are almost exactly the
/// same.
///
/// Namely:
/// 1. If the slice is empty (length 0), the number of current supplementary group IDs will be
///    returned.
/// 2. If the slice is long enough to hold all the current supplementary group IDs, it will be
///    filled with the current supplementary group IDs, and the number of supplementary group IDs
///    will be returned.
/// 3. If the slice is not empty and it is also not long enough to hold all the current
///    supplementary group IDs, an error will be returned.
#[inline]
pub fn getgroups(groups: &mut [libc::gid_t]) -> Result<usize> {
    let n = Error::unpack(unsafe {
        libc::getgroups(
            groups.len().try_into().unwrap_or(libc::c_int::MAX),
            groups.as_mut_ptr(),
        )
    })?;

    Ok(n as usize)
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn getgroups_alloc() -> Result<Vec<libc::gid_t>> {
    let mut groups = Vec::new();

    loop {
        let ngroups = getgroups(&mut groups)?;
        if ngroups == 0 {
            return Ok(Vec::new());
        }
        groups.resize(ngroups, 0);

        match getgroups(&mut groups) {
            Ok(n) => {
                groups.truncate(n);
                return Ok(groups);
            }
            Err(e) if e.code() == libc::EINVAL => (),
            Err(e) => return Err(e),
        }
    }
}

#[inline]
pub fn setgroups(groups: &[libc::gid_t]) -> Result<()> {
    // BSD-based systems have the length as type `int`; check for overflow on 64-bit
    #[cfg(all(target_pointer_width = "64", bsd))]
    if groups.len() > libc::c_int::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    Error::unpack_nz(unsafe { libc::setgroups(groups.len() as _, groups.as_ptr()) })
}

#[inline]
pub fn read(fd: RawFd, buf: &mut [u8]) -> Result<usize> {
    Error::unpack_size(unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) })
}

#[inline]
pub fn write(fd: RawFd, buf: &[u8]) -> Result<usize> {
    Error::unpack_size(unsafe { libc::write(fd, buf.as_ptr() as *const _, buf.len()) })
}

#[inline]
pub fn pread(fd: RawFd, buf: &mut [u8], offset: u64) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::pread(fd, buf.as_mut_ptr() as *mut _, buf.len(), offset as _)
    })
}

#[inline]
pub fn pwrite(fd: RawFd, buf: &[u8], offset: u64) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::pwrite(fd, buf.as_ptr() as *const _, buf.len(), offset as _)
    })
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum SeekPos {
    Start(u64),
    End(i64),
    Current(i64),
}

#[inline]
pub fn lseek(fd: RawFd, pos: SeekPos) -> Result<u64> {
    let (off, whence) = match pos {
        SeekPos::Start(off) => (off as i64, libc::SEEK_SET),
        SeekPos::End(off) => (off, libc::SEEK_END),
        SeekPos::Current(off) => (off, libc::SEEK_CUR),
    };

    match unsafe { libc::lseek(fd, off as _, whence) } {
        -1 => Err(Error::last()),
        pos => Ok(pos as u64),
    }
}

#[inline]
pub fn sleep(seconds: libc::c_uint) -> core::result::Result<(), libc::c_uint> {
    match unsafe { libc::sleep(seconds) } {
        0 => Ok(()),
        rem => Err(rem),
    }
}

#[inline]
pub fn usleep(usec: libc::useconds_t) -> Result<()> {
    Error::unpack_nz(unsafe { libc::usleep(usec) })
}

pub fn nice(inc: libc::c_int) -> Result<libc::c_int> {
    unsafe {
        let eno_ptr = util::errno_ptr();

        *eno_ptr = 0;

        match libc::nice(inc) {
            -1 => match *eno_ptr {
                0 => Ok(-1),
                eno => Err(Error::from_code(eno)),
            },

            prio => Ok(prio),
        }
    }
}

/// Exit the process immediately with the specified `status`, without performing any cleanup.
///
/// # Safety
///
/// Usually, `std::process::exit()` should be used, since (despite what the documentation claims)
/// it does perform some cleanup.
///
/// Generally, this function is used following a `fork()`, since the environment inside the child
/// following a `fork()` is such that a normal `exit()` may hang.
#[inline]
pub unsafe fn _exit(status: libc::c_int) -> ! {
    libc::_exit(status);
}

/// Duplicate the given file descriptor.
///
/// WARBUBG: The new file descriptor does NOT have its close-on-exec flag set! To set the
/// close-on-exec flag atomically, use [`fcntl_dupfd_cloexec()`], such as in
/// `fcntl_dupfd_cloexec(fd, 0)`.
///
/// [`fcntl_dupfd_cloexec()`]: ./fn.fcntl_dupfd_cloexec.html
#[inline]
pub fn dup(fd: RawFd) -> Result<FileDesc> {
    unsafe { Ok(FileDesc::new(Error::unpack(libc::dup(fd))?)) }
}

/// Duplicate the file descriptor `oldfd`, using the file descriptor specified by `newfd` instead
/// of the lowest available one.
///
/// WARNING: The new file descriptor does NOT have its close-on-exec flag set! To set the
/// close-on-exec flag atomically, use [`dup3()`] (on platforms that support it).
///
/// If `newfd` was open, it is silently closed and any errors are ignored.
///
/// # Safety
///
/// If `newfd` was an open file descriptor, it must not be in use by other sections of code;
/// otherwise they may attempt to perform operations on it or close it.
#[inline]
pub unsafe fn dup2(oldfd: RawFd, newfd: RawFd) -> Result<FileDesc> {
    Ok(FileDesc::new(Error::unpack(libc::dup2(oldfd, newfd))?))
}

/// A variant of [`dup2()`] with a `flags` argument.
///
/// This differs from [`dup2()`] in the following ways:
/// 1. If `oldfd == newfd`, this will fail with EINVAL.
/// 2. If `O_CLOEXEC` is specified in `flags`, the close-on-exec flag will be set for the new
///    file descriptor.
///
/// # Safety
///
/// See [`dup2()`].
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
))]
#[inline]
pub unsafe fn dup3(oldfd: RawFd, newfd: RawFd, flags: OFlag) -> Result<FileDesc> {
    Ok(FileDesc::new(Error::unpack(sys::dup3(
        oldfd,
        newfd,
        flags.bits(),
    ))?))
}

/// Create a pipe.
///
/// WARNING: The new file descriptors do NOT have their close-on-exec flag set! To set the
/// close-on-exec flag atomically, use [`pipe2()`] (on platforms that support it).
///
/// On success, this returns a `(read end, write end)` tuple. Data can be written to the write end,
/// and it will be buffered until it is read from the read end.
#[inline]
pub fn pipe() -> Result<(FileDesc, FileDesc)> {
    unsafe {
        let mut fds = [0; 2];
        Error::unpack_nz(libc::pipe(fds.as_mut_ptr()))?;
        Ok((FileDesc::new(fds[0]), FileDesc::new(fds[1])))
    }
}

/// Create a pipe, specifying flags to alter behavior of the pipe.
///
/// The following flags can be specified:
///
/// - `O_CLOEXEC`: Atomically set the close-on-exec flag on both new file descriptors.
/// - `O_NONBLOCK`: Atomically set the non-blocking flag on both new file descriptors.
/// - `O_DIRECT` (Linux 3.4+): Create a pipe in "packet" mode; see `pipe2(2)` for more information.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
))]
#[inline]
pub fn pipe2(flags: OFlag) -> Result<(FileDesc, FileDesc)> {
    unsafe {
        let mut fds = [0; 2];
        Error::unpack_nz(libc::pipe2(fds.as_mut_ptr(), flags.bits()))?;
        Ok((FileDesc::new(fds[0]), FileDesc::new(fds[1])))
    }
}

#[inline]
pub fn gethostname(buf: &mut [u8]) -> Result<&CStr> {
    Error::unpack_nz(unsafe { libc::gethostname(buf.as_mut_ptr() as *mut _, buf.len()) })?;

    if let Some(index) = memchr(buf, 0) {
        // It's nul-terminated
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(&buf[..index + 1]) })
    } else if let Some(end_ptr) = buf.last_mut() {
        // Add a nul at the end
        *end_ptr = 0;
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(buf) })
    } else {
        // The buffer was empty (!).
        Err(Error::from_code(libc::ENAMETOOLONG))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn gethostname_alloc() -> Result<CString> {
    // _SC_HOST_NAME_MAX may not take the trailing NUL byte into account
    let maxlen = sysconf(SysconfName::HOST_NAME_MAX).unwrap_or(1024) + 1;

    let mut buf = Vec::with_capacity(maxlen);
    unsafe {
        buf.set_len(maxlen);
    }

    Error::unpack(unsafe { libc::gethostname(buf.as_mut_ptr() as *mut _, buf.len()) })?;

    util::cstring_from_buf(buf).ok_or_else(|| Error::from_code(libc::ENAMETOOLONG))
}

#[inline]
pub fn sethostname<S: AsRef<OsStr>>(s: S) -> Result<()> {
    let buf = s.as_ref().as_bytes();

    // FreeBSD-based systems have the length as type `int`; check for overflow on 64-bit
    #[cfg(all(target_pointer_width = "64", any(freebsdlike, apple)))]
    if buf.len() > libc::c_int::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    Error::unpack_nz(unsafe { libc::sethostname(buf.as_ptr() as *const _, buf.len() as _) })
}

#[cfg(not(target_os = "android"))]
#[inline]
pub fn getdomainname(buf: &mut [u8]) -> Result<&CStr> {
    // FreeBSD-based systems have the length as type `int`; check for overflow on 64-bit
    #[cfg(all(target_pointer_width = "64", any(freebsdlike, apple)))]
    let buf = if buf.len() > libc::c_int::MAX as usize {
        &mut buf[..libc::c_int::MAX as usize]
    } else {
        buf
    };

    Error::unpack_nz(unsafe { libc::getdomainname(buf.as_mut_ptr() as *mut _, buf.len() as _) })?;

    if let Some(index) = memchr(buf, 0) {
        // It's nul-terminated
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(&buf[..index + 1]) })
    } else if let Some(end_ptr) = buf.last_mut() {
        // Add a nul at the end
        *end_ptr = 0;
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(buf) })
    } else {
        // The buffer was empty (!).
        Err(Error::from_code(libc::ENAMETOOLONG))
    }
}

#[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", not(target_os = "android")))))]
#[cfg(all(feature = "alloc", not(target_os = "android")))]
pub fn getdomainname_alloc() -> Result<CString> {
    // On Linux, the maximum length of an NIS domainname (with the terminating NUL) is 64 bytes
    #[allow(non_upper_case_globals)]
    #[cfg(linuxlike)]
    const maxlen: usize = 64;
    // On most other OSes, the limit is the same as for gethostname()
    #[cfg(not(linuxlike))]
    let maxlen = sysconf(SysconfName::HOST_NAME_MAX).unwrap_or(1024) + 1;

    let mut buf = Vec::with_capacity(maxlen);
    unsafe {
        buf.set_len(maxlen);
    }

    Error::unpack(unsafe { libc::getdomainname(buf.as_mut_ptr() as *mut _, buf.len() as _) })?;

    util::cstring_from_buf(buf).ok_or_else(|| Error::from_code(libc::ENAMETOOLONG))
}

#[cfg(not(target_os = "android"))]
#[inline]
pub fn setdomainname<S: AsRef<OsStr>>(s: S) -> Result<()> {
    let buf = s.as_ref().as_bytes();

    // FreeBSD-based systems have the length as type `int`; check for overflow on 64-bit
    #[cfg(all(target_pointer_width = "64", any(freebsdlike, apple)))]
    if buf.len() > libc::c_int::MAX as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    Error::unpack_nz(unsafe { libc::setdomainname(buf.as_ptr() as *const _, buf.len() as _) })
}

/// Check if the given file descriptor refers to a terminal.
///
/// This function returns `Ok(true)` if the file descriptor refers to a TTY, `Ok(false)` if it does
/// not, and `Err(e)` if an error occurs while checking (for example, if `fd` is not valid).
///
/// See also [`isatty_raw()`].
#[inline]
pub fn isatty(fd: RawFd) -> Result<bool> {
    if unsafe { libc::isatty(fd) } == 1 {
        Ok(true)
    } else {
        match get_errno() {
            libc::ENOTTY => Ok(false),
            eno => Err(Error::from_code(eno)),
        }
    }
}

/// Check if the given file descriptor refers to a terminal (lower-level).
///
/// 1. If the given file descriptor is a TTY, this function returns `Ok(())`.
/// 2. If the given file descriptor is not a TTY, this function returns an `Error` with the code
///    `ENOTTY`.
/// 3. If an error occurred, this function returns another `Error`.
#[inline]
pub fn isatty_raw(fd: RawFd) -> Result<()> {
    if unsafe { libc::isatty(fd) } == 1 {
        Ok(())
    } else {
        Err(Error::last())
    }
}

/// Check if the given file descriptor refers to a terminal (simple check).
///
/// This returns `true` if the given file descriptor is a terminal, and `false` if it is not OR if
/// an error occurred.
#[inline]
pub fn isatty_simple(fd: RawFd) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

/// Get the path to the specified terminal device.
///
/// **WARNING**: It is **highly recommended** to use [`ttyname_r()`] instead.
///
/// # Safety
///
/// The string returned by this function is only valid until the next call to `ttyname()`. That
/// means that not only is the arbitrary `'a` lifetime not entirely accurate, but calling this in a
/// multithreaded program is not safe.
///
/// Again, it is recommended to use [`ttyname_r()`] instead.
#[inline]
pub unsafe fn ttyname<'a>(fd: RawFd) -> Result<&'a CStr> {
    let ptr = libc::ttyname(fd);

    if ptr.is_null() {
        Err(Error::last())
    } else {
        Ok(CStr::from_ptr(ptr))
    }
}

/// Get the path to the specified terminal device.
#[inline]
pub fn ttyname_r(fd: RawFd, buf: &mut [u8]) -> Result<&CStr> {
    match unsafe { libc::ttyname_r(fd, buf.as_mut_ptr() as *mut _, buf.len()) } {
        0 => Ok(util::cstr_from_buf(buf).unwrap()),
        eno => Err(Error::from_code(eno)),
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn ttyname_alloc(fd: RawFd) -> Result<CString> {
    let maxlen = crate::sysconf(crate::SysconfName::TTY_NAME_MAX).unwrap_or(100);

    let mut buf = Vec::with_capacity(maxlen);
    unsafe {
        buf.set_len(maxlen);
    }

    let len = ttyname_r(fd, &mut buf)?.to_bytes().len();

    buf.truncate(len);
    Ok(unsafe { CString::from_vec_unchecked(buf) })
}

/// Get the name of the user logged in on this process's controlling terminal.
///
/// **WARNING**: It is **highly recommended** to use [`getlogin_r()`] instead on platforms where it
/// is supported (like Linux and the BSDs).
///
/// # Safety
///
/// The string returned by this function is only valid until the next call to `getlogin()`. That
/// means that not only is the arbitrary `'a` lifetime not entirely accurate, but calling this in a
/// multithreaded program is not safe.
///
/// Again, it is recommended to use [`getlogin_r()`] instead if it is supported.
#[inline]
pub unsafe fn getlogin<'a>() -> Result<&'a CStr> {
    let ptr = libc::getlogin();

    if ptr.is_null() {
        Err(Error::last())
    } else {
        Ok(CStr::from_ptr(ptr))
    }
}

/// Get the name of the user logged in on this process's controlling terminal.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    )))
)]
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
))]
#[inline]
pub fn getlogin_r(buf: &mut [u8]) -> Result<&CStr> {
    match unsafe { sys::getlogin_r(buf.as_mut_ptr() as *mut _, buf.len()) } {
        0 => Ok(util::cstr_from_buf(buf).unwrap()),
        eno => Err(Error::from_code(eno)),
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "alloc",
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly",
        )
    )))
)]
#[cfg(all(
    feature = "alloc",
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    )
))]
pub fn getlogin_alloc() -> Result<CString> {
    let maxlen = crate::sysconf(crate::SysconfName::LOGIN_NAME_MAX).unwrap_or(100);

    let mut buf = Vec::with_capacity(maxlen);
    unsafe {
        buf.set_len(maxlen);
    }

    let len = getlogin_r(&mut buf)?.to_bytes().len();

    buf.truncate(len);
    Ok(unsafe { CString::from_vec_unchecked(buf) })
}

/// Set the name of the user logged in on this process's controlling terminal.
///
/// This can only be called by the superuser.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    )))
)]
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
))]
#[inline]
pub fn setlogin<N: AsPath>(name: N) -> Result<()> {
    name.with_cstr(|name| Error::unpack_nz(unsafe { sys::setlogin(name.as_ptr()) }))
}

#[inline]
pub fn getcwd(buf: &mut [u8]) -> Result<&CStr> {
    if unsafe { libc::getcwd(buf.as_mut_ptr() as *mut _, buf.len()) }.is_null() {
        Err(Error::last())
    } else {
        Ok(util::cstr_from_buf(buf).unwrap())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn getcwd_alloc() -> Result<CString> {
    let mut buf = Vec::with_capacity(crate::PATH_MAX);
    unsafe {
        buf.set_len(crate::PATH_MAX);
    }

    if unsafe { libc::getcwd(buf.as_mut_ptr() as *mut _, buf.len()) }.is_null() {
        return Err(Error::last());
    }

    Ok(util::cstring_from_buf(buf).unwrap())
}

#[inline]
pub fn unlink<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::unlink(path.as_ptr()) }))
}

#[inline]
pub fn rmdir<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::rmdir(path.as_ptr()) }))
}

#[inline]
pub fn unlinkat<P: AsPath>(dfd: RawFd, path: P, flags: crate::AtFlag) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { libc::unlinkat(dfd, path.as_ptr(), flags.bits()) })
    })
}

bitflags::bitflags! {
    pub struct AccessMode: libc::c_int {
        const F_OK = libc::F_OK;
        const R_OK = libc::R_OK;
        const W_OK = libc::W_OK;
        const X_OK = libc::X_OK;
    }
}

#[inline]
pub fn access<P: AsPath>(path: P, mode: AccessMode) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::access(path.as_ptr(), mode.bits()) }))
}

#[inline]
pub fn faccessat<P: AsPath>(
    dirfd: RawFd,
    path: P,
    mode: AccessMode,
    flags: crate::AtFlag,
) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::faccessat(dirfd, path.as_ptr(), mode.bits(), flags.bits())
        })
    })
}

#[inline]
pub fn chown<P: AsPath>(
    path: P,
    owner: Option<libc::uid_t>,
    group: Option<libc::gid_t>,
) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::chown(
                path.as_ptr(),
                owner.unwrap_or(libc::uid_t::MAX),
                group.unwrap_or(libc::gid_t::MAX),
            )
        })
    })
}

#[inline]
pub fn lchown<P: AsPath>(
    path: P,
    owner: Option<libc::uid_t>,
    group: Option<libc::gid_t>,
) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::lchown(
                path.as_ptr(),
                owner.unwrap_or(libc::uid_t::MAX),
                group.unwrap_or(libc::gid_t::MAX),
            )
        })
    })
}

#[inline]
pub fn fchown(fd: RawFd, owner: Option<libc::uid_t>, group: Option<libc::gid_t>) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::fchown(
            fd,
            owner.unwrap_or(libc::uid_t::MAX),
            group.unwrap_or(libc::gid_t::MAX),
        )
    })
}

#[inline]
pub fn fchownat<P: AsPath>(
    fd: RawFd,
    path: P,
    owner: Option<libc::uid_t>,
    group: Option<libc::gid_t>,
    flags: crate::AtFlag,
) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            libc::fchownat(
                fd,
                path.as_ptr(),
                owner.unwrap_or(libc::uid_t::MAX),
                group.unwrap_or(libc::gid_t::MAX),
                flags.bits(),
            )
        })
    })
}

pub fn chmod<P: AsPath>(path: P, mode: u32) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::chmod(path.as_ptr(), mode as _) }))
}

pub fn fchmod(fd: RawFd, mode: u32) -> Result<()> {
    Error::unpack_nz(unsafe { libc::fchmod(fd, mode as _) })
}

pub fn fchmodat<P: AsPath>(dirfd: RawFd, path: P, mode: u32, flags: crate::AtFlag) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { libc::fchmodat(dirfd, path.as_ptr(), mode as _, flags.bits()) })
    })
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(bsd)]
#[inline]
pub fn getpeereid(sock: RawFd) -> Result<(libc::uid_t, libc::gid_t)> {
    let mut uid = MaybeUninit::uninit();
    let mut gid = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::getpeereid(sock, uid.as_mut_ptr(), gid.as_mut_ptr()) })?;
    unsafe { Ok((uid.assume_init(), gid.assume_init())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    fn tgkill(tgid: libc::pid_t, tid: libc::pid_t, sig: libc::c_int) -> Result<()> {
        Error::unpack_nz(unsafe { libc::syscall(libc::SYS_tgkill, tgid, tid, sig) } as _)
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_getpids_valid() {
        // Check that each is valid:

        // The current thread
        tgkill(getpid(), gettid(), 0).unwrap();
        // The main thread in the current process
        tgkill(getpid(), getpid(), 0).unwrap();
        // The main thread in the parent process
        tgkill(getppid(), getppid(), 0).unwrap();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_getpid() {
        assert_eq!(getpid() as u32, std::process::id());
    }

    #[test]
    fn test_pagesize() {
        assert_eq!(getpagesize(), unsafe {
            libc::sysconf(libc::_SC_PAGESIZE) as usize
        });

        assert_eq!(getpagesize(), sysconf(SysconfName::PAGESIZE).unwrap());
    }

    #[test]
    fn test_pgid_sid() {
        assert_eq!(getpgid(0).unwrap(), getpgrp());
        assert_eq!(getpgid(getpid()).unwrap(), getpgrp());

        assert!(matches!(getpgid(1).unwrap(), 0 | 1));

        assert_eq!(getpgid(libc::pid_t::MAX).unwrap_err().code(), libc::ESRCH);

        assert_eq!(getsid(libc::pid_t::MAX).unwrap_err().code(), libc::ESRCH);

        if getpgrp() != getpid() {
            // Not a process group leader
            setsid().unwrap();

            // If setsid() succeeded, the session ID should match the process ID
            assert_eq!(getsid(0).unwrap(), getpid());
            assert_eq!(getsid(getpid()).unwrap(), getpid());

            // And the process group ID should also match
            assert_eq!(getpgrp(), getpid());
            assert_eq!(getpgid(0).unwrap(), getpid());
            assert_eq!(getpgid(getpid()).unwrap(), getpid());
        }

        // Now that we're a process group leader, setsid() should fail with EPERM
        assert_eq!(setsid().unwrap_err().code(), libc::EPERM);
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    ))]
    #[test]
    fn test_resids() {
        let uid = getuid();
        assert_eq!(geteuid(), uid);
        assert_eq!(getresuid(), (uid, uid, uid));

        let gid = getgid();
        assert_eq!(getegid(), gid);
        assert_eq!(getresgid(), (gid, gid, gid));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_getgroups() {
        let mut buf = [0; 65536];
        let n = getgroups(&mut buf).unwrap();

        assert_eq!(getgroups_alloc().unwrap(), &buf[..n]);

        if n > 0 {
            let mut buf = Vec::new();
            buf.resize(n - 1, 0);
            assert_eq!(getgroups(&mut buf).unwrap_err().code(), libc::EINVAL);
        }
    }

    #[test]
    fn test_getcwd_error() {
        assert_eq!(getcwd(&mut []).unwrap_err().code(), libc::EINVAL);
        assert_eq!(getcwd(&mut [0]).unwrap_err().code(), libc::ERANGE);
    }

    #[test]
    fn test_access() {
        access(CStr::from_bytes_with_nul(b".\0").unwrap(), AccessMode::F_OK).unwrap();
        access(
            CStr::from_bytes_with_nul(b".\0").unwrap(),
            AccessMode::R_OK | AccessMode::X_OK,
        )
        .unwrap();

        assert_eq!(
            access(
                CStr::from_bytes_with_nul(b"/NOEXIST\0").unwrap(),
                AccessMode::F_OK
            )
            .unwrap_err()
            .code(),
            libc::ENOENT
        );
        assert_eq!(
            access(
                CStr::from_bytes_with_nul(b"/NOEXIST\0").unwrap(),
                AccessMode::R_OK | AccessMode::W_OK | AccessMode::X_OK
            )
            .unwrap_err()
            .code(),
            libc::ENOENT
        );
    }
}
