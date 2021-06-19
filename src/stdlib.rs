use crate::internal_prelude::*;

/// Given a master pseudoterminal device, obtain the name of the corresponding slave pseudoterminal
/// device.
///
/// **WARNING**: It is **highly recommended** to use [`ptsname_r()`] instead on platforms where it
/// is supported (like Linux).
///
/// # Safety
///
/// The string returned by this function is only valid until the next call to `ptsname()`. That
/// means that not only is the arbitrary `'a` lifetime not accurate, but calling this in a
/// multithreaded program is not safe.
///
/// Again, it is recommended to use [`ptsname_r()`] instead if it is supported.
#[inline]
pub unsafe fn ptsname<'a>(fd: RawFd) -> Result<&'a CStr> {
    let ptr = libc::ptsname(fd);

    if ptr.is_null() {
        Err(Error::last())
    } else {
        Ok(CStr::from_ptr(ptr))
    }
}

/// Given a master pseudoterminal device, obtain the name of the corresponding slave pseudoterminal
/// device.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn ptsname_r(fd: RawFd, buf: &mut [u8]) -> Result<&CStr> {
    Error::unpack_eno(unsafe { libc::ptsname_r(fd, buf.as_mut_ptr() as *mut _, buf.len()) })?;
    Ok(util::cstr_from_buf(buf).unwrap())
}

#[cfg_attr(docsrs, doc(cfg(all(target_os = "linux", feature = "alloc"))))]
#[cfg(all(target_os = "linux", feature = "alloc"))]
pub fn ptsname_alloc(fd: RawFd) -> Result<CString> {
    let maxlen = crate::sysconf(crate::SysconfName::TTY_NAME_MAX).unwrap_or(100);

    let mut buf = Vec::with_capacity(maxlen);
    unsafe {
        buf.set_len(maxlen);
    }

    let len = ptsname_r(fd, &mut buf)?.to_bytes().len();

    buf.truncate(len);
    Ok(unsafe { CString::from_vec_unchecked(buf) })
}

#[cfg(any(linuxlike, freebsdlike))]
bitflags::bitflags! {
    /// Flags for [`getrandom()`].
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
        )))
    )]
    #[derive(Default)]
    pub struct GrndFlags: libc::c_uint {
        /// Obtain the random data from the `random` source (i.e. `/dev/random`) instead of the
        /// `urandom` source (i.e. `/dev/urandom`) source.
        ///
        /// # On Linux/Android
        ///
        /// The `random` source is more limited in its available entropy, and it may not be able to
        /// fill the supplied buffer if not enough bytes are available.
        ///
        /// Note that this flag is ignored since Linux 5.6 (since `/dev/urandom` and `/dev/random`
        /// now use the same pool). See the notes in [`Self::INSECURE`].
        ///
        /// # On FreeBSD/DragonFlyBSD
        ///
        /// This flag is ignored, since `/dev/random` and `/dev/urandom` use the same source on
        /// FreeBSD and DragonFlyBSD.
        const RANDOM = 0x2;
        /// Fail with `EAGAIN` isntead of blocking if insufficient entropy is available.
        const NONBLOCK = 0x1;
        /// If the random pool is not initialized, return non-cryptographic random bytes.
        ///
        /// # On Linux/Android
        ///
        /// Added in Linux 5.6 (will fail with `EINVAL` on older kernels). Cannot be specified
        /// along with [`Self::NONBLOCK`].
        ///
        /// Linux 5.6 made several changes to `/dev/random`, `/dev/urandom`, and `getrandom()`.
        /// Essentially, on Linux 5.6+:
        ///
        /// - Specifying neither [`Self::NONBLOCK`] nor [`Self::INSECURE`] is equivalent to reading
        ///   from `/dev/random`: obtain random bytes, blocking if the pool is not fully
        ///   initialized.
        /// - Specifying [`Self::INSECURE`] is equivalent to reading from `/dev/urandom`: read
        ///   random bytes, even if the pool is not fully initialized (may not be cryptographically
        ///   secure).
        /// - Specifying [`Self::NONBLOCK`] is equivalent to reading from `/dev/random`, except
        ///   that it will fail with `EAGAIN` if the pool is not fully initialized.
        ///
        /// # On FreeBSD/DragonFlyBSD
        ///
        /// This flag is treated as an alias for [`Self::NONBLOCK`], and is only present for API
        /// compatibility with Linux.
        const INSECURE = 0x4;
    }
}

