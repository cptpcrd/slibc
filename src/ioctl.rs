use crate::internal_prelude::*;

/// Call the given `ioctl()`.
///
/// # Safety
///
/// 1. The given `arg` must be a valid pointer to a buffer that is large enough for the given
///    `req`.
/// 2. This must not be used to violate other invariants.
#[inline]
pub unsafe fn ioctl(fd: RawFd, req: libc::c_ulong, ptr: *mut libc::c_void) -> Result<libc::c_int> {
    Error::unpack(libc::ioctl(fd, req as _, ptr))
}

#[inline]
pub fn ioctl_fioclex(fd: RawFd) -> Result<()> {
    unsafe {
        ioctl(fd, libc::FIOCLEX as _, core::ptr::null_mut())?;
    }
    Ok(())
}

#[inline]
pub fn ioctl_fionclex(fd: RawFd) -> Result<()> {
    unsafe {
        ioctl(fd, libc::FIONCLEX as _, core::ptr::null_mut())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fioclex() {
        let (r, _) = crate::pipe().unwrap();

        assert!(!r.get_cloexec().unwrap());

        ioctl_fioclex(r.fd()).unwrap();
        assert!(r.get_cloexec().unwrap());

        ioctl_fionclex(r.fd()).unwrap();
        assert!(!r.get_cloexec().unwrap());
    }
}
