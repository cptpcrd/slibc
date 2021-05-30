use crate::internal_prelude::*;
use crate::{OFlag, SigSet};

/// A list of file-related actions to be performed in a child launched by [`posix_spawn_raw()`] or
/// [`posix_spawnp_raw()`].
///
/// Each method of this struct will add one action. Actions are performed in the order they are
/// added.
#[derive(Debug)]
pub struct PosixSpawnFileActions(sys::posix_spawn_file_actions_t);

impl PosixSpawnFileActions {
    /// Create a new empty file action list.
    #[inline]
    pub fn new() -> Result<Self> {
        let mut actions = MaybeUninit::uninit();
        Error::unpack_eno(unsafe { sys::posix_spawn_file_actions_init(actions.as_mut_ptr()) })?;
        Ok(Self(unsafe { actions.assume_init() }))
    }

    /// Add a file action to open a file inside the child.
    ///
    /// This will cause the file at the specified `path` to be opened with the specified `flags`
    /// and `mode` at the file descriptor specified by `fd`.
    #[inline]
    pub fn addopen<P: AsPath>(
        &mut self,
        fd: RawFd,
        path: P,
        flags: OFlag,
        mode: u32,
    ) -> Result<()> {
        path.with_cstr(|path| {
            Error::unpack_eno(unsafe {
                sys::posix_spawn_file_actions_addopen(
                    &mut self.0,
                    fd,
                    path.as_ptr(),
                    flags.bits(),
                    mode as _,
                )
            })
        })
    }

    /// Add a file action to close a file descriptor inside the child.
    ///
    /// This will cause the file descriptor specified by `fd` to be closed.
    #[inline]
    pub fn addclose(&mut self, fd: RawFd) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawn_file_actions_addclose(&mut self.0, fd) })
    }

    /// Add a file action to duplicate a file descriptor inside the child.
    ///
    /// This has the same effect as calling `dup2(oldfd, newfd)` inside the child.
    ///
    /// Note: Some implementations handle the case of `oldfd == newfd` by unsetting the
    /// close-on-exec flag on the file descriptor. See OS/libc-specific documentation for details.
    #[inline]
    pub fn adddup2(&mut self, oldfd: RawFd, newfd: RawFd) -> Result<()> {
        Error::unpack_eno(unsafe {
            sys::posix_spawn_file_actions_adddup2(&mut self.0, oldfd, newfd)
        })
    }

    /// Add a a file action to change the child's working directory to a specified path.
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn addchdir_np<P: AsPath>(&mut self, path: P) -> Result<()> {
        path.with_cstr(|path| {
            Error::unpack_eno(unsafe {
                sys::posix_spawn_file_actions_addchdir_np(&mut self.0, path.as_ptr())
            })
        })
    }

    /// Add a a file action to change the child's working directory to the directory referred to by
    /// the given `fd`.
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn addfchdir_np(&mut self, fd: RawFd) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawn_file_actions_addfchdir_np(&mut self.0, fd) })
    }
}

impl Drop for PosixSpawnFileActions {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::posix_spawn_file_actions_destroy(&mut self.0);
        }
    }
}

impl AsRef<sys::posix_spawn_file_actions_t> for PosixSpawnFileActions {
    #[inline]
    fn as_ref(&self) -> &sys::posix_spawn_file_actions_t {
        &self.0
    }
}

