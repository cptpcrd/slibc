use crate::internal_prelude::*;

#[inline]
pub fn rename<O: AsPath, N: AsPath>(oldpath: O, newpath: N) -> Result<()> {
    oldpath.with_cstr(|oldpath| {
        newpath.with_cstr(|newpath| {
            Error::unpack_nz(unsafe { libc::rename(oldpath.as_ptr(), newpath.as_ptr()) })
        })
    })
}

#[inline]
pub fn renameat<O: AsPath, N: AsPath>(
    olddirfd: RawFd,
    oldpath: O,
    newdirfd: RawFd,
    newpath: N,
) -> Result<()> {
    oldpath.with_cstr(|oldpath| {
        newpath.with_cstr(|newpath| {
            Error::unpack_nz(unsafe {
                libc::renameat(olddirfd, oldpath.as_ptr(), newdirfd, newpath.as_ptr())
            })
        })
    })
}

#[cfg(linuxlike)]
bitflags::bitflags! {
    /// Extra flags for [`renameat2()`].
    ///
    /// See `renameat2(2)` for more information.
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    pub struct RenameFlags: libc::c_uint {
        /// Don't overwrite the destination path; fail with `EEXIST` if it already exists.
        ///
        /// This requires support for the underlying filesystem (support has been added to more
        /// filesystems in newer kernel versions).
        const NOREPLACE = 1;
        /// Atomically exchange the two paths.
        ///
        /// Both paths must exist, but can be of different types. This cannot be used together with
        /// [`Self::NOREPLACE`] or [`Self::WHITEOUT`].
        const EXCHANGE = 2;
        /// Create a "whiteout" object at the source of the rename.
        ///
        /// This may be useful for overlay/union filesystem implementations. It requires the
        /// CAP_MKNOD capability, and needs support for the underlying filesystem (see
        /// `renameat2(2)` for a list of supporting filesystems). It was added in Linux 3.18.
        const WHITEOUT = 4;
    }
}

/// Equivalent to [`renameat()`], but has an additional `flags` argument.
///
/// `renameat2()` was added in Linux 3.15; this function will fail with `ENOSYS` on older versions.
/// This function will also fail with `EINVAL` if one of the given `flags` is not supported by the
/// filesystem (or the kernel), or if the given `flags` conflict with each other.
///
/// Callers should check for (minimally) `ENOSYS` and `EINVAL`, and if possible fall back on an
/// alternative implementation in those cases.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn renameat2<O: AsPath, N: AsPath>(
    olddirfd: RawFd,
    oldpath: O,
    newdirfd: RawFd,
    newpath: N,
    flags: RenameFlags,
) -> Result<()> {
    oldpath.with_cstr(|oldpath| {
        newpath.with_cstr(|newpath| {
            Error::unpack_nz(unsafe {
                libc::syscall(
                    libc::SYS_renameat2,
                    olddirfd,
                    oldpath.as_ptr(),
                    newdirfd,
                    newpath.as_ptr(),
                    flags.bits(),
                ) as i32
            })
        })
    })
}
