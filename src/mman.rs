use crate::internal_prelude::*;

#[cfg(target_os = "linux")]
bitflags::bitflags! {
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[derive(Default)]
    pub struct Mlock2Flags: i32 {
        const ONFAULT = sys::MLOCK_ONFAULT;
    }
}

bitflags::bitflags! {
    pub struct MlockAllFlags: i32 {
        const CURRENT = libc::MCL_CURRENT;
        const FUTURE = libc::MCL_FUTURE;
        #[cfg(target_os = "linux")]
        const ONFAULT = sys::MCL_ONFAULT;
    }
}

bitflags::bitflags! {
    pub struct MsyncFlags: i32 {
        const ASYNC = libc::MS_ASYNC;
        const SYNC = libc::MS_SYNC;
        const INVALIDATE = libc::MS_INVALIDATE;
    }
}

#[inline]
pub fn mlock(data: &[u8]) -> Result<()> {
    unsafe { mlock_raw(data.as_ptr(), data.len()) }
}

#[inline]
pub unsafe fn mlock_raw(addr: *const u8, len: usize) -> Result<()> {
    Error::unpack_nz(libc::mlock(addr as *const _, len))
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn mlock2(data: &[u8], flags: Mlock2Flags) -> Result<()> {
    unsafe { mlock2_raw(data.as_ptr(), data.len(), flags) }
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub unsafe fn mlock2_raw(addr: *const u8, len: usize, flags: Mlock2Flags) -> Result<()> {
    Error::unpack_nz(sys::mlock2(addr as *const _, len, flags.bits()))
}

#[inline]
pub fn munlock(data: &[u8]) -> Result<()> {
    unsafe { munlock_raw(data.as_ptr(), data.len()) }
}

#[inline]
pub unsafe fn munlock_raw(addr: *const u8, len: usize) -> Result<()> {
    Error::unpack_nz(libc::munlock(addr as *const _, len))
}

#[inline]
pub fn mlockall(flags: MlockAllFlags) -> Result<()> {
    Error::unpack_nz(unsafe { libc::mlockall(flags.bits()) })
}

#[inline]
pub fn munlockall() -> Result<()> {
    Error::unpack_nz(unsafe { libc::munlockall() })
}

pub fn msync(data: &mut [u8], flags: MsyncFlags) -> Result<()> {
    Error::unpack_nz(unsafe { libc::msync(data.as_mut_ptr() as *mut _, data.len(), flags.bits()) })
}

#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum PosixMAdvice {
    NORMAL = libc::POSIX_MADV_NORMAL,
    SEQUENTIAL = libc::POSIX_MADV_SEQUENTIAL,
    RANDOM = libc::POSIX_MADV_RANDOM,
    WILLNEED = libc::POSIX_MADV_WILLNEED,
    DONTNEED = libc::POSIX_MADV_DONTNEED,
}

#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub fn posix_madvise(data: &mut [u8], advice: PosixMAdvice) -> Result<()> {
    unsafe { posix_madvise_raw(data.as_mut_ptr(), data.len(), advice) }
}

#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub unsafe fn posix_madvise_raw(addr: *mut u8, length: usize, advice: PosixMAdvice) -> Result<()> {
    Error::unpack_nz(libc::posix_madvise(addr as *mut _, length, advice as _))
}

macro_rules! define_madvice {
    ($(
        #[cfg($cfg:meta)]
        $(
            $(#[doc = $doc:literal])*
            $name:ident = $value:path,
        )+
    )*) => {
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[repr(i32)]
        pub enum MemAdvice {
            $($(
                #[cfg($cfg)]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                $(#[doc = $doc])*
                $name = $value as i32,
            )*)*
        }
    };
}

define_madvice! {
    #[cfg(all())]
    NORMAL = libc::MADV_NORMAL,
    RANDOM = libc::MADV_RANDOM,
    SEQUENTIAL = libc::MADV_SEQUENTIAL,
    WILLNEED = libc::MADV_WILLNEED,
    DONTNEED = libc::MADV_DONTNEED,
    FREE = libc::MADV_FREE,

    #[cfg(any(target_os = "linux", target_os = "android"))]
    REMOVE = libc::MADV_REMOVE,
    DONTFORK = libc::MADV_DONTFORK,
    DOFORK = libc::MADV_DOFORK,
    MERGEABLE = libc::MADV_MERGEABLE,
    UNMERGEABLE = libc::MADV_UNMERGEABLE,
    HUGEPAGE = libc::MADV_HUGEPAGE,
    NOHUGEPAGE = libc::MADV_NOHUGEPAGE,
    DONTDUMP = libc::MADV_DONTDUMP,
    DODUMP = libc::MADV_DODUMP,
    HWPOISON = libc::MADV_HWPOISON,
    SOFT_OFFLINE = libc::MADV_SOFT_OFFLINE,

    WIPEONFORK = sys::MADV_WIPEONFORK,
    KEEPONFORK = sys::MADV_KEEPONFORK,
    COLD = sys::MADV_COLD,
    PAGEOUT = sys::MADV_PAGEOUT,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    NOSYNC = libc::MADV_NOSYNC,
    AUTOSYNC = libc::MADV_AUTOSYNC,
    NOCORE = libc::MADV_NOCORE,
    CORE = libc::MADV_CORE,
    PROTECT = libc::MADV_PROTECT,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    ZERO_WIRED_PAGES = libc::MADV_ZERO_WIRED_PAGES,
}

#[inline]
pub unsafe fn madvise(data: &mut [u8], advice: MemAdvice) -> Result<()> {
    madvise_raw(data.as_mut_ptr(), data.len(), advice)
}

#[inline]
pub unsafe fn madvise_raw(addr: *mut u8, length: usize, advice: MemAdvice) -> Result<()> {
    Error::unpack_nz(libc::madvise(addr as *mut _, length, advice as _))
}
