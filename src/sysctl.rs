use crate::internal_prelude::*;

/// Get/set the value of the given sysctl.
///
/// This function is a simple wrapper around `libc::sysctl()`.
///
/// `mib` should be a reference to a slice specifying a "Management Information
/// Base"-style name. See OS-specific documentation for details.
///
/// If `old_data` or `new_data` is `None`, `NULL` will be passed for `oldp` or
/// `newp`, respectively. Otherwise, the given slice will be passed.
///
/// In all cases, the return value is the value of `oldlenp` after the `sysctl()`
/// call.
///
/// # Safety
///
/// 1. This function has no way to verify that a slice with elements of type
///    `T` is the correct format for representing the value of the given sysctl.
/// 2. No checking is performed for partial reads that could lead to partially
///    filled out data in the `old_data` slice.
///
/// If it can be verified that neither of these is the case (the data structure
/// is correct for the given option AND the amount of data read is correct for
/// the given structure), then this function should be safe to use.
#[cfg(any(
    target_os = "macos",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
))]
pub unsafe fn sysctl<T>(
    mib: &[libc::c_int],
    old_data: Option<&mut [T]>,
    new_data: Option<&mut [T]>,
) -> Result<usize> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            // macOS requires a non-const pointer for some reason
            let mut mib_buf = [0; 10];
            mib_buf[..mib.len()].copy_from_slice(mib);
            let mib_ptr = mib_buf.as_ptr() as *mut _;
        } else {
            let mib_ptr = mib.as_ptr();
        }
    }

    let (old_ptr, mut old_len) = if let Some(old_data_slice) = old_data {
        (
            old_data_slice.as_mut_ptr(),
            old_data_slice.len() * std::mem::size_of::<T>(),
        )
    } else {
        (std::ptr::null_mut(), 0)
    };

    let (new_ptr, new_len) = if let Some(new_data_slice) = new_data {
        (
            new_data_slice.as_mut_ptr(),
            new_data_slice.len() * std::mem::size_of::<T>(),
        )
    } else {
        (std::ptr::null_mut(), 0)
    };

    Error::unpack_nz(libc::sysctl(
        mib_ptr,
        mib.len() as _,
        old_ptr as *mut libc::c_void,
        &mut old_len,
        new_ptr as *mut libc::c_void,
        new_len,
    ))?;

    Ok(old_len)
}
