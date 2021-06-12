use core::cmp::Ordering;

/// Search a given byte slice for a given byte.
///
/// This is a simple wrapper around the system's `memchr()` function. For more advanced uses,
/// investigate the `memchr` crate.
#[inline]
pub fn memchr(s: &[u8], c: u8) -> Option<usize> {
    unsafe {
        let ptr = libc::memchr(s.as_ptr() as *const _, c as _, s.len());

        if ptr.is_null() {
            None
        } else {
            Some((ptr as *const u8).offset_from(s.as_ptr()) as usize)
        }
    }
}

/// Search a given byte slice for a given byte, starting from the end.
///
/// This is exactly like [`memchr()`], except it searches backwards.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly"
    )))
)]
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly"
))]
#[inline]
pub fn memrchr(s: &[u8], c: u8) -> Option<usize> {
    unsafe {
        let ptr = libc::memrchr(s.as_ptr() as *const _, c as _, s.len());

        if ptr.is_null() {
            None
        } else {
            Some((ptr as *const u8).offset_from(s.as_ptr()) as usize)
        }
    }
}

/// Compare two memory areas.
///
/// See `memcmp(2)`.
///
/// # Safety
///
/// Both `s1` and `s2` must be valid for reads of size `n` bytes.
#[inline]
pub unsafe fn memcmp_raw(s1: *const u8, s2: *const u8, n: usize) -> Ordering {
    libc::memcmp(s1 as *const _, s2 as *const _, n).cmp(&0)
}

/// Compare two byte slices.
///
/// This is a safe version of [`memcmp_raw()`]. If `s1` and `s2` are different lengths, it returns
/// an `Ordering` comparing their lengths.
///
/// # Example
///
/// ```
/// # use core::cmp::Ordering;
/// # use slibc::memcmp;
/// assert_eq!(memcmp(b"", b""), Ordering::Equal);
/// assert_eq!(memcmp(b"a", b"b"), Ordering::Less);
/// assert_eq!(memcmp(b"ab", b"b"), Ordering::Greater);
/// ```
#[inline]
pub fn memcmp(s1: &[u8], s2: &[u8]) -> Ordering {
    s1.len()
        .cmp(&s2.len())
        .then_with(|| unsafe { memcmp_raw(s1.as_ptr(), s2.as_ptr(), s1.len()) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memchr() {
        assert_eq!(memchr(b"abcdef", b'a'), Some(0));
        assert_eq!(memchr(b"abcdef", b'c'), Some(2));
        assert_eq!(memchr(b"abcdef", b'f'), Some(5));
        assert_eq!(memchr(b"abcdef", b'\0'), None);

        assert_eq!(memchr(b"abcdefabc", b'a'), Some(0));
        assert_eq!(memchr(b"abcdefabc", b'c'), Some(2));
        assert_eq!(memchr(b"abcdefabc", b'f'), Some(5));
        assert_eq!(memchr(b"abcdefabc", b'\0'), None);

        assert_eq!(memchr(b"", b'a'), None);
        assert_eq!(memchr(b"", b'\0'), None);
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly"
    ))]
    #[test]
    fn test_memrchr() {
        assert_eq!(memrchr(b"abcdef", b'a'), Some(0));
        assert_eq!(memrchr(b"abcdef", b'c'), Some(2));
        assert_eq!(memrchr(b"abcdef", b'f'), Some(5));
        assert_eq!(memrchr(b"abcdef", b'\0'), None);

        assert_eq!(memrchr(b"abcdefabc", b'a'), Some(6));
        assert_eq!(memrchr(b"abcdefabc", b'c'), Some(8));
        assert_eq!(memrchr(b"abcdefabc", b'f'), Some(5));
        assert_eq!(memrchr(b"abcdefabc", b'\0'), None);

        assert_eq!(memrchr(b"", b'a'), None);
        assert_eq!(memrchr(b"", b'\0'), None);
    }

    #[test]
    fn test_memcmp_raw() {
        unsafe {
            assert_eq!(memcmp_raw(b"".as_ptr(), b"".as_ptr(), 0), Ordering::Equal);
            assert_eq!(
                memcmp_raw(b"abc".as_ptr(), b"abc".as_ptr(), 3),
                Ordering::Equal
            );

            assert_eq!(
                memcmp_raw(b"abc".as_ptr(), b"abd".as_ptr(), 3),
                Ordering::Less
            );
            assert_eq!(
                memcmp_raw(b"zbc".as_ptr(), b"abc".as_ptr(), 3),
                Ordering::Greater
            );
        }
    }

    #[test]
    fn test_memcmp() {
        assert_eq!(memcmp(b"", b""), Ordering::Equal);
        assert_eq!(memcmp(b"abc", b"abc"), Ordering::Equal);

        assert_eq!(memcmp(b"a", b"b"), Ordering::Less);
        assert_eq!(memcmp(b"abcd", b"abcc"), Ordering::Greater);

        assert_eq!(memcmp(b"ab", b"b"), Ordering::Greater);
        assert_eq!(memcmp(b"ab", b"abc"), Ordering::Less);
    }
}
