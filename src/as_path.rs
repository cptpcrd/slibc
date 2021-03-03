#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

use crate::internal_prelude::*;

/// Represents a string that can be cheaply re-cast as a `OsStr`, and possibly also as a `CStr`.
///
/// The design of this was inspired by `openat`'s `AsPath` trait and `nix`'s `NixPath` trait. It's
/// essentially a combination of `AsRef<OsStr>` and `NixPath`.
pub trait AsPath {
    /// Convert this string to a `OsStr`.
    ///
    /// This serves a similar purpose to `AsRef<OsStr>::as_ref()`, so many of the `AsRef` rules apply
    /// (i.e. it should be very inexpensive and never fail).
    fn as_os_str(&self) -> &OsStr;

    /// Calls the given closure with a version of `self` converted to a `CStr`.
    ///
    /// The `CStr` may actually be a `CString` (allocated from the heap), or it may be the original
    /// string if that string is already nul-terminated.
    ///
    /// IMPORTANT: If the string contains an interior nul byte that prevents it from being converted
    /// to a `CString`, the closure will not be called, and an error will be returned.
    fn with_cstr<T, F: FnMut(&CStr) -> Result<T>>(&self, f: F) -> Result<T>;
}

#[cfg(feature = "alloc")]
macro_rules! osstr_ref_impl {
    ($($type:ty)*) => {
        $(
            impl AsPath for $type {
                #[inline]
                fn as_os_str(&self) -> &OsStr {
                    self.as_ref()
                }

                fn with_cstr<T, F: FnMut(&CStr) -> Result<T>>(&self, mut f: F) -> Result<T> {
                    if let Ok(s) = CString::new(self.as_os_str().as_bytes()) {
                        f(&s)
                    } else {
                        Err(Error::mid_nul())
                    }
                }
            }
        )*
    };
}

macro_rules! cstr_impl {
    ($($type:ty)*) => {
        $(
            impl AsPath for $type {
                #[inline]
                fn as_os_str(&self) -> &OsStr {
                    OsStr::from_bytes(self.to_bytes())
                }

                #[inline]
                fn with_cstr<T, F: FnMut(&CStr) -> Result<T>>(&self, mut f: F) -> Result<T> {
                    f(self)
                }
            }
        )*
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        osstr_ref_impl! { &Path PathBuf &PathBuf &OsStr OsString &OsString &str String &String }
        cstr_impl! { &CStr CString &CString }
    } else if #[cfg(feature =  "alloc")] {
        use alloc::string::String;
        osstr_ref_impl! { &OsStr OsString &OsString &str String &String }
        cstr_impl! { &CStr CString &CString }
    } else {
        cstr_impl! { &CStr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_os_str() {
        assert_eq!(
            CStr::from_bytes_with_nul(b"abc/def\0").unwrap().as_os_str(),
            OsStr::new("abc/def")
        );

        #[cfg(feature = "alloc")]
        {
            assert_eq!("abc/def".as_os_str(), OsStr::new("abc/def"));
            assert_eq!(String::from("abc/def").as_os_str(), OsStr::new("abc/def"));

            assert_eq!(OsStr::new("abc/def").as_os_str(), OsStr::new("abc/def"));
            assert_eq!(OsString::from("abc/def").as_os_str(), OsStr::new("abc/def"));

            assert_eq!(
                CString::new(b"abc/def".as_ref()).unwrap().as_os_str(),
                OsStr::new("abc/def")
            );
        }

        #[cfg(feature = "std")]
        {
            assert_eq!(Path::new("abc/def").as_os_str(), OsStr::new("abc/def"));
            assert_eq!(PathBuf::from("abc/def").as_os_str(), OsStr::new("abc/def"));
        }
    }

    #[test]
    fn test_with_cstr() {
        fn do_it<P: AsPath>(p: P) {
            let expected = CStr::from_bytes_with_nul(b"abc/def\0").unwrap();

            p.with_cstr(|s| {
                assert_eq!(s, expected);
                Ok(())
            })
            .unwrap();
        }

        #[cfg(feature = "alloc")]
        {
            do_it("abc/def");
            do_it(String::from("abc/def"));
            do_it(&String::from("abc/def"));

            do_it(OsStr::new("abc/def"));
            do_it(OsString::from("abc/def"));
            do_it(&OsString::from("abc/def"));

            do_it(CString::new("abc/def").unwrap());
            do_it(&CString::new("abc/def").unwrap());
        }

        #[cfg(feature = "std")]
        {
            do_it(Path::new("abc/def"));
            do_it(PathBuf::from("abc/def"));
            do_it(&PathBuf::from("abc/def"));
        }

        do_it(CStr::from_bytes_with_nul(b"abc/def\0").unwrap());
    }

    #[test]
    fn test_as_path_arg() {
        fn do_it<P: AsPath>(p: P) {
            p.as_os_str();
        }

        do_it(CStr::from_bytes_with_nul(b"\0").unwrap());

        #[cfg(feature = "alloc")]
        {
            do_it("");

            do_it(OsStr::new(""));

            do_it(String::new());
            do_it(&String::new());

            do_it(OsString::from(""));
            do_it(&OsString::from(""));

            do_it(CString::new("").unwrap());
            do_it(&CString::new("").unwrap());
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_mid_nul() {
        fn check<P: AsPath>(path: P) {
            assert_eq!(path.with_cstr(|_| Ok(())).unwrap_err(), Errno::EINVAL);
        }

        check("\0");
        check(String::from("\0"));
        check(OsStr::new("\0"));
        check(OsString::from("\0"));
    }
}
