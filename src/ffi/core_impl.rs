#[cfg(feature = "alloc")]
use crate::internal_prelude::*;

use core::cmp::{Ordering, PartialEq, PartialOrd};
use core::fmt;
use core::ops::{Index, RangeFrom};

#[cfg(feature = "alloc")]
use super::alloc_impl::{CString, OsString};

#[derive(Clone, Eq, PartialEq)]
pub struct FromBytesWithNulError {
    is_mid: bool,
}

impl FromBytesWithNulError {
    fn strerror(&self) -> &str {
        if self.is_mid {
            "unexpected nul byte in middle of string"
        } else {
            "expected nul byte at end of string"
        }
    }
}

impl fmt::Debug for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromBytesWithNulError")
            .field("message", &self.strerror())
            .finish()
    }
}

impl fmt::Display for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.strerror())
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct CStr([u8]);

impl CStr {
    #[inline]
    pub unsafe fn from_ptr<'a>(ptr: *const libc::c_char) -> &'a Self {
        Self::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
            ptr as *const _,
            libc::strlen(ptr) + 1,
        ))
    }

    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        core::mem::transmute(bytes)
    }

    pub fn from_bytes_with_nul(bytes: &[u8]) -> core::result::Result<&Self, FromBytesWithNulError> {
        match bytes.split_last() {
            Some((0, rest)) => {
                if rest.contains(&0) {
                    Err(FromBytesWithNulError { is_mid: true })
                } else {
                    Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
                }
            }

            _ => Err(FromBytesWithNulError { is_mid: false }),
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const libc::c_char {
        self.0.as_ptr() as *const _
    }

    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }

    #[inline]
    pub fn to_str(&self) -> core::result::Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.to_bytes())
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.to_bytes())
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    pub fn into_c_string(self: Box<CStr>) -> CString {
        let vec = unsafe { core::mem::transmute::<_, Box<[u8]>>(self) }.into();
        unsafe { CString::from_vec_with_nul_unchecked(vec) }
    }
}

impl AsRef<CStr> for CStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl Index<RangeFrom<usize>> for CStr {
    type Output = CStr;

    fn index(&self, r: RangeFrom<usize>) -> &CStr {
        // Perform bound checks
        let _ = self.to_bytes()[r.start..];

        unsafe {
            CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
                self.0.as_ptr().add(r.start),
                self.0.len() - r.start,
            ))
        }
    }
}

