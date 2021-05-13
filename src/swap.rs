#[allow(unused_imports)]
use crate::internal_prelude::*;

cfg_if::cfg_if! {
    if #[cfg(linuxlike)] {
        const SWAP_FLAG_PREFER: libc::c_int = 0x8000;
        const SWAP_FLAG_PRIO_MASK: libc::c_int = 0x7fff;
        const SWAP_FLAG_PRIO_SHIFT: libc::c_int = 0;
        const SWAP_FLAG_DISCARD: libc::c_int = 0x10000;
        const SWAP_FLAG_DISCARD_ONCE: libc::c_int = 0x20000;
        const SWAP_FLAG_DISCARD_PAGES: libc::c_int = 0x40000;
    }
}

/// Stop swapping on the specified device.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly"
    )))
)]
#[cfg(any(linuxlike, freebsdlike))]
pub fn swapoff<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { sys::swapoff(path.as_ptr()) }))
}

/// A set of flags to be passed to [`swapon()`].
///
/// This structure wraps an integer "flags" argument and provides convenience methods for altering
/// it.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct SwapFlags(libc::c_int);

#[cfg(linuxlike)]
impl SwapFlags {
    /// Create a new (empty) set of flags.
    #[inline]
    pub fn new() -> Self {
        Self(0)
    }

    /// Set (or clear) a higher priority in the flags.
    ///
    /// `prio` values greater than 32767 (2 ^ 15 - 1) will be truncated by discarding the most
    /// significant bit, effectively subtracting 32768 from them. (TL;DR: Only pass priorities
    /// between 0 and 32767, inclusive.)
    #[inline]
    pub fn set_prio(&mut self, prio: Option<u16>) -> &mut Self {
        if let Some(prio) = prio {
            self.0 |=
                SWAP_FLAG_PREFER | (((prio as i32) << SWAP_FLAG_PRIO_SHIFT) & SWAP_FLAG_PRIO_MASK);
        } else {
            self.0 &= !(SWAP_FLAG_PREFER | SWAP_FLAG_PRIO_MASK);
        }
        self
    }

    /// Set the `SWAP_FLAG_DISCARD` flag.
    ///
    /// See `swapon(2)` for more information.
    #[inline]
    pub fn set_discard(&mut self, discard: bool) -> &mut Self {
        if discard {
            self.0 |= SWAP_FLAG_DISCARD;
        } else {
            self.0 &= !SWAP_FLAG_DISCARD;
        }
        self
    }

    /// Set the `SWAP_FLAG_DISCARD_ONCE` flag.
    ///
    /// This is not properly documented in `swapon(2)`; see the `-d/--discard` argument in
    /// `swapon(8)` for more information.
    #[inline]
    pub fn set_discard_once(&mut self, discard: bool) -> &mut Self {
        if discard {
            self.0 |= SWAP_FLAG_DISCARD_ONCE;
        } else {
            self.0 &= !SWAP_FLAG_DISCARD_ONCE;
        }
        self
    }

    /// Set the `SWAP_FLAG_DISCARD_PAGES` flag.
    ///
    /// This is not properly documented in `swapon(2)`; see the `-d/--discard` argument in
    /// `swapon(8)` for more information.
    #[inline]
    pub fn set_discard_pages(&mut self, discard: bool) -> &mut Self {
        if discard {
            self.0 |= SWAP_FLAG_DISCARD_PAGES;
        } else {
            self.0 &= !SWAP_FLAG_DISCARD_PAGES;
        }
        self
    }

    /// Return the representation of these flags as a raw integer.
    #[inline]
    pub fn as_raw(&self) -> libc::c_int {
        self.0
    }

    /// Create a `SwapFlags` structure from an integer flags argument.
    ///
    /// # Safety
    ///
    /// In the future, the kernel may add a new `swapon()` flag that can't be used safely. No such
    /// flags may be specified here unless their soundness can be verified.
    #[inline]
    pub unsafe fn from_raw(flags: libc::c_int) -> Self {
        Self(flags)
    }
}

