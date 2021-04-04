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

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
pub fn swapon<P: AsPath>(path: P, flags: SwapFlags) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { libc::swapon(path.as_ptr(), flags.0) }))
}

#[cfg_attr(docsrs, doc(cfg(target_os = "freebsd", target_os = "dragonfly")))]
#[cfg(freebsdlike)]
pub fn swapon<P: AsPath>(path: P) -> Result<()> {
    path.with_cstr(|path| Error::unpack_nz(unsafe { sys::swapon(path.as_ptr()) }))
}

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
