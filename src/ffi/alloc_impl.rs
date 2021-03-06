use super::core_impl::{CStr, OsStr};

use core::borrow::Borrow;
use core::cmp::{Ordering, PartialEq, PartialOrd};
use core::fmt;
use core::ops::{Deref, DerefMut, Index, IndexMut, RangeFull};

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;

use crate::internal_prelude::*;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NulError {
    index: usize,
    vec: Vec<u8>,
}

impl NulError {
    #[inline]
    pub fn nul_position(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.vec
    }
}

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nul byte found at position: {}", self.index)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntoStringError(CString, core::str::Utf8Error);

impl IntoStringError {
    #[inline]
    pub fn into_cstring(self) -> CString {
        self.0
    }

    #[inline]
    pub fn utf8_error(&self) -> core::str::Utf8Error {
        self.1
    }
}

impl fmt::Display for IntoStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("C string contained non-utf8 bytes")
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CString(Vec<u8>);

impl CString {
    #[inline]
    pub fn new<T: Into<Vec<u8>>>(t: T) -> core::result::Result<Self, NulError> {
        Self::from_vec(t.into())
    }

    fn from_vec(vec: Vec<u8>) -> core::result::Result<Self, NulError> {
        if let Some(index) = vec.iter().position(|&c| c == 0) {
            Err(NulError { index, vec })
        } else {
            Ok(unsafe { Self::from_vec_unchecked(vec) })
        }
    }

    #[inline]
    pub unsafe fn from_vec_unchecked(mut v: Vec<u8>) -> Self {
        v.push(0);
        Self(v)
    }

    #[inline]
    pub unsafe fn from_vec_with_nul_unchecked(v: Vec<u8>) -> Self {
        Self(v)
    }

    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }

    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.0) }
    }

    #[inline]
    pub fn into_bytes(mut self) -> Vec<u8> {
        let last = self.0.pop();
        debug_assert_eq!(last, Some(0));
        self.0
    }

    #[inline]
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.0
    }

    #[inline]
    pub fn into_boxed_c_str(self) -> Box<CStr> {
        unsafe { core::mem::transmute::<Box<[u8]>, Box<CStr>>(self.0.into()) }
    }

    pub fn into_string(self) -> core::result::Result<String, IntoStringError> {
        if let Err(e) = core::str::from_utf8(self.as_bytes()) {
            Err(IntoStringError(self, e))
        } else {
            Ok(unsafe { String::from_utf8_unchecked(self.into_bytes()) })
        }
    }

    pub fn into_raw(self) -> *mut libc::c_char {
        unsafe { (&*Box::into_raw(Box::<[u8]>::from(self.0))).as_ptr() as *mut _ }
    }

    pub unsafe fn from_raw(ptr: *mut libc::c_char) -> Self {
        Self(
            Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                ptr as *mut u8,
                libc::strlen(ptr) + 1,
            ))
            .into(),
        )
    }
}

impl Default for CString {
    #[inline]
    fn default() -> Self {
        Self(vec![0])
    }
}

impl Deref for CString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        self.as_c_str()
    }
}

impl AsRef<CStr> for CString {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}

impl Borrow<CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_c_str()
    }
}

impl Index<RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _: RangeFull) -> &CStr {
        self.as_c_str()
    }
}

impl From<&'_ CStr> for CString {
    #[inline]
    fn from(s: &CStr) -> CString {
        s.to_owned()
    }
}

impl From<Box<CStr>> for CString {
    #[inline]
    fn from(s: Box<CStr>) -> CString {
        CStr::into_c_string(s)
    }
}

impl<'a> From<&'a CString> for Cow<'a, CStr> {
    #[inline]
    fn from(s: &'a CString) -> Cow<'a, CStr> {
        Cow::Borrowed(s.as_c_str())
    }
}

impl From<CString> for Vec<u8> {
    #[inline]
    fn from(s: CString) -> Vec<u8> {
        s.into_bytes()
    }
}

impl From<CString> for Box<CStr> {
    #[inline]
    fn from(s: CString) -> Box<CStr> {
        s.into_boxed_c_str()
    }
}

impl<'a> From<CString> for Cow<'a, CStr> {
    #[inline]
    fn from(s: CString) -> Cow<'a, CStr> {
        Cow::Owned(s)
    }
}

impl<'a> From<Cow<'a, CStr>> for CString {
    #[inline]
    fn from(s: Cow<'a, CStr>) -> CString {
        s.into_owned()
    }
}

