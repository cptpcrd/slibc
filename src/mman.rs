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
    Error::unpack_nz(unsafe { libc::mlock(data.as_ptr() as *const _, data.len()) })
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn mlock2(data: &[u8], flags: Mlock2Flags) -> Result<()> {
    Error::unpack_nz(unsafe { sys::mlock2(data.as_ptr() as *const _, data.len(), flags.bits()) })
}

#[inline]
pub fn munlock(data: &[u8]) -> Result<()> {
    Error::unpack_nz(unsafe { libc::munlock(data.as_ptr() as *const _, data.len()) })
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
