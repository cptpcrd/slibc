use crate::internal_prelude::*;
use crate::{OFlag, SigSet};

/// A list of file-related actions to be performed in a child launched by
/// [`posix_spawn_raw()`]/[`posix_spawnp_raw()`]/[`posix_spawn()`]/[`posix_spawnp()`].
///
/// Each method of this struct will add one action. Actions are performed in the order they are
/// added.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
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

    /// Add a file action to mark the specified file descriptor as inheritable.
    ///
    /// The specified file descriptor will have its close-on-exec flag unset inside the child.
    ///
    /// This is especially useful in combination with [`PosixSpawnFlags::CLOEXEC_DEFAULT`].
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
    #[cfg(apple)]
    #[inline]
    pub fn addinherit_np(&mut self, fd: RawFd) -> Result<()> {
        Error::unpack_eno(unsafe { sys::posix_spawn_file_actions_addinherit_np(&mut self.0, fd) })
    }
}

#[cfg(target_os = "linux")]
static ADDCHDIR: util::DlFuncLoader<
    unsafe extern "C" fn(*mut libc::posix_spawn_file_actions_t, *const libc::c_char) -> libc::c_int,
> = unsafe { util::DlFuncLoader::new(b"posix_spawn_file_actions_addchdir_np\0") };

#[cfg(target_os = "linux")]
static ADDFCHDIR: util::DlFuncLoader<
    unsafe extern "C" fn(*mut libc::posix_spawn_file_actions_t, libc::c_int) -> libc::c_int,
> = unsafe { util::DlFuncLoader::new(b"posix_spawn_file_actions_addfchdir_np\0") };

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
impl PosixSpawnFileActions {
    /// Check whether [`Self::addchdir_np()`] is supported by the running libc.
    ///
    /// If this function returns `true`, [`Self::addchdir_np()`] will NOT fail with `ENOSYS`.
    #[inline]
    pub fn has_addchdir_np() -> bool {
        ADDCHDIR.get().is_some()
    }

    /// Check whether [`Self::addfchdir_np()`] is supported by the running libc.
    ///
    /// If this function returns `true`, [`Self::addfchdir_np()`] will NOT fail with `ENOSYS`.
    #[inline]
    pub fn has_addfchdir_np() -> bool {
        ADDFCHDIR.get().is_some()
    }

    /// Add a a file action to change the child's working directory to a specified path.
    ///
    /// This wrapper may fail with `EINVAL` if `path` contains an interior NUL byte, or with
    /// `ENOSYS` if the current libc does not have the `posix_spawn_file_actions_addchdir_np()`
    /// function. It will also fail with `ENOSYS` in statically linked programs.
    #[inline]
    pub fn addchdir_np<P: AsPath>(&mut self, path: P) -> Result<()> {
        path.with_cstr(|path| {
            Error::unpack_eno(unsafe {
                if let Some(func) = ADDCHDIR.get() {
                    func(&mut self.0, path.as_ptr())
                } else {
                    libc::ENOSYS
                }
            })
        })
    }

    /// Add a a file action to change the child's working directory to the directory referred to by
    /// the given `fd`.
    ///
    /// This wrapper may fail with `ENOSYS` if the current libc does not have the
    /// `posix_spawn_file_actions_addfchdir_np()` function. It will also fail with `ENOSYS` in
    /// statically linked programs.
    #[inline]
    pub fn addfchdir_np(&mut self, fd: RawFd) -> Result<()> {
        Error::unpack_eno(unsafe {
            if let Some(func) = ADDFCHDIR.get() {
                func(&mut self.0, fd)
            } else {
                libc::ENOSYS
            }
        })
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
    #[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
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

        /// Behave as if all file descriptors had the close-on-exec flag set on them.
        ///
        /// If this flag is specified, it is necessary to mark any file descriptors that need to be
        /// inherited (including stdin/stdout/stderr!) using
        /// [`PosixSpawnFileActions::addinherit_np()`].
        #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
        #[cfg(apple)]
        const CLOEXEC_DEFAULT = sys::POSIX_SPAWN_CLOEXEC_DEFAULT as libc::c_short;
    }
}

/// A set of attributes for a child launched by
/// [`posix_spawn_raw()`]/[`posix_spawnp_raw()`]/[`posix_spawn()`]/[`posix_spawnp()`].
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
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
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
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
/// 1. See [`posix_spawn_raw()`].
/// 2. See [`posix_spawnp()`].
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
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