impl Default for &'_ CStr {
    #[inline]
    fn default() -> Self {
        static BYTES: [u8; 1] = [0];
        unsafe { CStr::from_bytes_with_nul_unchecked(&BYTES) }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl From<&'_ CStr> for Box<CStr> {
    #[inline]
    fn from(s: &CStr) -> Box<CStr> {
        s.to_owned().into_boxed_c_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl<'a> From<&'a CStr> for Cow<'a, CStr> {
    #[inline]
    fn from(s: &'a CStr) -> Cow<'a, CStr> {
        Cow::Borrowed(s)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl ToOwned for CStr {
    type Owned = CString;

    #[inline]
    fn to_owned(&self) -> CString {
        unsafe { CString::from_vec_with_nul_unchecked(self.to_bytes_with_nul().to_vec()) }
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct OsStr([u8]);

impl OsStr {
    #[inline]
    pub fn new<S: AsRef<Self> + ?Sized>(s: &S) -> &Self {
        s.as_ref()
    }

    #[inline]
    pub fn from_bytes(slice: &[u8]) -> &Self {
        unsafe { core::mem::transmute(slice) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::mem::transmute(self) }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn from_bytes_mut(slice: &mut [u8]) -> &mut Self {
        unsafe { core::mem::transmute(slice) }
    }

    #[inline]
    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::mem::transmute(self) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.as_bytes().is_ascii()
    }

    #[inline]
    pub fn eq_ignore_ascii_case<S: ?Sized + AsRef<OsStr>>(&self, other: &S) -> bool {
        self.as_bytes()
            .eq_ignore_ascii_case(other.as_ref().as_bytes())
    }

    #[inline]
    pub fn make_ascii_lowercase(&mut self) {
        self.as_bytes_mut().make_ascii_lowercase()
    }

    #[inline]
    pub fn make_ascii_uppercase(&mut self) {
        self.as_bytes_mut().make_ascii_uppercase()
    }

    #[inline]
    pub fn to_str(&self) -> Option<&str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_os_string(&self) -> OsString {
        self.to_owned()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    pub fn to_ascii_lowercase(&self) -> OsString {
        let mut s = self.to_owned();
        s.make_ascii_lowercase();
        s
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    pub fn to_ascii_uppercase(&self) -> OsString {
        let mut s = self.to_owned();
        s.make_ascii_uppercase();
        s
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    pub fn into_os_string(self: Box<OsStr>) -> OsString {
        OsString::from_vec(unsafe { core::mem::transmute::<_, Box<[u8]>>(self) }.to_vec())
    }
}

impl Default for &'_ OsStr {
    #[inline]
    fn default() -> Self {
        OsStr::from_bytes(&[])
    }
}

impl AsRef<OsStr> for OsStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl AsRef<OsStr> for str {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        OsStr::from_bytes(self.as_bytes())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl AsRef<OsStr> for String {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        OsStr::from_bytes(self.as_bytes())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl From<&'_ OsStr> for Box<OsStr> {
    #[inline]
    fn from(s: &OsStr) -> Box<OsStr> {
        s.to_os_string().into_boxed_os_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl<'a> From<&'a OsStr> for Cow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsStr) -> Cow<'a, OsStr> {
        Cow::Borrowed(s)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl ToOwned for OsStr {
    type Owned = OsString;

    #[inline]
    fn to_owned(&self) -> OsString {
        OsString::from_vec(self.as_bytes().to_vec())
    }
}

macro_rules! osstr_partial_ordeq {
    ($($type:ty)*) => {
        $(
            impl PartialOrd<OsStr> for $type {
                #[inline]
                fn partial_cmp(&self, other: &OsStr) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<OsStr> for $type {
                #[inline]
                fn eq(&self, other: &OsStr) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }

            impl PartialOrd<$type> for OsStr {
                #[inline]
                fn partial_cmp(&self, other: &$type) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<$type> for OsStr {
                #[inline]
                fn eq(&self, other: &$type) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }

            impl PartialOrd<&$type> for OsStr {
                #[inline]
                fn partial_cmp(&self, other: &&$type) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<&$type> for OsStr {
                #[inline]
                fn eq(&self, other: &&$type) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }
        )*
    };
}

osstr_partial_ordeq! { str }

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
osstr_partial_ordeq! { Cow<'_, OsStr> OsString }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_error() {
        let err = CStr::from_bytes_with_nul(b"").unwrap_err();
        assert!(!err.is_mid);
        #[cfg(feature = "alloc")]
        assert_eq!(err.to_string(), "expected nul byte at end of string");

        let err = CStr::from_bytes_with_nul(b"a").unwrap_err();
        assert!(!err.is_mid);
        #[cfg(feature = "alloc")]
        assert_eq!(err.to_string(), "expected nul byte at end of string");

        let err = CStr::from_bytes_with_nul(b"\0a").unwrap_err();
        assert!(!err.is_mid);
        #[cfg(feature = "alloc")]
        assert_eq!(err.to_string(), "expected nul byte at end of string");

        let err = CStr::from_bytes_with_nul(b"\0a\0").unwrap_err();
        assert!(err.is_mid);
        #[cfg(feature = "alloc")]
        assert_eq!(err.to_string(), "unexpected nul byte in middle of string");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_cstr_alloc() {
        let abc = CStr::from_bytes_with_nul(b"abc\0").unwrap();

        assert_eq!(abc.to_str().unwrap(), "abc");
        assert_eq!(abc.to_string_lossy(), "abc");

        assert_eq!(abc.to_owned().into_bytes_with_nul(), b"abc\0");
    }

    #[test]
    fn test_cstr() {
        let abc = CStr::from_bytes_with_nul(b"abc\0").unwrap();
        let bc = CStr::from_bytes_with_nul(b"bc\0").unwrap();
        let empty = CStr::from_bytes_with_nul(b"\0").unwrap();

        assert_eq!(empty, <&CStr as Default>::default());

        assert_eq!(&abc[0..], abc);
        assert_eq!(&abc[1..], bc);
        assert_eq!(&abc[3..], empty);
        assert_eq!(&bc[2..], empty);
        assert_eq!(&empty[0..], empty);

        assert_eq!(abc.to_bytes(), b"abc");
        assert_eq!(abc.to_bytes_with_nul(), b"abc\0");

        assert_eq!(bc.to_bytes(), b"bc");
        assert_eq!(bc.to_bytes_with_nul(), b"bc\0");

        assert_eq!(empty.to_bytes(), b"");
        assert_eq!(empty.to_bytes_with_nul(), b"\0");
    }
}