/// Fill a buffer with random data.
///
/// If this function returns `Ok(n)`, the first `n` bytes of the given `buf` have been filled with
/// random bytes.
///
/// When reading from the `urandom` source (i.e. if [`GrndFlags::RANDOM`] is not specified), reads
/// of up to 256 bytes will not be interrupted by signals and will always return as many bytes as
/// requested. Larger requests, however, may return a partially filled buffer or fail with `EINTR`.
///
/// See [`GrndFlags`] for more information.
///
/// This is only available on Linux 3.17 and newer, on FreeBSD 12.0 and newer, or on DragonFlyBSD
/// 5.7 and newer. It will fail with `ENOSYS` on older kernels.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
    )))
)]
#[cfg(any(linuxlike, freebsdlike))]
#[inline]
pub fn getrandom(buf: &mut [u8], flags: GrndFlags) -> Result<usize> {
    #[cfg(linuxlike)]
    let n = Error::unpack_size(unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            flags.bits(),
        ) as isize
    })?;

    #[cfg(freebsdlike)]
    let n = {
        if util::getosreldate_real().unwrap_or(0) < sys::GETRANDOM_FIRST {
            return Err(Error::from_code(libc::ENOSYS));
        }

        Error::unpack_size(unsafe {
            sys::__syscall(
                sys::SYS_GETRANDOM,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                flags.bits(),
            ) as isize
        })?
    };

    Ok(n)
}

/// Fill a buffer (up to 256 bytes) with random data.
///
/// Upon a successful return, the entire buffer has been filled.
///
/// On OpenBSD, this will always succeed unless `buf.len() > 256`. (Note, however, that if the
/// current process is `pledge()`d, the `stdio` promise must be specified to be able to call
/// `getentropy()`).
///
/// On FreeBSD, this is supported since FreeBSD 12.0. On older versions of FreeBSD, this function
/// will fail with `ENOSYS`.
///
/// On macOS, this is supported since macOS 10.12. On older versions of macOS, this function will
/// fail with `ENOSYS`.
///
/// On Linux, this is supported since glibc 2.25 (or musl 1.1.20), and only if `getrandom(2)` is
/// available (on Linux 3.17+). On older versions of glibc/musl, and on older kernels, this
/// function will fail with `ENOSYS`.
///
/// On Android, this is supported since API 28, or Android 9. It will fail with `ENOSYS` on older
/// versions.
///
/// On FreeBSD, macOS, and Linux/Android, this function will also fail with `ENOSYS` in statically
/// linked programs.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "macos"
    )))
)]
#[cfg(any(
    linuxlike,
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "macos",
))]
#[inline]
pub fn getentropy(buf: &mut [u8]) -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "openbsd")] {
            Error::unpack_nz(unsafe { libc::getentropy(buf.as_mut_ptr() as *mut _, buf.len()) })?;
        } else {
            static GETENTROPY: util::DlFuncLoader<
                unsafe extern "C" fn(*mut libc::c_void, usize) -> libc::c_int,
            > = unsafe { util::DlFuncLoader::new(b"getentropy\0") };

            if let Some(func) = GETENTROPY.get() {
                Error::unpack_nz(unsafe { func(buf.as_mut_ptr() as *mut _, buf.len()) })?;
            } else {
                return Err(Error::from_code(libc::ENOSYS));
            }
        }
    }

    Ok(())
}

/// Get the absolute, canonicalized version of the given `path`.
///
/// This corresponds to `std::fs::canonicalize()` in the standard library.
///
/// `buf` must be an array [`PATH_MAX`](./constant.PATH_MAX.html) bytes long; the resolved path
/// will be stored in there. To use a dynamically allocated buffer, see [`realpath_unchecked()`].
#[inline]
pub fn realpath<P: AsPath>(path: P, buf: &mut [u8; crate::PATH_MAX]) -> Result<&CStr> {
    // SAFETY: `buf` is guaranteed to be at least PATH_MAX bytes long
    unsafe { realpath_unchecked(path, buf) }
}

