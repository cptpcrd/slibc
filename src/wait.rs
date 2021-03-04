use crate::internal_prelude::*;

bitflags::bitflags! {
    pub struct WaitFlags: libc::c_int {
        const WNOHANG = libc::WNOHANG;
        const WUNTRACED = libc::WUNTRACED;
        const WCONTINUED = libc::WCONTINUED;
        const WEXITED = libc::WEXITED;
        const WSTOPPED = libc::WSTOPPED;
        const WNOWAIT = libc::WNOWAIT;
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum WaitStatus {
    Exited(i32),
    Signaled(i32, bool),
    Stopped(i32),
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[cfg(linuxlike)]
    PtraceEvent(i32, i32),
    Continued,
}

impl WaitStatus {
    #[inline]
    fn from_raw(stat: i32) -> Self {
        if !libc::WIFEXITED(stat) {
            if libc::WIFSIGNALED(stat) {
                let sig = libc::WTERMSIG(stat);

                #[cfg(linuxlike)]
                if sig == libc::SIGTRAP {
                    let event = (stat >> 16) & 0xF;
                    if event != 0 {
                        return Self::PtraceEvent(sig, event);
                    }
                }

                return Self::Signaled(sig, libc::WCOREDUMP(stat));
            } else if libc::WIFSTOPPED(stat) {
                return Self::Stopped(libc::WSTOPSIG(stat));
            } else if libc::WIFCONTINUED(stat) {
                return Self::Continued;
            }

            // Something's wrong... just fall through to Exited
        }

        Self::Exited(libc::WEXITSTATUS(stat))
    }
}

#[inline]
pub fn wait() -> Result<(libc::pid_t, WaitStatus)> {
    let mut wstat = MaybeUninit::uninit();
    let pid = Error::unpack(unsafe { libc::wait(wstat.as_mut_ptr()) })?;
    Ok((pid, WaitStatus::from_raw(unsafe { wstat.assume_init() })))
}

#[inline]
pub fn waitpid(pid: libc::pid_t, options: WaitFlags) -> Result<Option<(libc::pid_t, WaitStatus)>> {
    let mut wstat = MaybeUninit::uninit();
    let pid = Error::unpack(unsafe { libc::waitpid(pid, wstat.as_mut_ptr(), options.bits()) })?;

    if pid == 0 {
        Ok(None)
    } else {
        Ok(Some((
            pid,
            WaitStatus::from_raw(unsafe { wstat.assume_init() }),
        )))
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(linuxlike, freebsdlike, apple))] {
        #[cfg_attr(docsrs, doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "macos",
            target_os = "ios",
        ))))]
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        pub enum WaitidId {
            Pid(libc::pid_t),
            Pgid(libc::pid_t),
            All,
            #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
            #[cfg(linuxlike)]
            PidFd(i32),
        }

        impl WaitidId {
            #[inline]
            fn into_raw(self) -> (libc::idtype_t, libc::id_t) {
                match self {
                    Self::Pid(pid) => (libc::P_PID as _, pid as _),
                    Self::Pgid(pgid) => (libc::P_PGID as _, pgid as _),
                    Self::All => (libc::P_ALL, 0),
                    #[cfg(linuxlike)]
                    Self::PidFd(fd) => (libc::P_PIDFD as _, fd as _),
                }
            }
        }

        #[cfg_attr(docsrs, doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "macos",
            target_os = "ios",
        ))))]
        #[inline]
        pub fn waitid(id: WaitidId, options: WaitFlags) -> Result<Option<crate::SigInfo>> {
            let (idtype, id) = id.into_raw();
            let mut info = unsafe { core::mem::zeroed() };
            Error::unpack_nz(unsafe { libc::waitid(idtype, id, &mut info, options.bits()) })?;

            let info = crate::SigInfo(info);
            Ok(if info.si_pid() != 0 {
                Some(info)
            } else {
                None
            })
        }
    }
}
