use crate::internal_prelude::*;

use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

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
pub fn cvt_u8_buf(buf: &[u8]) -> &[libc::c_char] {
    unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const _, buf.len()) }
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
        Some(index) => {
            debug_assert!(index < buf.len());
            unsafe { &buf.get_unchecked(0..index) }
        }
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
#[allow(unused)]
#[inline]
pub unsafe fn bytes_from_ptr<'a>(ptr: *const libc::c_char) -> &'a [u8] {
    core::slice::from_raw_parts(ptr as *const u8, libc::strlen(ptr))
}

/// Wraps an iterator and displays it like a list.
pub struct DebugListField<T: fmt::Debug, I: Iterator<Item = T> + Clone>(pub I);

impl<T, I> fmt::Debug for DebugListField<T, I>
where
    T: fmt::Debug,
    I: Iterator<Item = T> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.0.clone()).finish()
    }
}

pub trait IntParseBytes: Default {
    fn _parse_bytes_push_digit(self, base: u8, digit: u8) -> Option<Self>;

    fn _parse_bytes_negate(self) -> Option<Self>;

    #[inline]
    fn parse_bytes(
        bytes: &[u8],
        allow_signs: bool,
    ) -> core::result::Result<Self, IntParseBytesError> {
        Self::parse_bytes_radix(bytes, 10, allow_signs)
    }

    fn parse_bytes_radix(
        mut bytes: &[u8],
        radix: u8,
        allow_signs: bool,
    ) -> core::result::Result<Self, IntParseBytesError> {
        let mut negated = false;

        match bytes.split_first() {
            None => return Err(IntParseBytesError::Empty),

            Some((&b'+', rest)) if allow_signs => bytes = rest,

            Some((&b'-', rest)) if allow_signs => {
                bytes = rest;
                negated = true;
            }

            _ => (),
        }

        let mut res = Self::default();

        for &ch in bytes {
            let digit = if (b'0'..=b'9').contains(&ch) {
                ch - b'0'
            } else if (b'a'..=b'z').contains(&ch) {
                (ch - b'a') + 10
            } else if (b'A'..=b'Z').contains(&ch) {
                (ch - b'A') + 10
            } else {
                return Err(IntParseBytesError::InvalidDigit);
            };

            if digit >= radix {
                return Err(IntParseBytesError::InvalidDigit);
            }

            res = res
                ._parse_bytes_push_digit(radix, digit)
                .ok_or(IntParseBytesError::Overflow)?
        }

        if negated {
            res = res
                ._parse_bytes_negate()
                .ok_or(IntParseBytesError::Overflow)?;
        }

        Ok(res)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum IntParseBytesError {
    Empty,
    InvalidDigit,
    Overflow,
}

impl fmt::Display for IntParseBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "cannot parse from empty data",
            Self::InvalidDigit => "invalid digit",
            Self::Overflow => "numerical overflow while parsing",
        })
    }
}

macro_rules! parse_bytes_int_impl {
    ($($ty:ty)*) => {
        $(
            impl IntParseBytes for $ty {
                #[inline]
                fn _parse_bytes_push_digit(self, base: u8, digit: u8) -> Option<Self> {
                    self.checked_mul(base as _)?.checked_add(digit as _)
                }

                #[inline]
                fn _parse_bytes_negate(self) -> Option<Self> {
                    self.checked_neg()
                }
            }
        )*
    };
}

parse_bytes_int_impl! { u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize }

#[allow(dead_code)]
pub struct DlFuncLoader<F> {
    name: &'static [u8],
    addr: AtomicUsize,
    func: PhantomData<F>,
}