/// Call `posix_spawn(3)` to launch a new process.
///
/// This is identical to [`posix_spawn_raw()`], except that it accepts
/// [`CStringVec`](./struct.CStringVec.html)s instead of raw pointers, which allows it to be safe.
///
/// # Panics
///
/// Panics if `envp` is `None`. This would normally translate to passing NULL, which would preserve
/// the current environment. However, it cannot currently be done safely. See item (1) in
/// [`execvp()`](./fn.execvp.html)'s [safety section](./fn.execvp.html#safety).
#[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", not(target_os = "android")))))]
#[cfg(feature = "alloc")]
#[inline]
pub fn posix_spawn<P: AsPath>(
    prog: P,
    file_actions: Option<&PosixSpawnFileActions>,
    attr: Option<&PosixSpawnAttr>,
    argv: &crate::CStringVec,
    envp: Option<&crate::CStringVec>,
) -> Result<libc::pid_t> {
    unsafe {
        posix_spawn_raw(
            prog,
            file_actions,
            attr,
            argv.as_ptr(),
            envp.unwrap().as_ptr(),
        )
    }
}

/// Call `posix_spawnp(3)` to launch a new process.
///
/// This is identical to [`posix_spawn_raw()`], except that it accepts
/// [`CStringVec`](./struct.CStringVec.html)s instead of raw pointers, which allows it to be safer.
///
/// # Safety
///
/// See item (2) in [`execvp()`](./fn.execvp.html)'s [safety section](./fn.execvp.html#safety).
///
/// Also, if you pass `envp` as `None` (which translates to passing NULL, which preserves the
/// current environment), see item (1).
#[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", not(target_os = "android")))))]
#[cfg(feature = "alloc")]
#[inline]
pub unsafe fn posix_spawnp<P: AsPath>(
    prog: P,
    file_actions: Option<&PosixSpawnFileActions>,
    attr: Option<&PosixSpawnAttr>,
    argv: &crate::CStringVec,
    envp: Option<&crate::CStringVec>,
) -> Result<libc::pid_t> {
    posix_spawnp_raw(
        prog,
        file_actions,
        attr,
        argv.as_ptr(),
        envp.map_or_else(core::ptr::null, |e| e.as_ptr()),
    )
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "std")]
    use crate::{pipe_cloexec, waitpid, WaitFlags};
    #[cfg(feature = "std")]
    use std::io::Read;

    #[cfg(feature = "std")]
    fn posix_spawn_simple<P: AsPath, S: AsPath, I: Iterator<Item = S>>(
        prog: P,
        factions: Option<PosixSpawnFileActions>,
        attr: Option<&PosixSpawnAttr>,
        argv: I,
    ) -> Result<(libc::pid_t, FileDesc, FileDesc, FileDesc)> {
        let mut factions = factions.unwrap_or_else(|| PosixSpawnFileActions::new().unwrap());

        let (in_r, in_w) = pipe_cloexec().unwrap();
        let (out_r, out_w) = pipe_cloexec().unwrap();
        let (err_r, err_w) = pipe_cloexec().unwrap();
        in_r.set_cloexec(false).unwrap();
        out_w.set_cloexec(false).unwrap();
        err_w.set_cloexec(false).unwrap();

        factions.adddup2(in_r.fd(), 0).unwrap();
        factions.adddup2(out_w.fd(), 1).unwrap();
        factions.adddup2(err_w.fd(), 2).unwrap();
        factions.addclose(in_r.fd()).unwrap();
        factions.addclose(out_w.fd()).unwrap();
        factions.addclose(err_w.fd()).unwrap();

        let env = std::env::vars_os()
            .map(|(mut key, val)| {
                key.push("=");
                key.push(val);
                CString::new(key.into_vec()).unwrap()
            })
            .collect();

        let pid = posix_spawn(
            prog,
            Some(&factions),
            attr,
            &argv
                .into_iter()
                .map(|s| CString::new(s.as_os_str().as_bytes()).unwrap())
                .collect(),
            Some(&env),
        )?;

        Ok((pid, in_w, out_r, err_r))
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_posix_spawn_basic() {
        let (pid, stdin, mut stdout, _) =
            posix_spawn_simple("/bin/sh", None, None, ["sh", "-c", "cat"].iter().copied()).unwrap();

        stdin.write_all(b"abc").unwrap();
        drop(stdin);
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"abc");
        drop(stdout);

        waitpid(pid, WaitFlags::empty()).unwrap();
    }

    #[cfg(all(feature = "std", target_os = "linux"))]
    #[test]
    fn test_posix_spawn_chdir() {
        if PosixSpawnFileActions::has_addchdir_np() {
            assert!(PosixSpawnFileActions::has_addfchdir_np());
        } else {
            assert!(!PosixSpawnFileActions::has_addfchdir_np());
            let mut factions = PosixSpawnFileActions::new().unwrap();
            assert_eq!(factions.addchdir_np("/").unwrap_err(), Errno::ENOSYS);
            assert_eq!(factions.addfchdir_np(-1).unwrap_err(), Errno::ENOSYS);
            return;
        }

        // Add a chdir("/")
        let mut factions = PosixSpawnFileActions::new().unwrap();
        factions.addchdir_np("/").unwrap();
        let (pid, _, mut stdout, _) = posix_spawn_simple(
            "/bin/sh",
            Some(factions),
            None,
            ["sh", "-c", "pwd"].iter().copied(),
        )
        .unwrap();

        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"/\n");
        drop(stdout);

        waitpid(pid, WaitFlags::empty()).unwrap();

        // open("/"), then fchdir(), then close() it
        let mut factions = PosixSpawnFileActions::new().unwrap();
        let (fdesc, _) = crate::pipe().unwrap();
        factions
            .addopen(fdesc.fd(), "/", OFlag::O_DIRECTORY | OFlag::O_RDONLY, 0)
            .unwrap();
        factions.addfchdir_np(fdesc.fd()).unwrap();
        factions.addclose(fdesc.fd()).unwrap();

        let (pid, _, mut stdout, _) = posix_spawn_simple(
            "/bin/sh",
            Some(factions),
            None,
            ["sh", "-c", "pwd"].iter().copied(),
        )
        .unwrap();

        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"/\n");
        drop(stdout);

        waitpid(pid, WaitFlags::empty()).unwrap();
    }

    #[test]
    fn test_posix_spawnattr_modify() {
        use crate::{sigset, Signal};

        let mut attr = PosixSpawnAttr::new().unwrap();

        assert_eq!(attr.getflags().unwrap(), PosixSpawnFlags::empty());
        for &flags in [
            PosixSpawnFlags::SETPGROUP,
            PosixSpawnFlags::empty(),
            PosixSpawnFlags::SETSIGDEF | PosixSpawnFlags::SETSIGMASK,
        ]
        .iter()
        {
            attr.setflags(flags).unwrap();
            assert_eq!(attr.getflags().unwrap(), flags);
        }

        assert_eq!(attr.getpgroup().unwrap(), 0);
        attr.setpgroup(123).unwrap();
        assert_eq!(attr.getpgroup().unwrap(), 123);
        attr.setpgroup(0).unwrap();
        assert_eq!(attr.getpgroup().unwrap(), 0);

        assert!(attr.getsigmask().unwrap().is_empty());
        attr.setsigmask(&sigset!(Signal::SIGINT)).unwrap();
        assert_eq!(attr.getsigmask().unwrap(), sigset!(Signal::SIGINT));
        attr.setsigmask(&sigset!()).unwrap();
        assert!(attr.getsigmask().unwrap().is_empty());

        assert!(attr.getsigdefault().unwrap().is_empty());
        attr.setsigdefault(&sigset!(Signal::SIGINT)).unwrap();
        assert_eq!(attr.getsigdefault().unwrap(), sigset!(Signal::SIGINT));
        attr.setsigdefault(&sigset!()).unwrap();
        assert!(attr.getsigdefault().unwrap().is_empty());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_posix_spawnattr_setpgroup() {
        let (pid, _, _, _) = posix_spawn_simple(
            "/bin/sh",
            None,
            None,
            ["sh", "-c", "sleep 10"].iter().copied(),
        )
        .unwrap();
        assert_eq!(crate::getpgid(pid).unwrap(), crate::getpgrp());
        assert_eq!(crate::getsid(pid).unwrap(), crate::getsid(0).unwrap());
        crate::kill(pid, crate::Signal::SIGTERM).unwrap();
        waitpid(pid, WaitFlags::empty()).unwrap();

        let mut attr = PosixSpawnAttr::new().unwrap();
        attr.setflags(PosixSpawnFlags::SETPGROUP).unwrap();
        let (pid, _, _, _) = posix_spawn_simple(
            "/bin/sh",
            None,
            Some(&attr),
            ["sh", "-c", "sleep 10"].iter().copied(),
        )
        .unwrap();
        assert_eq!(crate::getpgid(pid).unwrap(), pid);
        assert_eq!(crate::getsid(pid).unwrap(), crate::getsid(0).unwrap());
        crate::kill(pid, crate::Signal::SIGTERM).unwrap();
        waitpid(pid, WaitFlags::empty()).unwrap();
    }

    #[cfg(all(feature = "std", any(target_os = "linux", apple)))]
    #[test]
    fn test_posix_spawnattr_setsid() {
        let mut attr = PosixSpawnAttr::new().unwrap();
        attr.setflags(PosixSpawnFlags::SETSID).unwrap();
        let (pid, _, _, _) = posix_spawn_simple(
            "/bin/sh",
            None,
            Some(&attr),
            ["sh", "-c", ""].iter().copied(),
        )
        .unwrap();
        assert_eq!(crate::getpgid(pid).unwrap(), pid);
        assert_eq!(crate::getsid(pid).unwrap(), pid);
        waitpid(pid, WaitFlags::empty()).unwrap();
    }
}
