use crate::internal_prelude::*;

use core::fmt;
use core::str::FromStr;

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
use core::ops::RangeInclusive;

macro_rules! define_signal {
    ($(#[cfg($cfg:meta)] $($name:ident,)+)+ @alias, $(#[cfg($cfg2:meta)] $($name2:ident,)+)+) => {
        /// A struct representing a signal that can be sent to a process with [`kill()`].
        ///
        /// This is not an enum because that would make using real-time signals impossible.
        ///
        /// Signals specified by POSIX (`SIGINT`, `SIGTERM`, etc.) are present as constants; see
        /// the documentation below. They can also be listed with [`Self::posix_signals()`].
        /// Real-time signals can be listed with [`Self::rt_signals()`] on platforms where they are
        /// available.
        #[derive(Copy, Clone, Eq, Hash, PartialEq)]
        pub struct Signal(i32);

        impl Signal {
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                pub const $name: Self = Self(libc::$name);
            )*)*
            $($(
                #[cfg($cfg2)]
                #[cfg_attr(docsrs, doc(cfg($cfg2)))]
                pub const $name2: Self = Self(libc::$name2);
            )*)*

            /// Includes duplicates
            #[allow(dead_code)]
            const ALL_POSIX_SIGNALS: &'static [Self] = &[
                $($(
                    #[cfg($cfg)]
                    Self::$name,
                )*)*
                $($(
                    #[cfg($cfg2)]
                    Self::$name2,
                )*)*
            ];

            /// Does NOT include duplicates
            const POSIX_SIGNALS: &'static [Self] = &[
                $($(
                    #[cfg($cfg)]
                    Self::$name,
                )*)*
            ];

            /// Get the raw integer value of the signal.
            #[inline]
            pub fn as_i32(self) -> i32 {
                self.0
            }

            /// Construct a `Signal` from the given raw integer value (if it is valid).
            pub fn from_i32(sig: i32) -> Option<Self> {
                let sig = Self(sig);

                #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
                if Self::rt_signals().contains(sig) {
                    return Some(sig);
                }

                if Self::POSIX_SIGNALS.contains(&sig) {
                    Some(sig)
                } else {
                    None
                }
            }

            /// Returns whether this signal can be caught by the process.
            ///
            /// `SIGKILL` and `SIGSTOP` cannot be caught.
            #[inline]
            pub fn can_catch(self) -> bool {
                !matches!(self.0, libc::SIGKILL | libc::SIGSTOP)
            }

            /// Create an iterator over all of the POSIX signals.
            ///
            /// This will NOT include duplicates (i.e. aliases), but it is not guaranteed to be in
            /// any particular order.
            #[inline]
            pub fn posix_signals() -> SignalPosixIter {
                SignalPosixIter(Self::POSIX_SIGNALS.iter())
            }
        }

        #[cfg_attr(
            docsrs,
            doc(cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "freebsd",
                target_os = "netbsd",
            )))
        )]
        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
        impl Signal {
            /// Create an iterator over all of the real-time signals supported on the current
            /// system.
            ///
            /// This is the only way to get `Signal`s for the real time signal. If you want e.g.
            /// `SIGRTMIN+1`, use `Signal::rt_signals::nth(1).unwrap()`.
            #[inline]
            pub fn rt_signals() -> SignalRtIter {
                #[cfg(linuxlike)]
                let (sigrtmin, sigrtmax) = unsafe { (sys::__libc_current_sigrtmin(), sys::__libc_current_sigrtmax()) };
                #[cfg(not(linuxlike))]
                let (sigrtmin, sigrtmax) = (sys::SIGRTMIN, sys::SIGRTMAX);

                SignalRtIter(Self(sigrtmin)..=Self(sigrtmax))
            }
        }

        impl fmt::Debug for Signal {
            #[deny(unreachable_patterns)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let s = match self.0 {
                    $($(
                        #[cfg($cfg)]
                        libc::$name => stringify!($name),
                    )*)*
                    _ => {
                        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
                        if let Some(i) = Self::rt_signals().position_of(*self) {
                            return write!(f, "SIGRTMIN+{}", i);
                        }

                        unreachable!();
                    }
                };

                f.write_str(s)
            }
        }

        impl FromStr for Signal {
            type Err = SignalParseError;

            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                if !s.starts_with("SIG") {
                    return Err(SignalParseError(()));
                }

                #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
                if let Some(s) = s.strip_prefix("SIGRTMIN+") {
                    if s.bytes().all(|b| (b'0'..=b'9').contains(&b)) {
                        if let Ok(i) = s.parse() {
                            if let Some(sig) = Self::rt_signals().nth(i) {
                                return Ok(sig);
                            }
                        }
                    }

                    return Err(SignalParseError(()));
                } else if let Some(s) = s.strip_prefix("SIGRTMAX-") {
                    if s.bytes().all(|b| (b'0'..=b'9').contains(&b)) {
                        if let Ok(i) = s.parse() {
                            if let Some(sig) = Self::rt_signals().nth_back(i) {
                                return Ok(sig);
                            }
                        }
                    }

                    return Err(SignalParseError(()));
                }

                match s {
                    $($(
                        #[cfg($cfg)]
                        stringify!($name) => Ok(Self::$name),
                    )*)*
                    $($(
                        #[cfg($cfg2)]
                        stringify!($name2) => Ok(Self::$name2),
                    )*)*
                    _ => Err(SignalParseError(())),
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SignalParseError(());

impl fmt::Display for SignalParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Unknown signal")
    }
}