/// Get the absolute, canonicalized version of the given `path`.
///
/// This is identical to [`realpath()`], except that the `buf` argument takes a slice (which may be
/// dynamically allocated) instead of an array.
///
/// # Safety
///
/// `buf` must be at least [`PATH_MAX`](./constant.PATH_MAX.html) bytes long. (This is verified if
/// debug assertions are enabled.)
#[inline]
pub unsafe fn realpath_unchecked<P: AsPath>(path: P, buf: &mut [u8]) -> Result<&CStr> {
    debug_assert!(buf.len() >= crate::PATH_MAX);
    path.with_cstr(|path| {
        Error::unpack_ptr(libc::realpath(path.as_ptr(), buf.as_mut_ptr() as *mut _))
    })?;

    Ok(util::cstr_from_buf(buf).unwrap())
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
pub fn realpath_alloc<P: AsPath>(path: P) -> Result<CString> {
    let mut buf = Vec::with_capacity(crate::PATH_MAX);
    unsafe {
        buf.set_len(crate::PATH_MAX);
    }

    let len = unsafe { realpath_unchecked(path, &mut buf)? }
        .to_bytes()
        .len();
    buf.truncate(len);
    Ok(unsafe { CString::from_vec_unchecked(buf) })
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::alloc::{GlobalAlloc, Layout};
#[cfg(feature = "std")]
use std::alloc::{GlobalAlloc, Layout};

/// A memory allocator that uses the built-in `malloc()` family of functions to allocate memory.
///
/// This struct implements `GlobalAlloc`. When `Allocator` is stabilized, it may implement that as
/// well.
///
/// Some notes:
///
/// - The `GlobalAlloc` implementation does NOT handle zero-sized allocations specially. Behavior
///   in that case is "whatever the system's `malloc()` does."
///
///   (If and when an implementation of `Allocator` is added, the `Allocator` implementation *will*
///   handle zero-sized allocations specially, since that is required by `Allocator`.)
///
/// - On FreeBSD, the `*allocx()` functions are used, since they provide better handling of large
///   alignments.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
#[derive(Copy, Clone, Debug, Default)]
pub struct Malloc;

#[cfg(feature = "alloc")]
impl Malloc {
    #[cfg(not(target_os = "freebsd"))]
    const PTR_SIZE: usize = core::mem::size_of::<*mut libc::c_void>();

    #[cfg(not(target_os = "freebsd"))]
    #[inline]
    unsafe fn alloc_memalign(layout: Layout) -> *mut u8 {
        let mut ptr = core::ptr::null_mut();
        libc::posix_memalign(&mut ptr, layout.align(), layout.size());
        ptr as *mut u8
    }

    /// Attempt to retrieve the amount of usable space in the block of memory (allocated using
    /// `malloc()`) that starts at `ptr`.
    ///
    /// If the usable size cannot be determined, this function will return `None`. Callers should
    /// always be prepared to handle this, e.g. by falling back on using the original allocation
    /// size.
    ///
    /// Currently, the usable size can only be retrieved on Linux/Android, macOS/iOS, and FreeBSD;
    /// however, that may change without notice. Callers should not e.g. assume that the usable size
    /// can always be determined on any particular platform.
    ///
    /// # Safety
    ///
    /// `ptr` must be a non-NULL pointer representing an allocation obtained from the `malloc()`
    /// family of functions (e.g. using this struct).
    #[allow(unreachable_code, unused_variables)]
    #[inline]
    pub unsafe fn usable_size(&self, ptr: *mut u8) -> Option<usize> {
        #[cfg(linuxlike)]
        return Some(sys::malloc_usable_size(ptr as *mut _));
        #[cfg(apple)]
        return Some(sys::malloc_size(ptr as *const _));
        #[cfg(target_os = "freebsd")]
        return Some(sys::sallocx(ptr as *mut _, 0));

        None
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
unsafe impl GlobalAlloc for Malloc {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "freebsd")] {
            #[inline]
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                sys::mallocx(layout.size(), sys::MALLOCX_ALIGN(layout.align())) as *mut u8
            }

            #[inline]
            unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
                sys::mallocx(
                    layout.size(), sys::MALLOCX_ALIGN(layout.align()) | sys::MALLOCX_ZERO
                ) as *mut u8
            }

            #[inline]
            unsafe fn realloc(
                &self, old_ptr: *mut u8, layout: Layout, new_size: usize,
            ) -> *mut u8 {
                sys::rallocx(
                    old_ptr as *mut _, new_size, sys::MALLOCX_ALIGN(layout.align())
                ) as *mut u8
            }

            #[inline]
            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                sys::sdallocx(ptr as *mut _, layout.size(), 0);
            }
        } else {
            #[inline]
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                if layout.align() <= Self::PTR_SIZE {
                    libc::malloc(layout.size()) as *mut u8
                } else {
                    Self::alloc_memalign(layout)
                }
            }

            #[inline]
            unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
                if layout.align() <= Self::PTR_SIZE {
                    libc::calloc(1, layout.size()) as *mut u8
                } else {
                    let ptr = Self::alloc_memalign(layout);
                    if !ptr.is_null() {
                        core::ptr::write_bytes(ptr, 0, layout.size());
                    }
                    ptr
                }
            }

            #[inline]
            unsafe fn realloc(
                &self, old_ptr: *mut u8, layout: Layout, new_size: usize,
            ) -> *mut u8 {
                if layout.align() <= Self::PTR_SIZE {
                    libc::realloc(old_ptr as *mut _, new_size) as *mut u8
                } else {
                    let new_ptr = Self::alloc_memalign(
                        Layout::from_size_align_unchecked(new_size, layout.align())
                    );

                    if !new_ptr.is_null() {
                        core::ptr::copy_nonoverlapping(old_ptr, new_ptr, layout.size());
                        self.dealloc(old_ptr, layout);
                    }
                    new_ptr
                }
            }

            #[inline]
            unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
                libc::free(ptr as *mut _);
            }
        }
    }
}

