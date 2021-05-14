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
    /// Flags for [`mlockall()`].
    pub struct MlockAllFlags: i32 {
        /// Lock all pages currently mapped into the process' address space.
        const CURRENT = libc::MCL_CURRENT;
        /// Lock all pages that are mapped into the process's address space in the future.
        const FUTURE = libc::MCL_FUTURE;
        /// Lock all currently mapped pages (when specified with [`Self::CURRENT`]) or pages mapped
        /// in the future (when specified with [`Self::FUTURE`]) into memory when they are faulted
        /// in.
        ///
        /// See `mlockall(2)` for the exact semantics.
        #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
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

/// Lock the pages containing any part of the specified region of memory into RAM.
///
/// See `mlock(2)` for more information.
///
/// For a version of this function that accepts a raw pointer and length, see [`mlock_raw()`].
#[inline]
pub fn mlock(data: &[u8]) -> Result<()> {
    unsafe { mlock_raw(data.as_ptr(), data.len()) }
}

/// Lock the pages containing any part of the specified region of memory into RAM.
///
/// See `mlock(2)` for more information.
///
/// # Safety
///
/// `addr` and `len` must refer to a valid region of memory.
#[inline]
pub unsafe fn mlock_raw(addr: *const u8, len: usize) -> Result<()> {
    Error::unpack_nz(libc::mlock(addr as *const _, len))
}

/// Lock the pages containing any part of the specified region of memory into RAM.
///
/// If `flags` is not empty, it modifies aspects of the lock. See `mlock2(2)` for more information.
///
/// For a version of this function that accepts a raw pointer and length, see [`mlock2_raw()`].
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn mlock2(data: &[u8], flags: Mlock2Flags) -> Result<()> {
    unsafe { mlock2_raw(data.as_ptr(), data.len(), flags) }
}

/// Lock the pages containing any part of the specified region of memory into RAM.
///
/// If `flags` is not empty, it modifies aspects of the lock. See `mlock2(2)` for more information.
///
/// # Safety
///
/// `addr` and `len` must refer to a valid region of memory.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub unsafe fn mlock2_raw(addr: *const u8, len: usize, flags: Mlock2Flags) -> Result<()> {
    Error::unpack_nz(sys::mlock2(addr as *const _, len, flags.bits()))
}

/// Unlock the pages containing any part of the specified region of memory from RAM.
///
/// See `munlock(2)` for more information.
///
/// For a version of this function that accepts a raw pointer and length, see [`munlock_raw()`].
#[inline]
pub fn munlock(data: &[u8]) -> Result<()> {
    unsafe { munlock_raw(data.as_ptr(), data.len()) }
}

/// Unlock the pages containing any part of the specified region of memory from RAM.
///
/// See `munlock(2)` for more information.
///
/// # Safety
///
/// `addr` and `len` must refer to a valid region of memory.
#[inline]
pub unsafe fn munlock_raw(addr: *const u8, len: usize) -> Result<()> {
    Error::unpack_nz(libc::munlock(addr as *const _, len))
}

/// Lock all pages mapped into the address space of the current process into RAM.
///
/// At least one of
/// [`MlockallFlags::CURRENT`](./struct.MlockAllFlags.html#associatedconstant.CURRENT) or
/// [`MlockallFlags::FUTURE`](struct.MlockAllFlags.html#associatedconstant.FUTURE) must be
/// specified in `flags`.
///
/// See `mlockall(2)` and [`MlockAllFlags`] for more information.
#[inline]
pub fn mlockall(flags: MlockAllFlags) -> Result<()> {
    Error::unpack_nz(unsafe { libc::mlockall(flags.bits()) })
}

/// Unlock all pages mapped into the address space of the current process from RAM.
///
/// See `munlockall(2)` for more information.
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

/// Advise the system about this process's expected usage of the given version of memory.
///
/// For a version of this function that accepts a raw pointer and length, see [`posix_madvise_raw()`].
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub fn posix_madvise(data: &mut [u8], advice: PosixMAdvice) -> Result<()> {
    unsafe { posix_madvise_raw(data.as_mut_ptr(), data.len(), advice) }
}

/// Advise the system about this process's expected usage of the given version of memory.
///
/// # Safety
///
/// `addr` and `len` must refer to a valid region of memory.
#[cfg_attr(docsrs, doc(cfg(not(target_os = "android"))))]
#[cfg(not(target_os = "android"))]
#[inline]
pub unsafe fn posix_madvise_raw(addr: *mut u8, length: usize, advice: PosixMAdvice) -> Result<()> {
    Error::unpack_eno(libc::posix_madvise(addr as *mut _, length, advice as _))
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

    #[cfg(target_os = "freebsd")]
    PROTECT = libc::MADV_PROTECT,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    ZERO_WIRED_PAGES = libc::MADV_ZERO_WIRED_PAGES,
}

/// Advise the system about this process's expected usage of the given version of memory.
///
/// # Safety
///
/// Unlike [`posix_madvise()`], on some systems this function can be used to actually modify
/// aspects of the given region of memory; e.g. making the range unavailable in the child of a
/// `fork()`. Use extreme caution and read the OS-specific `madvise(2)` carefully.
///
/// It's recommended to use [`posix_madvise()`] if possible.
#[inline]
pub unsafe fn madvise(data: &mut [u8], advice: MemAdvice) -> Result<()> {
    madvise_raw(data.as_mut_ptr(), data.len(), advice)
}

/// Advise the system about this process's expected usage of the given version of memory.
///
/// # Safety
///
/// See [`madvise()`]. Additionally, `addr` and `len` must refer to a valid region of memory.
#[inline]
pub unsafe fn madvise_raw(addr: *mut u8, length: usize, advice: MemAdvice) -> Result<()> {
    Error::unpack_nz(libc::madvise(addr as *mut _, length, advice as _))
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
bitflags::bitflags! {
    pub struct MemfdFlags: libc::c_uint {
        const CLOEXEC = libc::MFD_CLOEXEC;
        const ALLOW_SEALING = libc::MFD_ALLOW_SEALING;
        const HUGETLB = libc::MFD_HUGETLB;
    }
}

#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn memfd_create<N: AsPath>(name: N, flags: MemfdFlags) -> Result<FileDesc> {
    name.with_cstr(|name| unsafe {
        Error::unpack_fdesc(
            libc::syscall(libc::SYS_memfd_create, name.as_ptr(), flags.bits()) as i32,
        )
    })
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(all(linuxlike, feature = "alloc"))]
    #[test]
    fn test_memfd_create() {
        let mfd = memfd_create("/test/memfd", MemfdFlags::CLOEXEC).unwrap();

        assert_eq!(
            crate::readlink_alloc(format!("/proc/self/fd/{}", mfd.fd())).unwrap(),
            "/memfd:/test/memfd (deleted)"
        );
    }
}
