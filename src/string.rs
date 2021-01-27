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
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly"
    )))
)]
#[cfg(any(
    target_os = "linux",
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
}
