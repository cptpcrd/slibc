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