bitflags::bitflags! {
    /// Flags for [`PosixSpawnAttr`].
    ///
    /// These flags a) modify attributes of the spawned process and b) change how the attributes in
    /// [`PosixSpawnAttr`] are applied.
    pub struct PosixSpawnFlags: libc::c_short {
        /// Set the effective UID/GID of the child to the real UID/GID of this process.
        const RESETIDS = sys::POSIX_SPAWN_RESETIDS as libc::c_short;
        /// Set the process group ID of the child to the value of the `pgroup` attribute.
        ///
        /// See [`PosixSpawnAttr::setpgroup()`].
        const SETPGROUP = sys::POSIX_SPAWN_SETPGROUP as libc::c_short;
        /// Reset all of the signals in the signal mask specified by the `sigdefault` attribute
        /// to the default disposition.
        ///
        /// See [`PosixSpawnAttr::setsigdefault()`].
        const SETSIGDEF = sys::POSIX_SPAWN_SETSIGDEF as libc::c_short;
        /// Set the signal mask of the child to the value of the `sigmask` attribute.
        ///
        /// See [`PosixSpawnAttr::setsigmask()`].
        const SETSIGMASK = sys::POSIX_SPAWN_SETSIGMASK as libc::c_short;

        /// Make the child process the session leader of a new session.
        ///
        /// This is equivalent to calling `setsid(2)` inside the child.
        #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "macos", target_os = "ios"))))]
        #[cfg(any(target_os = "linux", apple))]
        const SETSID = sys::POSIX_SPAWN_SETSID as libc::c_short;
    }
}

/// A set of attributes for a child launched by [`posix_spawn_raw()`] or [`posix_spawnp_raw()`].
#[derive(Debug)]
pub struct PosixSpawnAttr(sys::posix_spawnattr_t);

impl PosixSpawnAttr {
    /// Create a new attribute set.
    #[inline]
    pub fn new() -> Result<Self> {
        let mut attr = MaybeUninit::uninit();
        Error::unpack_eno(unsafe { sys::posix_spawnattr_init(attr.as_mut_ptr()) })?;
        Ok(Self(unsafe { attr.assume_init() }))
    }

    /// Set the flags indicating which attributes will be changed.
    ///
    /// See [`PosixSpawnFlags`] and `posix_spawnattr_setflags(3)`.
    #[inline]
    pub fn setflags(&mut self, flags: PosixSpawnFlags) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawnattr_setflags(&mut self.0, flags.bits()) })
    }

    /// Get the flags indicating which attributes will be changed.
    ///
    /// See [`PosixSpawnFlags`] and `posix_spawnattr_getflags(3)`.
    #[inline]
    pub fn getflags(&self) -> Result<PosixSpawnFlags> {
        let mut flags = MaybeUninit::uninit();
        Error::unpack_eno(unsafe { sys::posix_spawnattr_getflags(&self.0, flags.as_mut_ptr()) })?;
        Ok(PosixSpawnFlags::from_bits_truncate(unsafe {
            flags.assume_init()
        }))
    }

    /// Set the `pgroup` attribute of the child (default 0).
    ///
    /// If the [`PosixSpawnFlags::SETPGROUP`] attribute flag is set using [`Self::setflags()`], the
    /// child's process group ID will be changed to this value. If the value is 0, the child's
    /// process group ID will be changed to its PID.
    #[inline]
    pub fn setpgroup(&mut self, pgroup: libc::pid_t) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawnattr_setpgroup(&mut self.0, pgroup) })
    }

    /// Get the `pgroup` attribute of the child.
    ///
    /// See [`Self::setpgroup()`].
    #[inline]
    pub fn getpgroup(&self) -> Result<libc::pid_t> {
        let mut pgroup = MaybeUninit::uninit();
        Error::unpack_eno(unsafe { sys::posix_spawnattr_getpgroup(&self.0, pgroup.as_mut_ptr()) })?;
        Ok(unsafe { pgroup.assume_init() })
    }

    /// Set the signal mask of the child process.
    #[inline]
    pub fn setsigmask(&mut self, mask: &SigSet) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawnattr_setsigmask(&mut self.0, mask.as_ref()) })
    }

    /// Get the `sigmask` attribute of the child.
    ///
    /// See [`Self::setsigmask()`].
    #[inline]
    pub fn getsigmask(&self) -> Result<SigSet> {
        let mut sigmask = MaybeUninit::uninit();
        Error::unpack_eno(unsafe {
            sys::posix_spawnattr_getsigmask(&self.0, sigmask.as_mut_ptr())
        })?;
        Ok(unsafe { sigmask.assume_init() }.into())
    }

    /// Set the mask of signals whose disposition will be reset to the default inside the child.
    #[inline]
    pub fn setsigdefault(&mut self, mask: &SigSet) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawnattr_setsigdefault(&mut self.0, mask.as_ref()) })
    }

    /// Get the `sigdefault` attribute of the child.
    ///
    /// See [`Self::setsigdefault()`].
    #[inline]
    pub fn getsigdefault(&self) -> Result<SigSet> {
        let mut sigdefault = MaybeUninit::uninit();
        Error::unpack_eno(unsafe {
            sys::posix_spawnattr_getsigdefault(&self.0, sigdefault.as_mut_ptr())
        })?;
        Ok(unsafe { sigdefault.assume_init() }.into())
    }
}