#[cfg(linuxlike)]
impl Default for SwapFlags {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Begin swapping on the specified device with the specified `flags`.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
pub fn swapon<P: AsPath>(path: P, flags: SwapFlags) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::swapon(path.as_ptr(), flags.0) }))
}

/// Contains information on a swap device connected to the system.
///
/// See the descriptions of `SWAP_STATS` and `struct swapent` in `swapctl(2)`.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
#[derive(Copy, Clone)]
pub struct SwapEntry {
    se_dev: libc::dev_t,
    se_flags: libc::c_int,
    se_nblks: libc::c_int,
    se_inuse: libc::c_int,
    se_priority: libc::c_int,
    #[cfg(target_os = "openbsd")]
    se_path: [u8; libc::PATH_MAX as usize],
    #[cfg(target_os = "netbsd")]
    se_path: [u8; libc::PATH_MAX as usize + 1],
}

#[cfg(netbsdlike)]
impl SwapEntry {
    /// Create an empty `SwapEntry` structure (all fields 0 or empty).
    #[inline]
    pub const fn empty() -> Self {
        Self {
            se_dev: 0,
            se_flags: 0,
            se_nblks: 0,
            se_inuse: 0,
            se_priority: 0,
            #[cfg(target_os = "openbsd")]
            se_path: [0; libc::PATH_MAX as usize],
            #[cfg(target_os = "netbsd")]
            se_path: [0; libc::PATH_MAX as usize + 1],
        }
    }

    #[inline]
    pub fn dev(&self) -> libc::dev_t {
        self.se_dev
    }

    #[inline]
    pub fn flags(&self) -> SwapEntryFlags {
        SwapEntryFlags::from_bits_truncate(self.se_flags)
    }

    #[inline]
    pub fn nblks(&self) -> libc::c_int {
        self.se_nblks
    }

    #[inline]
    pub fn inuse(&self) -> libc::c_int {
        self.se_inuse
    }

    #[inline]
    pub fn priority(&self) -> libc::c_int {
        self.se_priority
    }

    #[inline]
    pub fn path(&self) -> &OsStr {
        util::osstr_from_buf(&self.se_path)
    }

    #[inline]
    pub fn path_cstr(&self) -> &CStr {
        util::cstr_from_buf(&self.se_path).unwrap()
    }
}

#[cfg(netbsdlike)]
bitflags::bitflags! {
    /// Flags returned by [`SwapEntry::flags()`].
    #[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
    pub struct SwapEntryFlags: libc::c_int {
        const INUSE = sys::SWF_INUSE;
        const ENABLE = sys::SWF_ENABLE;
        const BUSY = sys::SWF_BUSY;
        const FAKE = sys::SWF_FAKE;
    }
}

/// Begin swapping on the specified device.
#[cfg_attr(
    docsrs,
    doc(cfg(target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd"))
)]
#[cfg(any(freebsdlike, target_os = "netbsd"))]
pub fn swapon<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { sys::swapon(path.as_ptr()) }))
}

/// Get the number of swap devices in the system.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
#[inline]
pub fn swapctl_nswap() -> Result<usize> {
    Error::unpack(unsafe { sys::swapctl(sys::SWAP_NSWAP, core::ptr::null(), 0) })
        .map(|n| n as usize)
}

/// Collect statistics on system swap devices.
///
/// This copies information about each swap device on the system into an entry in `buf`. If there
/// are more entries in the system than `buf` has space for, the list is truncated. The number of
/// devices whose information was copied into `buf` is returned.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
#[inline]
pub fn swapctl_stats(buf: &mut [SwapEntry]) -> Result<usize> {
    use core::convert::TryInto;

    Error::unpack(unsafe {
        sys::swapctl(
            sys::SWAP_STATS,
            buf.as_ptr() as *const _,
            buf.len().try_into().unwrap_or(libc::c_int::MAX),
        )
    })
    .map(|n| n as usize)
}

