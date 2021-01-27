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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
