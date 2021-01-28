use crate::internal_prelude::*;

/// Open a pseudoterminal.
///
/// On success, this returns a tuple of the `(master, slave)` file descriptors.
///
/// # Safety
///
/// This function may call non-thread-safe functions such as
/// [`ptsname()`](./fn.ptsname.html) internally. It should not be called concurrently from multiple
/// threads, or concurrently with `ptsname()`.
#[inline]
pub unsafe fn openpty(winsize: Option<&crate::Winsize>) -> Result<(FileDesc, FileDesc)> {
    let mut master = -1;
    let mut slave = -1;

    let mut winsize = winsize.copied();

    Error::unpack_nz(libc::openpty(
        &mut master,
        &mut slave,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        winsize
            .as_mut()
            .map(|w| w as *mut _ as *mut _)
            .unwrap_or_else(core::ptr::null_mut),
    ))?;

    Ok((FileDesc::new(master), FileDesc::new(slave)))
}

/// Prepare for a login on the given terminal device, which may be a real terminal or a slave
/// pseudoterminal.
///
/// This performs the following steps:
/// 1. Create a new session.
/// 2. Make the given `fd` the controlling terminal for this process.
/// 3. Make the given `fd` the standard input/output/error for this process.
/// 4. Close `fd`.
///
/// # Safety
///
/// 1. This function closes `fd`, so it must not be used elsewhere.
/// 2. Some of the other operations may not be thread-safe.
#[inline]
pub unsafe fn login_tty(fd: RawFd) -> Result<()> {
    Error::unpack_nz(libc::login_tty(fd))
}