impl fmt::Debug for CString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OsString(Vec<u8>);

impl OsString {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        OsStr::from_bytes(&self.0)
    }

    #[inline]
    fn as_os_str_mut(&mut self) -> &mut OsStr {
        OsStr::from_bytes_mut(&mut self.0)
    }

    #[inline]
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    pub fn push<T: AsRef<OsStr>>(&mut self, s: T) {
        self.0.extend_from_slice(s.as_ref().as_bytes())
    }

    #[inline]
    pub fn into_boxed_os_str(self) -> Box<OsStr> {
        unsafe { core::mem::transmute::<Box<[u8]>, Box<OsStr>>(self.0.into()) }
    }

    #[inline]
    pub fn into_string(self) -> core::result::Result<String, OsString> {
        match String::from_utf8(self.0) {
            Ok(s) => Ok(s),
            Err(e) => Err(Self(e.into_bytes())),
        }
    }
}

impl<T: ?Sized + AsRef<OsStr>> From<&'_ T> for OsString {
    #[inline]
    fn from(s: &T) -> Self {
        s.as_ref().to_owned()
    }
}

impl From<String> for OsString {
    #[inline]
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<Cow<'_, OsStr>> for OsString {
    #[inline]
    fn from(s: Cow<'_, OsStr>) -> Self {
        s.into_owned()
    }
}

impl From<Box<OsStr>> for OsString {
    #[inline]
    fn from(s: Box<OsStr>) -> Self {
        OsStr::into_os_string(s)
    }
}

impl From<OsString> for Box<OsStr> {
    #[inline]
    fn from(s: OsString) -> Self {
        s.into_boxed_os_str()
    }
}

impl<'a> From<&'a OsString> for Cow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsString) -> Self {
        Cow::Borrowed(s.as_os_str())
    }
}

impl From<OsString> for Cow<'_, OsStr> {
    #[inline]
    fn from(s: OsString) -> Self {
        Cow::Owned(s)
    }
}

impl core::str::FromStr for OsString {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(String::from(s).into())
    }
}

impl Deref for OsString {
    type Target = OsStr;