impl fmt::Debug for SignalParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SignalParseError")
            .field("message", &"Unknown signal")
            .finish()
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SignalParseError {}

define_signal! {
    #[cfg(all())]
    SIGINT,
    SIGHUP,
    SIGTERM,
    SIGQUIT,
    SIGKILL,
    SIGILL,
    SIGABRT,
    SIGALRM,
    SIGBUS,
    SIGWINCH,
    SIGPIPE,
    SIGSEGV,
    SIGFPE,
    SIGSTOP,
    SIGCONT,
    SIGCHLD,
    SIGTTIN,
    SIGTTOU,
    SIGTSTP,
    SIGUSR1,
    SIGUSR2,
    SIGPROF,
    SIGSYS,
    SIGTRAP,
    SIGURG,
    SIGVTALRM,
    SIGXCPU,
    SIGXFSZ,
    SIGIO,

    #[cfg(any(target_os = "linux", target_os = "android"))]
    SIGSTKFLT,
    SIGPWR,

    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    SIGEMT,
    SIGINFO,

    // Signals that are aliases of other signals
    // These will have constants added to the structure, and they can be parsed with from_str(),
    // but they won't be listed with Signal::posix_signals() and they will never be written with
    // Debug (since they're duplicates)
    @alias,
    #[cfg(all())]
    SIGIOT,

    #[cfg(any(target_os = "linux", target_os = "android"))]
    SIGPOLL,
}

