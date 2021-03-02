use core::cmp::Ordering;
use core::fmt;
use core::time::Duration;

use crate::internal_prelude::*;

macro_rules! define_resource {
    ($(
        #[cfg($cfg:meta)]
        $(
            $(#[doc = $doc:literal])*
            $name:ident = $libc_name:ident,
        )+
    )*) => {
        /// An enum listing the resource limits available on the current platform.
        ///
        /// See getrlimit(2) for more information.
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[repr(i32)]
        pub enum Resource {
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                $(#[doc = $doc])*
                $name = libc::$libc_name as i32,
            )*)*
        }

        const RESOURCES: &[(Resource, &str)] = &[
            $($(
                #[cfg($cfg)]
                (Resource::$name, stringify!($name)),
            )*)*
        ];
    };
}

define_resource! {
    // Should be present on all POSIX systems
    #[cfg(all())]
    /// Maximum size of a core file that can be created by a process.
    CORE = RLIMIT_CORE,
    /// Maximum amount of CPU time that can be used by a process.
    CPU = RLIMIT_CPU,
    /// Maximum size of the process's data segment.
    DATA = RLIMIT_DATA,
    /// (Effectively) limits the number of files that can be opened by a process.
    NOFILE = RLIMIT_NOFILE,
    /// Maximum size of a file that can be created by a process.
    FSIZE = RLIMIT_FSIZE,
    /// Maximum size of the process's main thread's stack.
    STACK = RLIMIT_STACK,

    // OpenBSD and macOS don't have RLIMIT_AS
    #[cfg(not(any(target_os = "openbsd", target_os = "macos", target_os = "ios")))]
    /// Maximum size of the process's address space.
    AS = RLIMIT_AS,

    // Present on Linux, the BSDs, and macOS
    #[cfg(any(
        target_os = "linux",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
    ))]
    /// Maximum number of processes that may exist for this process's real user ID.
    NPROC = RLIMIT_NPROC,
    /// Maximum amount of memory that a process may lock into RAM.
    MEMLOCK = RLIMIT_MEMLOCK,
    /// Limits the process's resident set (the number of virtual pages that are in RAM).
    RSS = RLIMIT_RSS,

    // Present on most of the BSDs (but not OpenBSD)
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd"))]
    /// The maximum size of the socket buffers used by this user.
    SBSIZE = RLIMIT_SBSIZE,

    // FreeBSD-specific
    #[cfg(target_os = "freebsd")]
    /// The maximum number of kqueue instances that may exist for this process's user ID.
    KQUEUES = RLIMIT_KQUEUES,
    /// The maximum amount of swap space that may be reserved for this process's user ID.
    ///
    /// This is only enforced if bit 1 of the `vm.overcommit` sysctl is set. See tuning(7) for more
    /// information.
    SWAP = RLIMIT_SWAP,
    /// The maximum number of pseudoterminals that may exist for this process's user ID.
    NPTS = RLIMIT_NPTS,

    // NetBSD-specific
    #[cfg(target_os = "netbsd")]
    /// Maximum number of threads that may exist for this process's user ID.
    ///
    /// Does not include each process's first thread.
    NTHR = RLIMIT_NTHR,

    // DragonFly BSD-specific
    #[cfg(target_os = "dragonfly")]
    /// Maximum number of POSIX advisory-mode locks that may exist for this process's user ID.
    POSIXLOCKS = RLIMIT_POSIXLOCKS,

    // Linux-specific
    #[cfg(target_os = "linux")]
    /// The maximum number of bytes that can be allocated for POSIX message queues by this
    /// process's user ID.
    MSGQUEUE = RLIMIT_MSGQUEUE,
    /// A ceiling to which the process's nice value can be raised.
    ///
    /// See the helper functions [`rlimits::nice_rlimit_to_thresh()`] and
    /// [`rlimits::nice_thresh_to_rlimit()`] and getrlimit(2) for more information.
    NICE = RLIMIT_NICE,
    /// A ceiling to the real-time priority that can be set for this process.
    RTPRIO = RLIMIT_RTPRIO,
    /// The amount of CPU time that may be consumed by a process under real-time scheduling.
    RTTIME = RLIMIT_RTTIME,
    /// The maximum number of signals that may be queued for this process's user ID.
    SIGPENDING = RLIMIT_SIGPENDING,
}

