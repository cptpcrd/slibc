use crate::internal_prelude::*;

use core::fmt;

/// A target for [`ioprio_get()`] and [`ioprio_set()`] to operate on.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum IoprioWho {
    /// Operate on the process/thread with the specified PID/TID.
    ///
    /// If the PID/TID is 0, operate on the current thread.
    Process(libc::pid_t),
    /// Operate on the process group with the specified PGID.
    ///
    /// If the PGID is 0, operate on the current process group.
    Pgrp(libc::pid_t),
    /// Operate on the user with the specified UID.
    User(libc::uid_t),
}

impl IoprioWho {
    #[inline]
    fn unpack(&self) -> (libc::c_int, libc::c_int) {
        match *self {
            Self::Process(pid) => (sys::IOPRIO_WHO_PROCESS, pid as _),
            Self::Pgrp(pgid) => (sys::IOPRIO_WHO_PGRP, pgid as _),
            Self::User(uid) => (sys::IOPRIO_WHO_USER, uid as _),
        }
    }
}

/// An I/O scheduling class.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum IoprioClass {
    None = sys::IOPRIO_CLASS_NONE,
    RealTime = sys::IOPRIO_CLASS_RT,
    BestEffort = sys::IOPRIO_CLASS_BE,
    Idle = sys::IOPRIO_CLASS_IDLE,
}

/// An I/O scheduling priority value.
///
/// This wraps a bitmask that contains a scheduling class ([`Self::class()`]) and priority
/// ([`Self::data()`]).
///
/// Note that this struct assumes the mask it contains has a valid scheduling `class` value.
/// Associated methods may panic if this is not the case.
///
/// This struct is guaranteed to be ABI-compatible with a C `int`.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Ioprio(pub libc::c_int);

impl Ioprio {
    /// Assemble a new `Ioprio` from a scheduling class and a priority.
    ///
    /// # Panics
    ///
    /// Panics if `data` is too large. Currently the maximum value is `2 ^ 13 - 1`.
    #[inline]
    pub fn new(class: IoprioClass, data: u16) -> Self {
        assert_eq!(
            (data as i32) & !sys::IOPRIO_PRIO_MASK,
            0,
            "data argument too large"
        );
        Self((class as i32) << sys::IOPRIO_CLASS_SHIFT | (data as i32))
    }

    /// Create a new `Ioprio` wrapping an integer value, returning `None` if the value is invalid.
    ///
    /// This allows being sure that [`Self::class()`] will not panic on the returned `Ioprio`
    /// value.
    #[inline]
    pub fn from_raw(value: i32) -> Option<Self> {
        if matches!(
            value >> sys::IOPRIO_CLASS_SHIFT,
            sys::IOPRIO_CLASS_NONE
                | sys::IOPRIO_CLASS_RT
                | sys::IOPRIO_CLASS_BE
                | sys::IOPRIO_CLASS_IDLE
        ) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the scheduling class represented by this priority mask.
    ///
    /// # Panics
    ///
    /// Panics if this priority mask does not contain a valid class.
    #[inline]
    pub fn class(&self) -> IoprioClass {
        match self.0 >> sys::IOPRIO_CLASS_SHIFT {
            sys::IOPRIO_CLASS_NONE => IoprioClass::None,
            sys::IOPRIO_CLASS_RT => IoprioClass::RealTime,
            sys::IOPRIO_CLASS_BE => IoprioClass::BestEffort,
            sys::IOPRIO_CLASS_IDLE => IoprioClass::Idle,
            _ => panic!("invalid ioprio class"),
        }
    }

    /// Get the data (i.e. the actual priority) stored in this priority mask.
    ///
    /// This is only honored for the [`IoprioClass::RealTime`] and [`IoprioClass::BestEffort`]
    /// classes. For [`IoprioClass::None`], it must be 0.
    #[inline]
    pub fn data(&self) -> u16 {
        (self.0 & sys::IOPRIO_PRIO_MASK) as _
    }
}

impl fmt::Debug for Ioprio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ioprio")
            .field("class", &self.class())
            .field("data", &self.data())
            .field("value", &self.0)
            .finish()
    }
}

/// Get the I/O priority of the given target.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn ioprio_get(who: IoprioWho) -> Result<Ioprio> {
    let (which, who) = who.unpack();
    Error::unpack(unsafe { libc::syscall(libc::SYS_ioprio_get, which, who) } as _).map(Ioprio)
}

/// Set the I/O priority of the given target.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn ioprio_set(who: IoprioWho, ioprio: Ioprio) -> Result<()> {
    let (which, who) = who.unpack();
    Error::unpack_nz(unsafe { libc::syscall(libc::SYS_ioprio_set, which, who, ioprio.0) } as _)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ioprio_getset_same() {
        let mut ioprio = ioprio_get(IoprioWho::Process(0)).unwrap();
        assert_eq!(
            ioprio_get(IoprioWho::Process(crate::gettid())).unwrap(),
            ioprio
        );

        if ioprio.class() == IoprioClass::None && ioprio.data() != 0 {
            ioprio = Ioprio(0);
        }

        ioprio_set(IoprioWho::Process(0), ioprio).unwrap();
    }

    #[test]
    fn test_ioprio_from_raw() {
        assert_eq!(Ioprio::from_raw(0), Some(Ioprio(0)));
        assert_eq!(Ioprio::from_raw(i32::MAX), None);
    }

    #[test]
    fn test_ioprio_error() {
        assert_eq!(
            ioprio_get(IoprioWho::Process(-1)).unwrap_err(),
            Errno::ESRCH
        );
        assert_eq!(
            ioprio_set(IoprioWho::Process(-1), Ioprio(0)).unwrap_err(),
            Errno::ESRCH
        );
    }
}