#[derive(Clone, Debug)]
pub struct SignalPosixIter(core::slice::Iter<'static, Signal>);

impl SignalPosixIter {
    #[inline]
    pub fn contains(&self, sig: Signal) -> bool {
        self.0.as_slice().contains(&sig)
    }
}

impl Iterator for SignalPosixIter {
    type Item = Signal;

    #[inline]
    fn next(&mut self) -> Option<Signal> {
        self.0.next().copied()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Signal> {
        self.0.nth(n).copied()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.len()
    }

    #[inline]
    fn last(self) -> Option<Signal> {
        self.0.last().copied()
    }
}

impl ExactSizeIterator for SignalPosixIter {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl core::iter::FusedIterator for SignalPosixIter {}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
#[derive(Clone, Debug)]
pub struct SignalRtIter(RangeInclusive<Signal>);

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
impl SignalRtIter {
    #[inline]
    fn mark_exhausted(&mut self) {
        self.0 = Signal(0)..=Signal(0);
    }

    #[inline]
    pub fn contains(&self, sig: Signal) -> bool {
        sig.0 >= self.0.start().0 && sig.0 <= self.0.end().0
    }

    /// Get the position of the specified signal within this iterator.
    ///
    /// This is equivalent to `self.position(|s| s == sig)`, but it is more efficient.
    #[inline]
    pub fn position_of(&self, sig: Signal) -> Option<usize> {
        if self.contains(sig) {
            Some((sig.0 - self.0.start().0) as usize)
        } else {
            None
        }
    }
}

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
impl Iterator for SignalRtIter {
    type Item = Signal;

    #[allow(clippy::iter_nth_zero)]
    #[inline]
    fn next(&mut self) -> Option<Signal> {
        self.nth(0)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Signal> {
        if n >= self.len() {
            self.mark_exhausted();
            None
        } else {
            let sig = Signal(self.0.start().0 + n as i32);
            self.0 = Signal(sig.0 + 1)..=*self.0.end();
            Some(sig)
        }
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

    #[inline]
    fn last(mut self) -> Option<Signal> {
        self.next_back()
    }
}

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
impl DoubleEndedIterator for SignalRtIter {
    #[allow(clippy::iter_nth_zero)]
    #[inline]
    fn next_back(&mut self) -> Option<Signal> {
        self.nth_back(0)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Signal> {
        if n >= self.len() {
            self.mark_exhausted();
            None
        } else {
            let sig = Signal(self.0.end().0 - n as i32);
            self.0 = *self.0.start()..=Signal(sig.0 - 1);
            Some(sig)
        }
    }
}

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
impl ExactSizeIterator for SignalRtIter {
    #[inline]
    fn len(&self) -> usize {
        (self.0.end().0 + 1 - self.0.start().0) as usize
    }
}

#[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
impl core::iter::FusedIterator for SignalRtIter {}

#[inline]
pub fn kill<S: Into<Option<Signal>>>(pid: libc::pid_t, sig: S) -> Result<()> {
    Error::unpack_nz(unsafe { libc::kill(pid, sig.into().map_or(0, |s| s.0)) })
}

#[inline]
pub fn killpg<S: Into<Option<Signal>>>(pgid: libc::pid_t, sig: S) -> Result<()> {
    Error::unpack_nz(unsafe { libc::killpg(pgid, sig.into().map_or(0, |s| s.0)) })
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn tgkill<S: Into<Option<Signal>>>(tgid: libc::pid_t, tid: libc::pid_t, sig: S) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::syscall(libc::SYS_tgkill, tgid, tid, sig.into().map_or(0, |s| s.0)) as _
    })
}

#[derive(Copy, Clone)]
pub struct SigSet(libc::sigset_t);

impl SigSet {
    /// Create an empty signal set.
    #[inline]
    pub fn empty() -> Self {
        unsafe {
            let mut set = MaybeUninit::uninit();
            libc::sigemptyset(set.as_mut_ptr());
            Self(set.assume_init())
        }
    }

    /// Create a full signal set.
    #[inline]
    pub fn full() -> Self {
        unsafe {
            let mut set = MaybeUninit::uninit();
            libc::sigfillset(set.as_mut_ptr());
            Self(set.assume_init())
        }
    }

    /// Empty this signal set.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            libc::sigemptyset(&mut self.0);
        }
    }

    /// Fill this signal set.
    #[inline]
    pub fn fill(&mut self) {
        unsafe {
            libc::sigfillset(&mut self.0);
        }
    }

    #[inline]
    pub fn contains(&self, sig: Signal) -> bool {
        let res = unsafe { libc::sigismember(&self.0, sig.0) };
        debug_assert!(res >= 0);
        res != 0
    }

    #[inline]
    pub fn add(&mut self, sig: Signal) {
        let res = unsafe { libc::sigaddset(&mut self.0, sig.0) };
        debug_assert_eq!(res, 0);
    }

    #[inline]
    pub fn remove(&mut self, sig: Signal) {
        let res = unsafe { libc::sigdelset(&mut self.0, sig.0) };
        debug_assert_eq!(res, 0);
    }

    /// Create a new signal set that is the union of the two provided signal sets (i.e. all signals
    /// present in either set).
    ///
    /// `let s3 = s1.union(s2)` is equivalent to `let mut s3 = s1.clone(); s3.extend(s2)`, but
    /// `union()` is faster on Linux and FreeBSD (which have the `sigorset()` extension function).
    #[allow(clippy::needless_return)]
    #[inline]
    pub fn union(&self, other: &SigSet) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "linux", target_os = "freebsd"))] {
                return unsafe {
                    let mut newset = MaybeUninit::zeroed();
                    sys::sigorset(newset.as_mut_ptr(), &self.0, &other.0);
                    Self(newset.assume_init())
                };
            } else {
                let mut newset = self.clone();
                newset.extend(other.iter());
                return newset;
            }
        }
    }

    /// Create a new signal set that is the intersection of the two provided signal sets (i.e. all
    /// signals present in both sets).
    ///
    /// On Linux and FreeBSD, the `sigandset()` extension function is used here to improve
    /// performance.
    #[allow(clippy::needless_return)]
    #[inline]
    pub fn intersection(&self, other: &SigSet) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "linux", target_os = "freebsd"))] {
                return unsafe {
                    let mut newset = MaybeUninit::zeroed();
                    sys::sigandset(newset.as_mut_ptr(), &self.0, &other.0);
                    Self(newset.assume_init())
                };
            } else {
                let mut newset = self.clone();
                for sig in self.iter() {
                    if !other.contains(sig) {
                        newset.remove(sig);
                    }
                }
                return newset;
            }
        }
    }

    /// Create an iterator over this signal set.
    #[inline]
    pub fn iter(&self) -> SigSetIter {
        self.into_iter()
    }

    /// Get the current thread's signal mask.
    #[inline]
    pub fn thread_get_mask() -> Result<Self> {
        pthread_sigmask(SigmaskHow::BLOCK, None)
    }

    /// Set the current thread's signal mask to this set, and return the previous mask.
    #[inline]
    pub fn thread_set_mask(&self) -> Result<SigSet> {
        pthread_sigmask(SigmaskHow::SETMASK, Some(self))
    }

    /// Block the signals from this thread in the current thread's signal mask, and return the
    /// previous mask.
    #[inline]
    pub fn thread_block(&self) -> Result<SigSet> {
        pthread_sigmask(SigmaskHow::BLOCK, Some(self))
    }

    /// Unblock the signals from this thread in the current thread's signal mask, and return the
    /// previous mask.
    #[inline]
    pub fn thread_unblock(&self) -> Result<SigSet> {
        pthread_sigmask(SigmaskHow::UNBLOCK, Some(self))
    }

    /// Wait for one of the signals in this set to become pending, and return the signal number.
    #[inline]
    pub fn wait(&self) -> Result<Signal> {
        sigwait(self)
    }
}