#[inline]
pub fn abort() -> ! {
    unsafe { libc::abort() }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[cfg(any(linuxlike, freebsdlike))]
    fn has_getrandom() -> bool {
        #[cfg(target_os = "freebsd")]
        if crate::getosreldate().unwrap() < 1200061 {
            return false;
        }
        true
    }

    #[cfg(any(linuxlike, freebsdlike))]
    #[test]
    fn test_getrandom() {
        if !has_getrandom() {
            assert_eq!(
                getrandom(&mut [], GrndFlags::default()).unwrap_err(),
                Errno::ENOSYS
            );
            return;
        }

        let mut buf = [0; 256];
        assert_eq!(getrandom(&mut buf, GrndFlags::default()).unwrap(), 256);
    }

    #[cfg(any(
        linuxlike,
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "macos",
    ))]
    fn has_getentropy() -> bool {
        if cfg!(target_feature = "crt-static") {
            return false;
        }
        #[cfg(target_os = "freebsd")]
        if crate::getosreldate().unwrap() < 1200061 {
            return false;
        }
        true
    }

    #[cfg(any(
        linuxlike,
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "macos",
    ))]
    #[test]
    fn test_getentropy() {
        if !has_getentropy() {
            assert_eq!(getentropy(&mut []).unwrap_err(), Errno::ENOSYS);
            return;
        }

        let mut buf = [0; 256];
        getentropy(&mut buf).unwrap();
    }

    #[test]
    fn test_realpath() {
        let mut cwdbuf = [0; crate::PATH_MAX];
        let cwd = crate::getcwd(&mut cwdbuf).unwrap();

        let mut buf = [0; crate::PATH_MAX];
        let path = realpath(crate::c_paths::dot(), &mut buf).unwrap();
        assert_eq!(path, cwd);

        let mut buf = [0; crate::PATH_MAX];
        let path = realpath(crate::c_paths::slash(), &mut buf).unwrap();
        assert_eq!(path, CStr::from_bytes_with_nul(b"/\0").unwrap());

        assert_eq!(
            realpath(CStr::from_bytes_with_nul(b"/NOEXIST\0").unwrap(), &mut buf).unwrap_err(),
            Errno::ENOENT
        );

        #[cfg(feature = "alloc")]
        {
            assert_eq!(realpath_alloc(".").unwrap().as_c_str(), cwd);
            assert_eq!(
                realpath_alloc("/").unwrap().as_c_str(),
                realpath("/", &mut buf).unwrap()
            );
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_malloc() {
        // Having 16-bit alignment forces this code to use posix_memalign() instead of
        // malloc()/calloc()/realloc()
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[repr(C, align(16))]
        pub struct Aligned16 {
            a: u64,
            b: u64,
        }

        unsafe fn check_alloc_single<T: core::fmt::Debug + core::cmp::PartialEq>() {
            let layout = Layout::new::<T>();

            let ptr = Malloc.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            Malloc.dealloc(ptr, layout);
            if let Some(size) = Malloc.usable_size(ptr) {
                assert!(size >= layout.size());
            }

            let ptr = Malloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            assert_eq!(*(ptr as *mut T), core::mem::zeroed::<T>());
            Malloc.dealloc(ptr, layout);
            if let Some(size) = Malloc.usable_size(ptr) {
                assert!(size >= layout.size());
            }
        }

        unsafe {
            check_alloc_single::<u8>();
            check_alloc_single::<u16>();
            check_alloc_single::<u32>();
            check_alloc_single::<u64>();
            check_alloc_single::<u128>();
            check_alloc_single::<usize>();
            check_alloc_single::<Aligned16>();
        }

        unsafe fn check_alloc_array<
            T: core::fmt::Debug + core::cmp::PartialEq + core::marker::Copy,
        >() {
            let layout = Layout::array::<T>(10).unwrap();

            let ptr = Malloc.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            Malloc.dealloc(ptr, layout);
            if let Some(size) = Malloc.usable_size(ptr) {
                assert!(size >= layout.size());
            }

            let ptr = Malloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            assert_eq!(*(ptr as *mut [T; 10]), [core::mem::zeroed(); 10]);
            Malloc.dealloc(ptr, layout);
            if let Some(size) = Malloc.usable_size(ptr) {
                assert!(size >= layout.size());
            }
        }

        unsafe {
            check_alloc_array::<u8>();
            check_alloc_array::<u16>();
            check_alloc_array::<u32>();
            check_alloc_array::<u64>();
            check_alloc_array::<u128>();
            check_alloc_array::<usize>();
            check_alloc_array::<Aligned16>();
        }
    }
}
