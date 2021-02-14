use crate::internal_prelude::*;

#[cfg(any(target_os = "linux", target_os = "dragonfly"))]
pub use libc::__errno_location as errno_ptr;

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub use libc::__error as errno_ptr;

#[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
pub use libc::__errno as errno_ptr;

#[inline]
pub fn cvt_char_buf(buf: &[libc::c_char]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) }
}

#[inline]
pub fn cstr_from_buf(buf: &[u8]) -> Option<&CStr> {
    let index = crate::memchr(buf, 0)?;
    debug_assert!(index < buf.len());
    let buf = unsafe { &buf.get_unchecked(0..index + 1) };

    #[cfg(debug_assertions)]
    CStr::from_bytes_with_nul(buf).unwrap();

    Some(unsafe { CStr::from_bytes_with_nul_unchecked(buf) })
}

#[inline]
pub fn osstr_from_buf(buf: &[u8]) -> &OsStr {
    OsStr::from_bytes(match crate::memchr(buf, 0) {
        Some(index) => &buf[..index],
        None => buf,
    })
}

#[cfg(feature = "alloc")]
#[inline]
pub fn cstring_from_buf(mut buf: Vec<u8>) -> Option<CString> {
    let index = crate::memchr(&buf, 0)?;
    buf.truncate(index);
    Some(unsafe { CString::from_vec_unchecked(buf) })
}

/// Equivalent to `CStr::from_ptr(ptr).to_bytes()`, but (possibly) slightly faster.
#[inline]
pub unsafe fn bytes_from_ptr<'a>(ptr: *const libc::c_char) -> &'a [u8] {
    core::slice::from_raw_parts(ptr as *const u8, libc::strlen(ptr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::unnecessary_cast)]
    #[test]
    fn test_cvt_char_buf() {
        assert_eq!(cvt_char_buf(&[]), &[]);
        assert_eq!(cvt_char_buf(&[0 as libc::c_char, 1, 2]), &[0u8, 1, 2]);
    }

    #[test]
    fn test_cstr_from_buf() {
        let abc = CStr::from_bytes_with_nul(b"abc\0").unwrap();
        let empty = CStr::from_bytes_with_nul(b"\0").unwrap();

        assert_eq!(cstr_from_buf(b"abc\0"), Some(abc));
        assert_eq!(cstr_from_buf(b"abc\0def"), Some(abc));
        assert_eq!(cstr_from_buf(b"\0abc\0def"), Some(empty));
        assert_eq!(cstr_from_buf(b"\0"), Some(empty));

        assert_eq!(cstr_from_buf(b""), None);
        assert_eq!(cstr_from_buf(b"abc"), None);
    }

    #[test]
    fn test_osstr_from_buf() {
        let abc = OsStr::new("abc");
        let empty = OsStr::new("");

        assert_eq!(osstr_from_buf(b"abc\0"), abc);
        assert_eq!(osstr_from_buf(b"abc\0def"), abc);
        assert_eq!(osstr_from_buf(b"\0abc\0def"), empty);
        assert_eq!(osstr_from_buf(b"\0"), empty);

        assert_eq!(osstr_from_buf(b""), empty);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_cstring_from_buf() {
        let abc = CString::new(*b"abc").unwrap();
        let empty = CString::new(*b"").unwrap();

        assert_eq!(cstring_from_buf(b"abc\0".to_vec()), Some(abc.clone()));
        assert_eq!(cstring_from_buf(b"abc\0def".to_vec()), Some(abc));
        assert_eq!(
            cstring_from_buf(b"\0abc\0def".to_vec()),
            Some(empty.clone())
        );
        assert_eq!(cstring_from_buf(b"\0".to_vec()), Some(empty));

        assert_eq!(cstring_from_buf(b"".to_vec()), None);
        assert_eq!(cstring_from_buf(b"abc".to_vec()), None);
    }
}