impl PartialEq for SigSet {
    fn eq(&self, other: &Self) -> bool {
        for sig in Signal::posix_signals() {
            if self.contains(sig) != other.contains(sig) {
                return false;
            }
        }

        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
        for sig in Signal::rt_signals() {
            if self.contains(sig) != other.contains(sig) {
                return false;
            }
        }

        true
    }
}

impl Eq for SigSet {}

impl core::hash::Hash for SigSet {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for sig in self.iter() {
            state.write_i32(sig.0);
        }
    }
}

impl Default for SigSet {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl AsRef<libc::sigset_t> for SigSet {
    #[inline]
    fn as_ref(&self) -> &libc::sigset_t {
        &self.0
    }
}

impl fmt::Debug for SigSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl core::iter::FromIterator<Signal> for SigSet {
    #[inline]
    fn from_iter<I: IntoIterator<Item = Signal>>(it: I) -> Self {
        let mut set = Self::empty();
        set.extend(it);
        set
    }
}

impl core::iter::Extend<Signal> for SigSet {
    #[inline]
    fn extend<I: IntoIterator<Item = Signal>>(&mut self, it: I) {
        for sig in it.into_iter() {
            self.add(sig);
        }
    }
}

impl IntoIterator for SigSet {
    type Item = Signal;
    type IntoIter = SigSetIter;