impl Resource {
    /// Create an iterator over the `Resource`s that are available on the current platform.
    #[inline]
    pub fn iter() -> ResourceIter {
        ResourceIter(RESOURCES)
    }
}

impl fmt::Display for Resource {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// An iterator over the `Resource`s that are available on the current platform.
#[derive(Clone, Debug)]
pub struct ResourceIter(&'static [(Resource, &'static str)]);

impl Iterator for ResourceIter {
    type Item = Resource;

    #[inline]
    fn next(&mut self) -> Option<Resource> {
        let ((res, _), rest) = self.0.split_first()?;
        self.0 = rest;
        Some(*res)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Resource> {
        match self.0.get(n..) {
            Some(rest) => {
                self.0 = rest;
                self.next()
            }
            None => {
                self.0 = &[];
                None
            }
        }
    }

    #[inline]
    fn last(self) -> Option<Resource> {
        self.0.last().map(|(r, _)| *r)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl DoubleEndedIterator for ResourceIter {
    #[inline]
    fn next_back(&mut self) -> Option<Resource> {
        let ((res, _), rest) = self.0.split_last()?;
        self.0 = rest;
        Some(*res)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Resource> {
        self.0 = &self.0[..self.0.len().saturating_sub(n)];
        self.next_back()
    }
}

impl ExactSizeIterator for ResourceIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl core::iter::FusedIterator for ResourceIter {}

pub type Limit = libc::rlim_t;

/// A special resource limit value that that means "infinity" or "no limit".
pub const RLIM_INFINITY: Limit = libc::RLIM_INFINITY;

/// Get the soft and hard limits for the given resource.
#[inline]
pub fn getrlimit(resource: Resource) -> Result<(Limit, Limit)> {
    let mut rlim = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::getrlimit(resource as _, rlim.as_mut_ptr()) })?;
    let rlim = unsafe { rlim.assume_init() };
    Ok((rlim.rlim_cur, rlim.rlim_max))
}

/// Set the soft and hard limits for the given resource.
///
/// # Safety
///
/// Altering the memory resource limits (such as [`Resource::STACK`] and [`Resource::AS`]) can
/// cause the program to misbehave, panic, and/or segfault.
#[inline]
pub unsafe fn setrlimit(resource: Resource, new_limits: (Limit, Limit)) -> Result<()> {
    let rlim = libc::rlimit {
        rlim_cur: new_limits.0,
        rlim_max: new_limits.1,
    };

    Error::unpack_nz(libc::setrlimit(resource as _, &rlim))
}

/// Get/set the soft and hard limits for the given resource on an arbitrary process.
///
/// If `pid` is 0, this operates on the current process.
///
/// # Safety
///
/// See [`setrlimit()`].
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub unsafe fn prlimit(
    pid: libc::pid_t,
    resource: Resource,
    new_limits: Option<(Limit, Limit)>,
) -> Result<(Limit, Limit)> {
    let mut old_rlim = MaybeUninit::uninit();

    let new_limits = new_limits.map(|(soft, hard)| libc::rlimit {
        rlim_cur: soft,
        rlim_max: hard,
    });

    Error::unpack_nz(libc::prlimit(
        pid,
        resource as _,
        new_limits
            .as_ref()
            .map(|rl| rl as *const _)
            .unwrap_or_else(core::ptr::null),
        old_rlim.as_mut_ptr(),
    ))?;

    let old_rlim = old_rlim.assume_init();
    Ok((old_rlim.rlim_cur, old_rlim.rlim_max))
}

/// Get/set the soft and hard limits for the given resource on an arbitrary process.
///
/// Note: Unlike `prlimit()` on Linux, if `pid` is 0, this does NOT operate on the current process.
///
/// # Safety
///
/// See [`setrlimit()`].
#[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
#[cfg(target_os = "freebsd")]
pub unsafe fn proc_rlimit(
    pid: libc::pid_t,
    resource: Resource,
    new_limits: Option<(Limit, Limit)>,
) -> Result<(Limit, Limit)> {
    let mut old_rlim = MaybeUninit::<libc::rlimit>::uninit();

    let new_limits = new_limits.map(|(soft, hard)| libc::rlimit {
        rlim_cur: soft,
        rlim_max: hard,
    });

    let old_len = crate::sysctl(
        &[
            libc::CTL_KERN,
            libc::KERN_PROC,
            libc::KERN_PROC_RLIMIT,
            pid as _,
            resource as _,
        ],
        Some(core::slice::from_raw_parts_mut(old_rlim.as_mut_ptr(), 1)),
        new_limits.as_ref().map(core::slice::from_ref),
    )?;

    debug_assert_eq!(old_len, core::mem::size_of::<libc::rlimit>());

    let old_rlim = old_rlim.assume_init();
    Ok((old_rlim.rlim_cur, old_rlim.rlim_max))
}

/// A module with utility functions for manipulating resource limits.
pub mod rlimits {
    use super::*;

    /// Compare two resource limits.
    ///
    /// This is equivalent to `val1.cmp(&val2)`, except that it properly acknowledges "infinite"
    /// resource limits (i.e. [`RLIM_INFINITY`]).
    pub fn compare_limits(val1: Limit, val2: Limit) -> Ordering {
        match (val1, val2) {
            (RLIM_INFINITY, RLIM_INFINITY) => Ordering::Equal,
            (RLIM_INFINITY, _) => Ordering::Greater,
            (_, RLIM_INFINITY) => Ordering::Less,
            (_, _) => val1.cmp(&val2),
        }
    }

    /// Convert a `NICE` resource limit value to the corresponding maximum priority value.
    ///
    /// Notes:
    /// 1. This function will only produce results in the range -20 to 19 (inclusive), since that is
    ///    the range of acceptable priority values.
    /// 2. An infinite resource limit will translate to -20.
    /// 3. Remember, lower priority values mean higher priority.
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn nice_rlimit_to_thresh(nice_rlim: Limit) -> libc::c_int {
        if nice_rlim == RLIM_INFINITY {
            -20
        } else {
            20 - (nice_rlim.max(1).min(40) as libc::c_int)
        }
    }

    /// Convert a `NICE` resource limit value to the corresponding priority value.
    ///
    /// Notes:
    /// 1. This function will only produce results in the range 1 to 20 (inclusive), since that is
    ///    the range of useful `NICE` resource limit values.
    /// 2. Remember, lower priority values mean higher priority.
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn nice_thresh_to_rlimit(nice_thresh: libc::c_int) -> Limit {
        (20 - nice_thresh.max(-20).min(19)) as Limit
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum RusageWho {
    /// Return usage statistics for all threads in the current process.
    SELF = libc::RUSAGE_SELF,
    /// Return usage statistics for all children of the current process that have terminated and
    /// been waited for.
    CHILDREN = libc::RUSAGE_CHILDREN,
    /// Return usage statistics for the current thread.
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    THREAD = libc::RUSAGE_THREAD,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Rusage(libc::rusage);

macro_rules! rusage_getters {
    ($($(#[doc = $doc:literal])* $name:ident, $field_name:ident;)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> usize {
                self.0.$field_name as usize
            }
        )*
    }
}

impl Rusage {
    #[inline]
    pub fn utime(&self) -> Duration {
        Duration::new(
            self.0.ru_utime.tv_sec as u64,
            self.0.ru_utime.tv_usec as u32 * 1000,
        )
    }

    #[inline]
    pub fn stime(&self) -> Duration {
        Duration::new(
            self.0.ru_stime.tv_sec as u64,
            self.0.ru_stime.tv_usec as u32 * 1000,
        )
    }

    rusage_getters! {
        maxrss, ru_maxrss;
        ixrss, ru_ixrss;
        idrss, ru_idrss;
        isrss, ru_isrss;
        minflt, ru_minflt;
        majflt, ru_majflt;
        nswap, ru_nswap;
        inblock, ru_inblock;
        oublock, ru_oublock;
        msgsnd, ru_msgsnd;
        msgrcv, ru_msgrcv;
        nsignals, ru_nsignals;
        nvcsw, ru_nvcsw;
        nivcsw, ru_nivcsw;
    }
}

pub fn getrusage(who: RusageWho) -> Result<Rusage> {
    let mut buf = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::getrusage(who as libc::c_int, buf.as_mut_ptr()) })?;
    Ok(Rusage(unsafe { buf.assume_init() }))
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum PrioWho {
    Process(libc::pid_t),
    Pgrp(libc::pid_t),
    User(libc::uid_t),
}

impl PrioWho {
    #[inline]
    fn unpack(&self) -> (libc::c_int, libc::id_t) {
        match *self {
            Self::Process(pid) => (libc::PRIO_PROCESS as _, pid as _),
            Self::Pgrp(pgid) => (libc::PRIO_PGRP as _, pgid as _),
            Self::User(uid) => (libc::PRIO_USER as _, uid as _),
        }
    }
}

#[inline]
pub fn getpriority(who: PrioWho) -> Result<libc::c_int> {
    unsafe {
        let eno_ptr = util::errno_ptr();
        *eno_ptr = 0;

        let (which, who) = who.unpack();
        match libc::getpriority(which as _, who as _) {
            -1 => match *eno_ptr {
                0 => Ok(-1),
                eno => Err(Error::from_code(eno)),
            },
            prio => Ok(prio),
        }
    }
}

#[inline]
pub fn setpriority(who: PrioWho, prio: libc::c_int) -> Result<()> {
    let (which, who) = who.unpack();
    Error::unpack_nz(unsafe { libc::setpriority(which as _, who as _, prio) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_limits() {
        use rlimits::*;

        assert_eq!(compare_limits(0, 0), Ordering::Equal);
        assert_eq!(compare_limits(1, 0), Ordering::Greater);
        assert_eq!(compare_limits(0, 1), Ordering::Less);

        assert_eq!(
            compare_limits(RLIM_INFINITY, RLIM_INFINITY),
            Ordering::Equal,
        );
        assert_eq!(compare_limits(RLIM_INFINITY, 0), Ordering::Greater);
        assert_eq!(compare_limits(0, RLIM_INFINITY), Ordering::Less);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_nice_rlimit_thresh() {
        use rlimits::*;

        assert_eq!(nice_rlimit_to_thresh(RLIM_INFINITY), -20);

        assert_eq!(nice_rlimit_to_thresh(40), -20);
        assert_eq!(nice_rlimit_to_thresh(30), -10);
        assert_eq!(nice_rlimit_to_thresh(20), 0);
        assert_eq!(nice_rlimit_to_thresh(10), 10);
        assert_eq!(nice_rlimit_to_thresh(1), 19);

        assert_eq!(nice_rlimit_to_thresh(100), -20);
        assert_eq!(nice_rlimit_to_thresh(0), 19);

        assert_eq!(nice_thresh_to_rlimit(-20), 40);
        assert_eq!(nice_thresh_to_rlimit(-10), 30);
        assert_eq!(nice_thresh_to_rlimit(0), 20);
        assert_eq!(nice_thresh_to_rlimit(10), 10);
        assert_eq!(nice_thresh_to_rlimit(19), 1);

        assert_eq!(nice_thresh_to_rlimit(-100), 40);
        assert_eq!(nice_thresh_to_rlimit(100), 1);
    }

    #[test]
    fn test_get_set_rlimits_same() {
        for res in Resource::iter() {
            #[cfg(apple)]
            if res == Resource::NPROC {
                // The kernel clamps RLIMIT_NPROC in strange ways
                continue;
            }

            let limits = getrlimit(res).unwrap();
            unsafe {
                setrlimit(res, limits).unwrap();
            }
            assert_eq!(getrlimit(res).unwrap(), limits);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_prlimit_same() {
        for res in Resource::iter() {
            unsafe {
                let limits = prlimit(0, res, None).unwrap();
                assert_eq!(prlimit(0, res, Some(limits)).unwrap(), limits);
                assert_eq!(prlimit(crate::getpid(), res, Some(limits)).unwrap(), limits);
                assert_eq!(prlimit(crate::getpid(), res, None).unwrap(), limits);
            }
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_resourceiter() {
        let resources: Vec<Resource> = Resource::iter().collect();
        let len = resources.len();

        assert_eq!(Resource::iter().len(), len);
        assert_eq!(Resource::iter().count(), len);
        assert_eq!(Resource::iter().size_hint(), (len, Some(len)));

        for (i, &resource) in resources.iter().enumerate() {
            assert_eq!(Resource::iter().nth(i), Some(resource));
            assert_eq!(Resource::iter().nth_back(len - i - 1), Some(resource));
        }

        assert_eq!(Resource::iter().nth(len), None);
        assert_eq!(Resource::iter().nth(len + 1), None);
        assert_eq!(Resource::iter().nth_back(len), None);
        assert_eq!(Resource::iter().nth_back(len + 1), None);

        assert_eq!(Resource::iter().last(), resources.last().cloned());

        let mut it = Resource::iter();
        // Exhaust
        it.by_ref().count();
        assert_eq!(it.len(), 0);
        assert_eq!(it.clone().count(), 0);
        assert_eq!(it.last(), None);
    }
}