/// Begin swapping on the specified device with the specified priority.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
pub fn swapctl_on<P: AsPath>(path: P, priority: libc::c_int) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { sys::swapctl(sys::SWAP_ON, path.as_ptr() as *const _, priority) })
    })
}

/// Stop swapping on the specified device.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
pub fn swapctl_off<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { sys::swapctl(sys::SWAP_OFF, path.as_ptr() as *const _, 0) })
    })
}

/// Like [`swapctl_on()`], but change the parameters of a currently enabled swap device.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
pub fn swapctl_ctl<P: AsPath>(path: P, priority: libc::c_int) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe {
            sys::swapctl(sys::SWAP_CTL, path.as_ptr() as *const _, priority)
        })
    })
}

/// Set the device used as a dump device in case of system panic.
#[cfg_attr(docsrs, doc(cfg(target_os = "openbsd", target_os = "netbsd")))]
#[cfg(netbsdlike)]
pub fn swapctl_dumpdev<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| {
        Error::unpack_nz(unsafe { sys::swapctl(sys::SWAP_DUMPDEV, path.as_ptr() as *const _, 0) })
    })
}

/// Get the device number of the current dump device.
#[cfg_attr(docsrs, doc(cfg(target_os = "netbsd")))]
#[cfg(target_os = "netbsd")]
#[inline]
pub fn swapctl_getdumpdev() -> Result<Option<libc::dev_t>> {
    let mut dev = 0;
    Error::unpack_nz(unsafe {
        sys::swapctl(sys::SWAP_GETDUMPDEV, &mut dev as *mut _ as *mut _, 0)
    })?;
    Ok(if dev == -1i32 as _ { None } else { Some(dev) })
}

/// Clear the system dump device.
#[cfg_attr(docsrs, doc(cfg(target_os = "netbsd")))]
#[cfg(target_os = "netbsd")]
#[inline]
pub fn swapctl_dumpoff() -> Result<()> {
    Error::unpack_nz(unsafe { sys::swapctl(sys::SWAP_DUMPOFF, core::ptr::null(), 0) })
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(linuxlike)]
    #[test]
    fn test_swapflags() {
        let mut swflags = SwapFlags::new();
        assert_eq!(swflags.as_raw(), 0);

        swflags.set_prio(Some(0));
        assert_eq!(swflags.as_raw(), SWAP_FLAG_PREFER);
        swflags.set_prio(Some(4));
        assert_eq!(
            swflags.as_raw(),
            SWAP_FLAG_PREFER | ((4 << SWAP_FLAG_PRIO_SHIFT) & SWAP_FLAG_PRIO_MASK)
        );
        swflags.set_prio(None);

        swflags.set_discard(true);
        assert_eq!(swflags.as_raw(), SWAP_FLAG_DISCARD);
        swflags.set_discard_once(true);
        assert_eq!(swflags.as_raw(), SWAP_FLAG_DISCARD | SWAP_FLAG_DISCARD_ONCE);
        swflags.set_discard_once(false);
        swflags.set_discard_pages(true);
        assert_eq!(
            swflags.as_raw(),
            SWAP_FLAG_DISCARD | SWAP_FLAG_DISCARD_PAGES
        );
        swflags.set_discard_pages(false);
        swflags.set_discard(false);

        assert_eq!(swflags.as_raw(), 0);
    }

    #[cfg(netbsdlike)]
    #[test]
    fn test_swapctl_stats() {
        let mut buf = [SwapEntry::empty(); 20];
        let nswaps = swapctl_stats(&mut buf).unwrap();

        assert_eq!(swapctl_nswap().unwrap(), nswaps);

        for swap in &buf[..nswaps] {
            // Make sure the path is NUL-terminated and not empty
            swap.path_cstr();
            assert_ne!(swap.path(), "");
        }
    }

    #[cfg(target_os = "netbsd")]
    #[test]
    fn test_swapctl_getdumpdev() {
        // Just check that it works
        swapctl_getdumpdev().unwrap();
    }
}
