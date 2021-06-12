use crate::internal_prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct Winsize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

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

/// Call the `FIOCLEX` `ioctl()`.
///
/// On platforms that support it (Linux/Android, macOS, \*BSD), this sets the close-on-exec flag on
/// the given file descriptor.
///
/// On Linux, this does not work on `O_PATH` file descriptors.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(any(linuxlike, bsd))]
#[inline]
pub fn ioctl_fioclex(fd: RawFd) -> Result<()> {
    unsafe {
        ioctl(fd, libc::FIOCLEX as _, core::ptr::null_mut())?;
    }
    Ok(())
}

/// Call the `FIONCLEX` `ioctl()`.
///
/// On platforms that support it (Linux/Android, macOS, \*BSD), this unsets the close-on-exec flag on
/// the given file descriptor.
///
/// On Linux, this does not work on `O_PATH` file descriptors.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(any(linuxlike, bsd))]
#[inline]
pub fn ioctl_fionclex(fd: RawFd) -> Result<()> {
    unsafe {
        ioctl(fd, libc::FIONCLEX as _, core::ptr::null_mut())?;
    }
    Ok(())
}

/// Call the `FIONBIO` `ioctl()`.
///
/// This allows toggling the non-blocking flag on the given file descriptor.
#[inline]
pub fn ioctl_fionbio(fd: RawFd, nonblock: bool) -> Result<()> {
    // The argument seems to be read as an `int` by most platforms
    let mut nonblock = nonblock as libc::c_int;
    unsafe {
        ioctl(fd, libc::FIONBIO as _, &mut nonblock as *mut _ as *mut _)?;
    }
    Ok(())
}

#[inline]
pub fn ioctl_getwinsz(fd: RawFd) -> Result<Winsize> {
    let mut winsize = MaybeUninit::uninit();
    unsafe {
        ioctl(fd, libc::TIOCGWINSZ as _, winsize.as_mut_ptr() as *mut _)?;
    }
    Ok(unsafe { winsize.assume_init() })
}

#[inline]
pub fn ioctl_setwinsz(fd: RawFd, winsz: &Winsize) -> Result<()> {
    unsafe {
        ioctl(fd, libc::TIOCSWINSZ as _, winsz as *const _ as *mut _)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(linuxlike, bsd))]
    #[test]
    fn test_fioclex() {
        let (r, _) = crate::pipe().unwrap();

        assert!(!r.get_cloexec().unwrap());

        ioctl_fioclex(r.fd()).unwrap();
        assert!(r.get_cloexec().unwrap());

        ioctl_fionclex(r.fd()).unwrap();
        assert!(!r.get_cloexec().unwrap());
    }

    #[test]
    fn test_fionbio() {
        let (r, _) = crate::pipe().unwrap();

        assert!(!r.get_nonblocking().unwrap());

        ioctl_fionbio(r.fd(), true).unwrap();
        assert!(r.get_nonblocking().unwrap());

        ioctl_fionbio(r.fd(), false).unwrap();
        assert!(!r.get_nonblocking().unwrap());
    }
}