    #[inline]
    fn into_iter(self) -> SigSetIter {
        SigSetIter {
            set: self,
            #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
            it: Signal::posix_signals().chain(Signal::rt_signals()),
            #[cfg(not(any(linuxlike, target_os = "freebsd", target_os = "netbsd")))]
            it: Signal::posix_signals(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SigSetIter {
    set: SigSet,
    #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
    it: core::iter::Chain<SignalPosixIter, SignalRtIter>,
    #[cfg(not(any(linuxlike, target_os = "freebsd", target_os = "netbsd")))]
    it: SignalPosixIter,
}

impl Iterator for SigSetIter {
    type Item = Signal;

    fn next(&mut self) -> Option<Signal> {
        while let Some(sig) = self.it.next() {
            if self.set.contains(sig) {
                return Some(sig);
            }
        }

        None
    }
}

/// A helper macro to construct `SigSet`s.
///
/// Example:
/// ```
/// # use slibc::{sigset, Signal, SigSet};
/// assert_eq!(sigset!(), SigSet::empty());
/// assert!(sigset!(Signal::SIGINT).iter().eq([Signal::SIGINT].iter().copied()));
/// ```
///
/// Note that this is NOT more efficient than using [`SigSet::empty`] and [`SigSet::add`]; it's
/// just more convenient.
#[macro_export]
macro_rules! sigset {
    () => {
        $crate::SigSet::empty()
    };

    ($($sigs:expr),+ $(,)?) => {{
        let mut set = $crate::SigSet::empty();
        $(set.add($sigs);)+
        set
    }};
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SigInfo(pub(crate) libc::siginfo_t);

macro_rules! si_attrs {
    ($($(#[doc = $doc:literal])* $name:ident -> $type:ty,)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> $type {
                self.0.$name as _
            }
        )*
    }
}

#[cfg(not(netbsdlike))]
macro_rules! si_attrs_extra {
    ($($(#[doc = $doc:literal])* $name:ident -> $type:ty,)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> $type {
                unsafe { self.0.$name() as _ }
            }
        )*
    }
}

impl SigInfo {
    si_attrs! {
        si_signo -> i32,
        si_errno -> i32,
        si_code -> i32,
    }

    #[cfg(not(netbsdlike))]
    si_attrs_extra! {
        si_pid -> libc::pid_t,
        si_uid -> u32,
        si_status -> u32,
    }

    #[inline]
    pub fn signal(&self) -> Option<Signal> {
        Signal::from_i32(self.si_signo())
    }
}

#[inline]
pub fn pause() {
    unsafe {
        libc::pause();
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly"
    )))
)]
#[cfg(any(target_os = "linux", target_os = "netbsd", freebsdlike))]
#[inline]
pub fn sigwaitinfo(set: &SigSet) -> Result<(Signal, SigInfo)> {
    unsafe {
        let mut info = core::mem::zeroed();

        let signo = Error::unpack(libc::sigwaitinfo(&set.0, &mut info))?;

        Ok((Signal::from_i32(signo).unwrap(), SigInfo(info)))
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(target_os = "macos", target_os = "ios", target_os = "openbsd")))
)]
#[cfg(not(any(apple, target_os = "openbsd")))]
#[inline]
pub fn sigtimedwait(set: &SigSet, timeout: Option<crate::TimeSpec>) -> Result<(Signal, SigInfo)> {
    unsafe {
        let mut info = core::mem::zeroed();

        let signo = Error::unpack(libc::sigtimedwait(
            &set.0,
            &mut info,
            timeout.map_or_else(core::ptr::null, |t| t.as_ref()),
        ))?;

        Ok((Signal::from_i32(signo).unwrap(), SigInfo(info)))
    }
}

#[inline]
pub fn sigwait(set: &SigSet) -> Result<Signal> {
    let mut sig = MaybeUninit::uninit();
    Error::unpack_eno(unsafe { libc::sigwait(&set.0, sig.as_mut_ptr()) })?;
    Ok(Signal::from_i32(unsafe { sig.assume_init() }).unwrap())
}

#[inline]
pub fn sigpending() -> Result<SigSet> {
    let mut set = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::sigpending(set.as_mut_ptr()) })?;
    Ok(SigSet(unsafe { set.assume_init() }))
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum SigmaskHow {
    BLOCK = libc::SIG_BLOCK,
    UNBLOCK = libc::SIG_UNBLOCK,
    SETMASK = libc::SIG_SETMASK,
}

#[inline]
pub fn pthread_sigmask(how: SigmaskHow, set: Option<&SigSet>) -> Result<SigSet> {
    let mut oldset = unsafe { core::mem::zeroed() };
    Error::unpack_nz(unsafe {
        libc::pthread_sigmask(
            how as _,
            set.map_or_else(core::ptr::null, |s| s.as_ref()),
            &mut oldset,
        )
    })?;
    Ok(SigSet(oldset))
}