impl Drop for PosixSpawnAttr {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::posix_spawnattr_destroy(&mut self.0);
        }
    }
}

impl AsRef<sys::posix_spawnattr_t> for PosixSpawnAttr {
    #[inline]
    fn as_ref(&self) -> &sys::posix_spawnattr_t {
        &self.0
    }
}

/// Call `posix_spawn(3)` to launch a new process.
///
/// This function launches a child process. The modifications specified by `file_actions` and
/// `attr` (if they are not `None`) are applied, then the child process executes the specified
/// `prog` with the arguments specified in `argv` and the environment specified in `envp`. On
/// success, this function returns the PID of the child.
///
/// Note that not all platforms properly report errors that occurred in the child. They may simply
/// launch the child and return the PID, rather than waiting for it to perform all the necessary
/// modifications and call `execve()`.
///
/// If `envp` is NULL, the environment of the parent process is left unchanged.
///
/// # Safety
///
/// 1. `argv` (and `envp`, if it is not NULL) must be a valid pointer to a NULL-terminated array of
///    C strings. See `execve(2)` for more details.
/// 2. Passing NULL for `envp` is unsound in multi-threaded programs where other threads may be
///    concurrently modifying the environment. See rust-lang/rust#27970 for more information.
#[inline]
pub unsafe fn posix_spawn_raw<P: AsPath>(
    prog: P,
    file_actions: Option<&PosixSpawnFileActions>,
    attr: Option<&PosixSpawnAttr>,
    argv: *const *const libc::c_char,
    envp: *const *const libc::c_char,
) -> Result<libc::pid_t> {
    let mut pid = MaybeUninit::uninit();

    prog.with_cstr(|prog| {
        Error::unpack_eno(sys::posix_spawn(
            pid.as_mut_ptr(),
            prog.as_ptr(),
            file_actions.map_or_else(core::ptr::null, |a| &a.0),
            attr.map_or_else(core::ptr::null, |a| &a.0),
            argv as *const *mut _,
            envp as *const *mut _,
        ))
    })?;

    Ok(pid.assume_init())
}

/// Call `posix_spawnp(3)` to launch a new process.
///
/// This is identical to [`posix_spawnp_raw()`], except that if the specified `prog` does not
/// contain a slash, a search will be done through the directories listed in `PATH` (as for
/// `execvp(3)`). Note that most implementations of `posix_spawnp(3)` will use the `PATH` from the
/// current environment, not the `PATH` specified in `envp` (if any).
///
/// # Safety
///
/// 1. See [`posix_spawnp_raw()`].
/// 2. Since `posix_spawnp()` may access the old environment to do a `PATH` search, it is unsound
///    in multi-threaded programs where other threads may be concurrently modifying the
///    environment. See rust-lang/rust#27970 for more information.
#[inline]
pub unsafe fn posix_spawnp_raw<P: AsPath>(
    prog: P,
    file_actions: Option<&PosixSpawnFileActions>,
    attr: Option<&PosixSpawnAttr>,
    argv: *const *const libc::c_char,
    envp: *const *const libc::c_char,
) -> Result<libc::pid_t> {
    let mut pid = MaybeUninit::uninit();

    prog.with_cstr(|prog| {
        Error::unpack_eno(sys::posix_spawnp(
            pid.as_mut_ptr(),
            prog.as_ptr(),
            file_actions.map_or_else(core::ptr::null, |a| &a.0),
            attr.map_or_else(core::ptr::null, |a| &a.0),
            argv as *const *mut _,
            envp as *const *mut _,
        ))
    })?;

    Ok(pid.assume_init())
}
