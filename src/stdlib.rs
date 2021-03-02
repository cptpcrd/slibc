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
    match unsafe { libc::ptsname_r(fd, buf.as_mut_ptr() as *mut _, buf.len()) } {
        0 => Ok(util::cstr_from_buf(buf).unwrap()),
        eno => Err(Error::from_code(eno)),
    }
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

#[cfg(target_os = "linux")]
bitflags::bitflags! {
    /// Flags for [`getrandom()`].
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[derive(Default)]
    pub struct GrndFlags: libc::c_uint {
        /// Obtain the random data from the `random` source (i.e. `/dev/random`) instead of the
        /// `urandom` source (i.e. `/dev/urandom`) source.
        ///
        /// The `random` source is more limited in its available entropy, and it may not be able to
        /// fill the supplied buffer if not enough bytes are available.
        const RANDOM = libc::GRND_RANDOM;
        /// Fail with `EAGAIN` isntead of blocking if insufficient entropy is available.
        const NONBLOCK = libc::GRND_NONBLOCK;
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
/// This is only available on Linux 3.17 and newer.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn getrandom(buf: &mut [u8], flags: GrndFlags) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            flags.bits(),
        ) as isize
    })
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

#[cfg(all(feature = "alloc", not(target_os = "freebsd")))]
impl Malloc {
    const PTR_SIZE: usize = core::mem::size_of::<*mut libc::c_void>();

    #[inline]
    unsafe fn alloc_memalign(layout: Layout) -> *mut u8 {
        let mut ptr = core::ptr::null_mut();
        libc::posix_memalign(&mut ptr, layout.align(), layout.size());
        ptr as *mut u8
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

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_getrandom() {
        let mut buf = [0; 256];
        assert_eq!(getrandom(&mut buf, GrndFlags::default()).unwrap(), 256);
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

            let ptr = Malloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            assert_eq!(*(ptr as *mut T), core::mem::zeroed::<T>());
            Malloc.dealloc(ptr, layout);
        }

        unsafe {
            check_alloc_single::<bool>();
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

            let ptr = Malloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<T>(), 0);
            assert_eq!(*(ptr as *mut [T; 10]), [core::mem::zeroed(); 10]);
            Malloc.dealloc(ptr, layout);
        }

        unsafe {
            check_alloc_array::<bool>();
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
