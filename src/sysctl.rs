use crate::internal_prelude::*;

/// The maximum length of an MIB for [`sysctl()`] on this platform.
///
/// This may be useful e.g. as the length of a buffer used to store the result from
/// `sysctlnametomib()`.
pub const CTL_MAXNAME: usize = sys::CTL_MAXNAME as usize;

#[inline]
fn prepare_opt_slice<T>(s: Option<&[T]>) -> (*const libc::c_void, usize) {
    if let Some(s) = s {
        (s.as_ptr() as *const _, s.len() * core::mem::size_of::<T>())
    } else {
        (core::ptr::null(), 0)
    }
}

#[inline]
fn prepare_opt_slice_mut<T>(s: Option<&mut [T]>) -> (*mut libc::c_void, usize) {
    if let Some(s) = s {
        (
            s.as_mut_ptr() as *mut _,
            s.len() * core::mem::size_of::<T>(),
        )
    } else {
        (core::ptr::null_mut(), 0)
    }
}

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
/// call. If `old_data` is `None`, this may be an estimate (possibly rounded up if it can change
/// often) of the length of the currently available information. See OS-specific documentation for
/// more details.
///
/// # Safety
///
/// 1. This function has no way to verify that a slice with elements of type
///    `T` is the correct format for representing the value of the given sysctl.
/// 2. No checking is performed for partial reads that could lead to partially
///    filled out data in the `old_data` slice (i.e. the returned length must be checked).
/// 3. When running as root, `sysctl()` can be used to alter aspects of the system, possibly in
///    unsafe ways. Read the documentation carefully.
///
/// If it can be verified that none of these is the case, then this function should be safe to
/// use.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
pub unsafe fn sysctl<T>(
    mib: &[libc::c_int],
    old_data: Option<&mut [T]>,
    new_data: Option<&[T]>,
) -> Result<usize> {
    // Do a bounds check up front so we can a) copy it into a buffer on macOS and b) cast it to an
    // unsigned int for calling sysctl() without issues.
    if mib.len() > sys::CTL_MAXNAME as usize {
        return Err(Error::from_code(libc::EINVAL));
    }

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "macos", target_os = "ios"))] {
            // macOS requires a non-const pointer for some reason
            let mut mib_buf = [0; sys::CTL_MAXNAME as usize];
            mib_buf[..mib.len()].copy_from_slice(mib);
            let mib_ptr = mib_buf.as_mut_ptr();
        } else {
            let mib_ptr = mib.as_ptr();
        }
    }

    let (old_ptr, mut old_len) = prepare_opt_slice_mut(old_data);
    let (new_ptr, new_len) = prepare_opt_slice(new_data);

    // macOS and OpenBSD also want a mutable pointer here... but they *shouldn't* actually write to
    // the data, so just casting it should be fine.
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "openbsd"))]
    let new_ptr = new_ptr as *mut _;

    Error::unpack_nz(libc::sysctl(
        mib_ptr,
        mib.len() as _,
        old_ptr,
        &mut old_len,
        new_ptr,
        new_len,
    ))?;

    Ok(old_len)
}

/// Get/set the value of the sysctl with the given name.
///
/// This is equivalent to looking up the MIB of the sysctl with [`sysctlnametomib()`], then
/// calling [`sysctl()`] with the that MIB. (In fact, that may actually be preferable if repeated
/// lookups of the same sysctl are planned.)
///
/// # Safety
///
/// See [`sysctl()`].
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(not(target_os = "openbsd"))]
pub unsafe fn sysctlbyname<T, P: AsPath>(
    name: P,
    old_data: Option<&mut [T]>,
    new_data: Option<&[T]>,
) -> Result<usize> {
    let (old_ptr, mut old_len) = prepare_opt_slice_mut(old_data);
    let (new_ptr, new_len) = prepare_opt_slice(new_data);

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let new_ptr = new_ptr as *mut _;

    name.with_cstr(|name| {
        Error::unpack_nz(libc::sysctlbyname(
            name.as_ptr(),
            old_ptr,
            &mut old_len,
            new_ptr,
            new_len,
        ))
    })?;

    Ok(old_len)
}

/// Look up the MIB of the sysctl with the given name.
///
/// The name of the sysctl is specified by `name`, and the MIB will be copied into `mib`. The
/// length of the MIB will be returned upon success.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(not(target_os = "openbsd"))]
pub fn sysctlnametomib<P: AsPath>(name: P, mib: &mut [libc::c_int]) -> Result<usize> {
    let mut size = mib.len();
    name.with_cstr(|name| {
        Error::unpack_nz(unsafe {
            sys::sysctlnametomib(name.as_ptr(), mib.as_mut_ptr(), &mut size)
        })
    })?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_opt_slice() {
        macro_rules! check_arr {
            ($type:ty, $arr:expr $(,)?) => {{
                let slice = &mut $arr;

                assert_eq!(
                    prepare_opt_slice::<$type>(Some(slice)),
                    (
                        slice.as_ptr() as *const _,
                        slice.len() * core::mem::size_of::<$type>(),
                    )
                );

                assert_eq!(
                    prepare_opt_slice_mut::<$type>(Some(slice)),
                    (
                        slice.as_mut_ptr() as *mut _,
                        slice.len() * core::mem::size_of::<$type>(),
                    )
                );
            }};
        }

        check_arr!(i32, []);
        check_arr!(i32, [1, 2, 3]);
        check_arr!(usize, [1, 2, 3, 4, 5, 6]);

        assert_eq!(prepare_opt_slice::<i32>(None), (core::ptr::null(), 0));
        assert_eq!(
            prepare_opt_slice_mut::<i32>(None),
            (core::ptr::null_mut(), 0)
        );
    }

    #[cfg(any(freebsdlike, apple))]
    #[test]
    fn test_sysctlnametomib() {
        let mut buf = [0; CTL_MAXNAME];

        #[cfg(not(apple))]
        {
            let n = sysctlnametomib(
                CStr::from_bytes_with_nul(b"hw.pagesize\0").unwrap(),
                &mut buf,
            )
            .unwrap();
            assert_eq!(&buf[..n], &[libc::CTL_HW, libc::HW_PAGESIZE]);
        }

        let n = sysctlnametomib(
            CStr::from_bytes_with_nul(b"kern.argmax\0").unwrap(),
            &mut buf,
        )
        .unwrap();
        assert_eq!(&buf[..n], &[libc::CTL_KERN, libc::KERN_ARGMAX]);
    }

    #[cfg(any(freebsdlike, apple))]
    #[test]
    fn test_sysctlbyname() {
        let pagesize = crate::getpagesize();

        let mut pgsz = 0i32;
        let n = unsafe {
            sysctlbyname(
                CStr::from_bytes_with_nul(b"hw.pagesize\0").unwrap(),
                Some(core::slice::from_mut(&mut pgsz)),
                None,
            )
            .unwrap()
        };

        assert_eq!(n, core::mem::size_of::<i32>());
        assert_eq!(pgsz, pagesize as i32);
    }

    #[test]
    fn test_sysctl_error() {
        assert_eq!(
            unsafe { sysctl::<i32>(&[0; CTL_MAXNAME + 1], None, None).unwrap_err() },
            Errno::EINVAL,
        );

        assert_eq!(
            unsafe { sysctl::<i32>(&[], None, None).unwrap_err() },
            Errno::EINVAL,
        );
    }
}