    #[inline]
    fn deref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl DerefMut for OsString {
    #[inline]
    fn deref_mut(&mut self) -> &mut OsStr {
        self.as_os_str_mut()
    }
}

impl AsRef<OsStr> for OsString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl Borrow<OsStr> for OsString {
    #[inline]
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl Index<RangeFull> for OsString {
    type Output = OsStr;

    #[inline]
    fn index(&self, _: RangeFull) -> &OsStr {
        self.as_os_str()
    }
}

impl IndexMut<RangeFull> for OsString {
    #[inline]
    fn index_mut(&mut self, _: RangeFull) -> &mut OsStr {
        self.as_os_str_mut()
    }
}

impl fmt::Debug for OsString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

macro_rules! osstring_partial_ordeq {
    ($($type:ty)*) => {
        $(
            impl PartialOrd<OsString> for $type {
                #[inline]
                fn partial_cmp(&self, other: &OsString) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<OsString> for $type {
                #[inline]
                fn eq(&self, other: &OsString) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }

            impl PartialOrd<OsString> for &$type {
                #[inline]
                fn partial_cmp(&self, other: &OsString) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<OsString> for &$type {
                #[inline]
                fn eq(&self, other: &OsString) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }

            impl PartialOrd<$type> for OsString {
                #[inline]
                fn partial_cmp(&self, other: &$type) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<$type> for OsString {
                #[inline]
                fn eq(&self, other: &$type) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }

            impl PartialOrd<&$type> for OsString {
                #[inline]
                fn partial_cmp(&self, other: &&$type) -> Option<Ordering> {
                    Some(self.as_bytes().cmp(other.as_bytes()))
                }
            }

            impl PartialEq<&$type> for OsString {
                #[inline]
                fn eq(&self, other: &&$type) -> bool {
                    self.as_bytes().eq(other.as_bytes())
                }
            }
        )*
    };
}

osstring_partial_ordeq! { str }

#[cfg(feature = "alloc")]
osstring_partial_ordeq! { Cow<'_, OsStr> }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstring() {
        let abc = CString::new(*b"abc").unwrap();
        let empty = CString::new([]).unwrap();

        let abc_cstr = CStr::from_bytes_with_nul(b"abc\0").unwrap();
        let empty_cstr: &CStr = Default::default();

        assert_eq!(abc.as_ref(), abc_cstr);
        assert_eq!(abc.borrow() as &CStr, abc_cstr);
        assert_eq!(&abc[..], abc_cstr);
        assert_eq!(empty.as_ref(), empty_cstr);
        assert_eq!(empty.borrow() as &CStr, empty_cstr);
        assert_eq!(&empty[..], empty_cstr);

        assert_eq!(empty, CString::default());

        assert_eq!(abc.as_bytes(), b"abc");
        assert_eq!(empty.as_bytes(), b"");
        assert_eq!(abc.as_bytes_with_nul(), b"abc\0");
        assert_eq!(empty.as_bytes_with_nul(), b"\0");

        assert_eq!(abc.clone().into_string().unwrap(), "abc");
        assert_eq!(empty.clone().into_string().unwrap(), "");

        assert_eq!(abc.clone().into_bytes(), b"abc");
        assert_eq!(empty.clone().into_bytes(), b"");
        assert_eq!(abc.clone().into_bytes_with_nul(), b"abc\0");
        assert_eq!(empty.clone().into_bytes_with_nul(), b"\0");

        assert_eq!(CString::from(abc_cstr), abc);
        assert_eq!(CString::from(Cow::Borrowed(abc_cstr)), abc);
        assert_eq!(Vec::from(abc), b"abc");
    }

    #[test]
    fn test_cstring_raw() {
        unsafe {
            let ptr = CString::new(*b"abc").unwrap().into_raw();
            assert_eq!(libc::strlen(ptr), 3);
            assert_eq!(CString::from_raw(ptr).as_bytes_with_nul(), b"abc\0");
        }
    }

    #[test]
    fn test_cstring_error() {
        let err = CString::new(*b"\0").unwrap_err();
        assert_eq!(err.nul_position(), 0);
        assert_eq!(err.to_string(), "nul byte found at position: 0");
        assert_eq!(err.into_vec(), b"\0");

        let err = CString::new(*b"abc\0def").unwrap_err();
        assert_eq!(err.nul_position(), 3);
        assert_eq!(err.to_string(), "nul byte found at position: 3");
        assert_eq!(err.into_vec(), b"abc\0def");
    }

    #[test]
    fn test_osstring() {
        use core::str::FromStr;

        let abc = OsString::from("abc");
        let empty = OsString::new();
        let empty_cap = OsString::with_capacity(10);

        assert_eq!(abc.len(), 3);
        assert_eq!(empty.len(), 0);
        assert_eq!(empty_cap.len(), 0);
        assert_eq!(abc.capacity(), 3);
        assert_eq!(empty.capacity(), 0);
        assert_eq!(empty_cap.capacity(), 10);

        assert_eq!(abc, OsStr::new("abc"));
        assert_eq!(empty, OsStr::new(""));
        assert_eq!(empty_cap, OsStr::new(""));

        assert_eq!(empty, empty_cap);
        assert_eq!(OsStr::new(""), empty);
        assert_eq!(empty, "");
        assert_eq!("", empty);

        assert_eq!(abc.borrow() as &OsStr, OsStr::new("abc"));
        assert_eq!(empty.borrow() as &OsStr, OsStr::new(""));
        assert_eq!(&abc[..], OsStr::new("abc"));
        assert_eq!(&empty[..], OsStr::new(""));

        assert_eq!(abc.clone().into_vec(), b"abc");
        assert_eq!(empty.clone().into_vec(), b"");
        assert_eq!(empty_cap.clone().into_vec(), b"");

        assert_eq!(OsString::from_vec(b"abc".to_vec()), abc);
        assert_eq!(OsString::from_vec(b"".to_vec()), empty);

        assert_eq!(abc.clone().into_string().unwrap(), "abc");
        assert_eq!(empty.clone().into_string().unwrap(), "");

        assert_eq!(OsString::from_str("abc").unwrap(), abc);
        assert_eq!(OsString::from_str("").unwrap(), empty);
    }

    #[test]
    fn test_osstring_mut() {
        let mut s = OsString::new();

        s.push("abc");
        assert_eq!(s, "abc");

        s.reserve(10);
        assert!(s.capacity() >= 13);
        s.reserve_exact(10);
        assert!(s.capacity() >= 13 && s.capacity() <= 16);
        s.shrink_to_fit();
        assert!(s.capacity() <= 6);

        s.clear();
        assert_eq!(s, "");
    }
}