#[inline]
pub fn raise(sig: Signal) -> Result<()> {
    Error::unpack_nz(unsafe { libc::raise(sig.as_i32()) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_signals() -> impl Iterator<Item = Signal> {
        let sigs = Signal::posix_signals();
        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
        let sigs = sigs.chain(Signal::rt_signals());

        sigs
    }

    #[test]
    fn test_signal_i32() {
        for sig in all_signals() {
            assert_eq!(Signal::from_i32(sig.as_i32()), Some(sig));
        }

        assert_eq!(Signal::from_i32(0), None);
        assert_eq!(Signal::from_i32(-1), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_signal_string() {
        for sig in all_signals() {
            let name = format!("{:?}", sig);
            assert_eq!(Signal::from_str(&name).unwrap(), sig);
        }

        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
        {
            let n_rtsigs = Signal::rt_signals().len();

            for (i, sig) in Signal::rt_signals().enumerate() {
                assert_eq!(Signal::from_str(&format!("SIGRTMIN+{}", i)).unwrap(), sig);
                assert_eq!(
                    Signal::from_str(&format!("SIGRTMAX-{}", n_rtsigs - i - 1)).unwrap(),
                    sig
                );
            }
        }
    }

    #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
    #[test]
    fn test_signal_rt_iter() {
        let n_rtsigs = Signal::rt_signals().len();
        for (i, sig) in Signal::rt_signals().enumerate() {
            assert_eq!(Signal::rt_signals().nth(i), Some(sig));
            assert_eq!(Signal::rt_signals().nth_back(n_rtsigs - i - 1), Some(sig));
        }

        assert_eq!(Signal::rt_signals().nth(n_rtsigs), None);
        assert_eq!(Signal::rt_signals().nth_back(n_rtsigs), None);

        for sig in all_signals() {
            assert_eq!(
                Signal::rt_signals().position_of(sig),
                Signal::rt_signals().position(|s| s == sig)
            );
            assert_eq!(
                Signal::rt_signals().position_of(sig),
                Signal::rt_signals().rposition(|s| s == sig)
            );
        }
    }

    #[test]
    fn test_signal_parse_error() {
        Signal::from_str("SIG").unwrap_err();
        Signal::from_str("SIGNOEXIST").unwrap_err();
        Signal::from_str("NOEXIST").unwrap_err();

        Signal::from_str("SIGRTMAX-a").unwrap_err();
        Signal::from_str("SIGRTMAX-+0").unwrap_err();
        Signal::from_str("SIGRTMAX+0").unwrap_err();
        Signal::from_str("SIGRTMAX-1000").unwrap_err();

        Signal::from_str("SIGRTMIN+a").unwrap_err();
        Signal::from_str("SIGRTMIN++0").unwrap_err();
        Signal::from_str("SIGRTMIN-0").unwrap_err();
        Signal::from_str("SIGRTMIN+1000").unwrap_err();
    }

    #[test]
    fn test_sigset_ops() {
        #[cfg(feature = "std")]
        fn hash_set(set: SigSet) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            set.hash(&mut hasher);
            hasher.finish()
        }

        fn check_empty(set: SigSet) {
            assert_eq!(set, SigSet::empty());

            #[cfg(feature = "std")]
            assert_eq!(hash_set(set), hash_set(SigSet::empty()));

            for sig in all_signals() {
                assert!(!set.contains(sig));
            }
        }

        fn check_full(set: SigSet) {
            assert_eq!(set, SigSet::full());

            #[cfg(feature = "std")]
            assert_eq!(hash_set(set), hash_set(SigSet::full()));

            for sig in all_signals() {
                assert!(set.contains(sig));
            }
        }

        let mut s;

        s = SigSet::empty();
        check_empty(s);
        s.fill();
        check_full(s);
        s.clear();
        check_empty(s);

        s = SigSet::full();
        check_full(s);
        s.clear();
        check_empty(s);
        s.fill();
        check_full(s);

        s = SigSet::empty();
        check_empty(s);
        s = s.union(&SigSet::full());
        check_full(s);
        s.clear();
        check_empty(s);

        check_full(all_signals().collect());

        check_empty(SigSet::empty().union(&SigSet::empty()));
        check_full(SigSet::empty().union(&SigSet::full()));
        check_full(SigSet::full().union(&SigSet::empty()));
        check_full(SigSet::full().union(&SigSet::full()));

        check_empty(SigSet::empty().intersection(&SigSet::empty()));
        check_empty(SigSet::empty().intersection(&SigSet::full()));
        check_empty(SigSet::full().intersection(&SigSet::empty()));
        check_full(SigSet::full().intersection(&SigSet::full()));

        check_empty(SigSet::default());
        check_empty([Signal::SIGINT; 0].iter().cloned().collect::<SigSet>());
        check_empty(sigset!());

        s = SigSet::empty();
        s.add(Signal::SIGINT);
        assert_ne!(s, SigSet::empty());
        assert_ne!(s, SigSet::full());
        assert_eq!(s, [Signal::SIGINT].iter().cloned().collect::<SigSet>());
        assert_eq!(s, sigset!(Signal::SIGINT));

        assert!(s.contains(Signal::SIGINT));
        s.remove(Signal::SIGINT);
        assert!(!s.contains(Signal::SIGINT));
        assert_eq!(s, SigSet::empty());

        s = SigSet::empty();
        s = s.union(&SigSet::empty());
        assert_eq!(s, SigSet::empty());

        assert_eq!(SigSet::empty().iter().next(), None);
        assert!(SigSet::full().iter().eq(all_signals()));

        assert_eq!(
            sigset!(Signal::SIGINT, Signal::SIGTERM).union(&SigSet::full()),
            SigSet::full()
        );
        assert_eq!(
            sigset!(Signal::SIGINT, Signal::SIGTERM).union(&SigSet::empty()),
            sigset!(Signal::SIGINT, Signal::SIGTERM)
        );
        assert_eq!(
            sigset!(Signal::SIGINT, Signal::SIGTERM).intersection(&SigSet::full()),
            sigset!(Signal::SIGINT, Signal::SIGTERM)
        );
        assert_eq!(
            sigset!(Signal::SIGINT, Signal::SIGTERM).intersection(&SigSet::empty()),
            SigSet::empty()
        );

        assert_eq!(
            SigSet::full().union(&sigset!(Signal::SIGINT, Signal::SIGTERM)),
            SigSet::full()
        );
        assert_eq!(
            SigSet::empty().union(&sigset!(Signal::SIGINT, Signal::SIGTERM)),
            sigset!(Signal::SIGINT, Signal::SIGTERM)
        );
        assert_eq!(
            SigSet::full().intersection(&sigset!(Signal::SIGINT, Signal::SIGTERM)),
            sigset!(Signal::SIGINT, Signal::SIGTERM)
        );
        assert_eq!(
            SigSet::empty().intersection(&sigset!(Signal::SIGINT, Signal::SIGTERM)),
            SigSet::empty()
        );

        #[cfg(any(linuxlike, target_os = "freebsd", target_os = "netbsd"))]
        {
            assert_ne!(sigset!(), sigset!(Signal::rt_signals().next().unwrap()));

            assert_ne!(
                sigset!(Signal::SIGINT, Signal::SIGTERM),
                sigset!(
                    Signal::SIGINT,
                    Signal::SIGTERM,
                    Signal::rt_signals().next().unwrap()
                )
            );
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_sigset_debug() {
        assert_eq!(format!("{:?}", sigset!()), "{}");
        assert_eq!(format!("{:?}", sigset!(Signal::SIGINT)), "{SIGINT}");
    }

    #[test]
    fn test_signal_aliases() {
        for &sig in Signal::ALL_POSIX_SIGNALS.iter() {
            assert!(Signal::posix_signals().contains(sig));
        }

        assert_eq!(Signal::from_str("SIGIOT").unwrap(), Signal::SIGIOT);

        #[cfg(any(target_os = "linux", target_os = "android"))]
        assert_eq!(Signal::from_str("SIGPOLL").unwrap(), Signal::SIGPOLL);

        #[cfg(feature = "alloc")]
        {
            let mut sigs = all_signals().collect::<Vec<_>>();

            // Check that there are no duplicates
            sigs.sort_unstable_by_key(|s| s.as_i32());
            let sigs2 = sigs.clone();
            sigs.dedup();
            assert_eq!(sigs, sigs2);
        }
    }
}