#[allow(dead_code)]
impl<F> DlFuncLoader<F> {
    #[inline]
    pub const unsafe fn new(name: &'static [u8]) -> Self {
        Self {
            name,
            addr: AtomicUsize::new(0),
            func: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> Option<F> {
        debug_assert_eq!(self.name.last(), Some(&0));
        assert_eq!(core::mem::size_of::<F>(), core::mem::size_of::<usize>());

        if cfg!(target_feature = "crt-static") {
            // dlsym() won't work from statically linked executables... don't even try
            // This may also let the compiler optimize more out
            return None;
        }

        let addr = match self.addr.load(Ordering::SeqCst) {
            0 => {
                let addr =
                    unsafe { libc::dlsym(libc::RTLD_DEFAULT, self.name.as_ptr() as *const _) }
                        as *const u8;
                if addr.is_null() {
                    self.addr.store(usize::MAX, Ordering::SeqCst);
                    return None;
                }
                self.addr.store(addr as usize, Ordering::SeqCst);
                addr
            }
            usize::MAX => return None,
            addr => addr as *const u8,
        };

        debug_assert!(!addr.is_null());
        Some(unsafe { core::mem::transmute_copy(&addr) })
    }
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

    #[allow(clippy::unnecessary_cast)]
    #[test]
    fn test_cvt_u8_buf() {
        assert_eq!(cvt_u8_buf(&[]), &[]);
        assert_eq!(cvt_u8_buf(&[0u8, 1, 2]), &[0 as libc::c_char, 1, 2]);
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

    #[cfg(feature = "alloc")]
    #[test]
    fn test_debug_list_field() {
        assert_eq!(format!("{:?}", DebugListField([0; 0].iter())), "[]");

        assert_eq!(format!("{:?}", DebugListField([0].iter())), "[0]");

        assert_eq!(
            format!("{:?}", DebugListField([0, 1, 2].iter())),
            "[0, 1, 2]"
        );
    }

    #[test]
    fn test_int_parse_bytes() {
        assert_eq!(i32::parse_bytes(b"", true), Err(IntParseBytesError::Empty));
        assert_eq!(i32::parse_bytes(b"", false), Err(IntParseBytesError::Empty));

        assert_eq!(
            i32::parse_bytes(b"d", true),
            Err(IntParseBytesError::InvalidDigit)
        );
        assert_eq!(
            i32::parse_bytes(b"-d", true),
            Err(IntParseBytesError::InvalidDigit)
        );
        assert_eq!(
            i32::parse_bytes(b"+d", true),
            Err(IntParseBytesError::InvalidDigit)
        );

        assert_eq!(
            u8::parse_bytes(b"256", true),
            Err(IntParseBytesError::Overflow)
        );
        assert_eq!(
            i8::parse_bytes(b"128", true),
            Err(IntParseBytesError::Overflow)
        );
        assert_eq!(
            u8::parse_bytes(b"-1", true),
            Err(IntParseBytesError::Overflow)
        );
        assert_eq!(
            i8::parse_bytes(b"-129", true),
            Err(IntParseBytesError::Overflow)
        );

        assert_eq!(u8::parse_bytes(b"0", true), Ok(0));
        assert_eq!(u8::parse_bytes(b"+0", true), Ok(0));
        assert_eq!(u8::parse_bytes(b"-0", true), Ok(0));
        assert_eq!(u8::parse_bytes(b"0", false), Ok(0));

        assert_eq!(i8::parse_bytes(b"-1", true), Ok(-1));
        assert_eq!(i8::parse_bytes(b"-127", true), Ok(-127));

        assert_eq!(u8::parse_bytes(b"255", false), Ok(255));
        assert_eq!(u8::parse_bytes(b"254", false), Ok(254));

        assert_eq!(
            u8::parse_bytes(b"+0", false),
            Err(IntParseBytesError::InvalidDigit)
        );
        assert_eq!(
            u8::parse_bytes(b"-0", false),
            Err(IntParseBytesError::InvalidDigit)
        );
    }

    #[test]
    fn test_int_parse_bytes_radix() {
        assert_eq!(u8::parse_bytes_radix(b"a1", 16, true), Ok(161));
        assert_eq!(u16::parse_bytes_radix(b"z0", 36, true), Ok(35 * 36));
        assert_eq!(u16::parse_bytes_radix(b"Z0", 36, true), Ok(35 * 36));
        assert_eq!(u16::parse_bytes_radix(b"a0", 36, true), Ok(360));
        assert_eq!(u16::parse_bytes_radix(b"A0", 36, true), Ok(360));

        assert_eq!(u8::parse_bytes_radix(b"101", 2, true), Ok(5));
        assert_eq!(i8::parse_bytes_radix(b"-101", 2, true), Ok(-5));
        assert_eq!(
            u8::parse_bytes_radix(b"-101", 2, false),
            Err(IntParseBytesError::InvalidDigit)
        );

        assert_eq!(
            u8::parse_bytes_radix(b"z0", 16, true),
            Err(IntParseBytesError::InvalidDigit)
        );
        assert_eq!(
            u8::parse_bytes_radix(b"Z0", 16, true),
            Err(IntParseBytesError::InvalidDigit)
        );
        assert_eq!(
            u8::parse_bytes_radix(b"2", 2, true),
            Err(IntParseBytesError::InvalidDigit)
        );
    }

    #[test]
    fn test_dlsym() {
        static NOEXIST: DlFuncLoader<unsafe extern "C" fn()> =
            unsafe { DlFuncLoader::new(b"NO_SYMBOL_WITH_THIS_NAME_EXISTS\0") };
        static GETUID: DlFuncLoader<unsafe extern "C" fn() -> libc::uid_t> =
            unsafe { DlFuncLoader::new(b"getuid\0") };

        assert_eq!(NOEXIST.get(), None);
        assert_eq!(NOEXIST.get(), None);

        if cfg!(target_feature = "crt-static") {
            assert_eq!(GETUID.get(), None);
            assert_eq!(GETUID.get(), None);
        } else {
            assert_eq!(GETUID.get().unwrap() as usize, libc::getuid as usize);
            assert_eq!(GETUID.get().unwrap() as usize, libc::getuid as usize);
        }
    }
}
