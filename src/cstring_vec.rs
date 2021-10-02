use crate::internal_prelude::*;

use core::fmt;
use core::ops::Deref;

/// A NULL-terminated `Vec` of pointers to `CString`s.
///
/// This is intended for use in constructing the `argv` and `envp` arguments to e.g. `execve()` or
/// `posix_spawn()`.
///
/// This struct wraps a `Vec<*mut libc::c_char>`. It is intended for use in constructing the `argv`
/// and `envp` arguments to e.g. `execve(2)` or `posix_spawn(3)`.
///
/// # Invariants
///
/// - The wrapped `Vec` can never be empty.
/// - The last element of the wrapped `Vec` is always NULL.
/// - Every non-NULL element of the `Vec` is a pointer leaked from [`CString::into_raw()`].
///
/// The `CStringVec` will also try to ensure that no element of the wrapped `Vec` except the last
/// element is NULL. However, this will not be true if:
///
/// - A panic occurs at very specific points within `CStringVec`'s  implementation.
/// - A `Vec` which does not meet this criterion is passed to [`CStringVec::from_vec()`].
///
/// # Getting a reference as a slice
///
/// `CStringVec` implements `AsRef<[*mut libc::c_char]>`, `AsRef<[*const libc::c_char]>`, and
/// `Deref<Target=[*mut libc::c_char]>`. All of these return a slice representing a view of the
/// entire wrapped `Vec`, including the terminating NULL.
///
/// # Cloning
///
/// Cloning a `CStringVec` will clone all of the contained `CString`s, while leaving all of the
/// NULL pointers intact.
///
/// # Dropping
///
/// If a `CStringVec` is dropped, the wrapped `Vec` AND all of the `CString`s it contains are
/// freed.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct CStringVec(Vec<*mut libc::c_char>);

impl CStringVec {
    /// Create a new `CStringVec` containing one NULL.
    #[inline]
    pub fn new() -> Self {
        Self(vec![core::ptr::null_mut()])
    }

    /// Create a new `CStringVec` containing one NULL with enough capacity to hold `cap` elements
    /// total (including the trailing NULL).
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        let mut v = Vec::with_capacity(cap);
        v.push(core::ptr::null_mut());
        Self(v)
    }

    /// Replace the element at the specified index `i` with the new string `new`.
    #[inline]
    pub fn replace(&mut self, i: usize, new: CString) {
        assert!(
            i < self.0.len() - 1,
            "index {} out of bounds for CStringVec of length {} (the trailing NULL cannot be overwritten)",
            i,
            self.0.len(),
        );

        let ptr = core::mem::replace(&mut self.0[i], new.into_raw());
        if !ptr.is_null() {
            unsafe {
                CString::from_raw(ptr);
            }
        }
    }

    /// Insert a `CString` at the given index `i`.
    pub fn insert(&mut self, mut i: usize, new: CString) {
        assert!(
            i < self.0.len(),
            "index {} out of bounds for CStringVec of length {} (cannot insert after the trailing NULL)",
            i,
            self.0.len(),
        );

        // Add another NULL, then copy everything forward

        self.0.push(core::ptr::null_mut());

        // We avoid using e.g. copy_within() so we can ensure that we never have duplicate
        // (non-NULL) pointers in the case of a panic.
        let mut last = new.into_raw();
        while i < self.0.len() - 1 {
            core::mem::swap(&mut last, &mut self.0[i]);
            i += 1;
        }
        debug_assert!(last.is_null());
    }

    /// Remove the `CString` at the given index `i`.
    ///
    /// If there was a `CString` at the given index, it is returned. However, if there was a NULL
    /// pointer at the given index (see [invariants](#invariants)), `None` is returned. In either
    /// case the inner `Vec` is now one element shorter.
    pub fn remove(&mut self, i: usize) -> Option<CString> {
        assert!(
            i < self.0.len() - 1,
            "index {} out of bounds for CStringVec of length {} (cannot remove the trailing NULL)",
            i,
            self.0.len(),
        );

        // Again, we avoid using e.g. copy_within() so we can ensure that we never have duplicate
        // (non-NULL) pointers in the case of a panic.
        let mut ptr = core::ptr::null_mut();
        for j in (i..self.0.len()).rev() {
            core::mem::swap(&mut ptr, &mut self.0[j]);
        }
        self.0.truncate(self.0.len() - 1);

        if !ptr.is_null() {
            Some(unsafe { CString::from_raw(ptr) })
        } else {
            None
        }
    }

    /// Reserve space for `n` more `CString`s to be added to the end of the `Vec`.
    #[inline]
    pub fn reserve(&mut self, n: usize) {
        self.0.reserve(n);
    }

    /// The number of elements (including the trailing NULL) that this `CStringVec` can hold
    /// without resizing.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Push a new `CString` to the end of the `Vec`.
    pub fn push(&mut self, cstr: CString) {
        self.0.push(core::ptr::null_mut());

        let len = self.0.len();
        self.0[len - 2] = cstr.into_raw();
    }

    /// Get a `CStr` referring to the `CString` at the given index `i`.
    ///
    /// If there is a NULL pointer at the given index (see [invariants](#invariants)), `None` is
    /// returned.
    pub fn get_cstr(&self, i: usize) -> Option<&CStr> {
        assert!(
            i < self.0.len() - 1,
            "index {} out of bounds for CStringVec of length {} (cannot remove the trailing NULL)",
            i,
            self.0.len(),
        );

        let ptr = self.0[i];
        if !ptr.is_null() {
            Some(unsafe { CStr::from_ptr(ptr) })
        } else {
            None
        }
    }

    /// Get a raw pointer to the start of the `Vec`.
    ///
    /// This is suitable for passing as `argv` or `envp` to e.g. `execve()`.
    #[inline]
    pub fn as_ptr(&self) -> *const *const libc::c_char {
        self.0.as_ptr() as *const *const _
    }

    /// Consume the `CStringVec` and return the `Vec` it was wrapping.
    ///
    /// This can be used in combination with [`Self::from_vec()`] to perform operations that are
    /// normally impossible on a `CStringVec` (mainly for safety reasons). However, note that the
    /// `CString`s whose pointers are stored in the returned `Vec` will NOT be freed when the `Vec`
    /// is dropped.
    #[inline]
    pub fn into_vec(self) -> Vec<*mut libc::c_char> {
        let csvec = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(&csvec.0) }
    }

    /// Create a new `CStringVec` wrapping the given `Vec`.
    ///
    /// # Safety
    ///
    /// The given `Vec` must obey the [invariants](#invariants) listed above.
    #[inline]
    pub unsafe fn from_vec(vec: Vec<*mut libc::c_char>) -> Self {
        Self(vec)
    }
}

impl Default for CStringVec {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CStringVec {
    fn clone(&self) -> Self {
        Self(
            self.iter()
                .map(|&ptr| {
                    if ptr.is_null() {
                        core::ptr::null_mut()
                    } else {
                        unsafe { CStr::from_ptr(ptr) }.to_owned().into_raw()
                    }
                })
                .collect(),
        )
    }
}

impl Drop for CStringVec {
    fn drop(&mut self) {
        for &ptr in &self.0 {
            if !ptr.is_null() {
                unsafe {
                    CString::from_raw(ptr);
                }
            }
        }
    }
}

impl AsRef<[*mut libc::c_char]> for CStringVec {
    #[inline]
    fn as_ref(&self) -> &[*mut libc::c_char] {
        self
    }
}

impl AsRef<[*const libc::c_char]> for CStringVec {
    #[inline]
    fn as_ref(&self) -> &[*const libc::c_char] {
        unsafe { core::slice::from_raw_parts(self.0.as_ptr() as *const *const _, self.0.len()) }
    }
}

impl Deref for CStringVec {
    type Target = [*mut libc::c_char];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for CStringVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|&ptr| {
                if ptr.is_null() {
                    None
                } else {
                    Some(unsafe { CStr::from_ptr(ptr) })
                }
            }))
            .finish()
    }
}

impl core::iter::FromIterator<CString> for CStringVec {
    fn from_iter<I: IntoIterator<Item = CString>>(it: I) -> Self {
        let mut it = it.into_iter();

        if let Some(first) = it.next() {
            let mut vec = Vec::with_capacity(it.size_hint().0 + 2);
            vec.push(first.into_raw());
            vec.extend(it.map(|s| s.into_raw()));
            vec.push(core::ptr::null_mut());
            Self(vec)
        } else {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_cstringvec<S: AsRef<OsStr> + PartialEq<OsStr> + fmt::Debug + ?Sized>(
        csvec: CStringVec,
        strs: &[&S],
    ) {
        fn check_inner<S: AsRef<OsStr> + PartialEq<OsStr> + fmt::Debug + ?Sized>(
            csvec: &CStringVec,
            strs: &[&S],
        ) {
            assert_eq!(csvec.len(), strs.len() + 1, "{:?} != {:?}", csvec, strs);
            for (i, s) in strs.iter().enumerate() {
                assert_eq!(csvec.get_cstr(i).unwrap().to_bytes(), s.as_ref().as_bytes());
            }

            assert!(csvec.last().unwrap().is_null(), "{:?}", csvec);
        }

        check_inner(&csvec, strs);
        // Make sure it can be turned into a Vec and back with no problems
        let csvec = unsafe { CStringVec::from_vec(csvec.into_vec()) };
        check_inner(&csvec, strs);
    }

    fn check_cstringvec_empty(csvec: CStringVec) {
        assert_eq!(csvec.len(), 1);
        assert!(csvec[0].is_null());
        assert_eq!(format!("{:?}", csvec), "[None]");
        check_cstringvec(csvec, &[""; 0]);
    }

    #[test]
    fn test_cstringvec_new() {
        check_cstringvec_empty(CStringVec::new());
        check_cstringvec_empty(CStringVec::with_capacity(0));
        check_cstringvec_empty(CStringVec::with_capacity(10));
    }

    #[test]
    fn test_cstringvec_basic() {
        let mut csvec = CStringVec::new();
        csvec.push(CString::new("abc").unwrap());
        csvec.push(CString::new("def").unwrap());
        check_cstringvec(csvec, &["abc", "def"]);
    }

    #[test]
    fn test_cstringvec_get_cstr() {
        let mut csvec = CStringVec::new();
        csvec.push(CString::new("abc").unwrap());
        csvec.push(CString::new("def").unwrap());

        assert_eq!(
            csvec.get_cstr(0),
            Some(CStr::from_bytes_with_nul(b"abc\0").unwrap()),
        );
        assert_eq!(
            csvec.get_cstr(1),
            Some(CStr::from_bytes_with_nul(b"def\0").unwrap()),
        );

        let csvec = unsafe {
            CStringVec::from_vec(vec![
                CString::new("abc").unwrap().into_raw(),
                core::ptr::null_mut(),
                CString::new("def").unwrap().into_raw(),
                core::ptr::null_mut(),
            ])
        };
        assert_eq!(
            csvec.get_cstr(0),
            Some(CStr::from_bytes_with_nul(b"abc\0").unwrap()),
        );
        assert_eq!(csvec.get_cstr(1), None);
        assert_eq!(
            csvec.get_cstr(2),
            Some(CStr::from_bytes_with_nul(b"def\0").unwrap()),
        );
    }

    #[test]
    fn test_cstringvec_replace() {
        let mut csvec = CStringVec::new();
        csvec.push(CString::new("abc").unwrap());
        csvec.push(CString::new("def").unwrap());
        check_cstringvec(csvec.clone(), &["abc", "def"]);

        csvec.replace(1, CString::new("ghi").unwrap());
        check_cstringvec(csvec.clone(), &["abc", "ghi"]);

        csvec.replace(0, CString::new("jkl").unwrap());
        check_cstringvec(csvec, &["jkl", "ghi"]);
    }

    #[test]
    fn test_cstringvec_insert() {
        let mut csvec = CStringVec::new();
        csvec.insert(0, CString::new("abc").unwrap());
        check_cstringvec(csvec.clone(), &["abc"]);

        csvec.push(CString::new("def").unwrap());
        check_cstringvec(csvec.clone(), &["abc", "def"]);

        csvec.insert(1, CString::new("ghi").unwrap());
        check_cstringvec(csvec.clone(), &["abc", "ghi", "def"]);

        csvec.insert(0, CString::new("jkl").unwrap());
        check_cstringvec(csvec.clone(), &["jkl", "abc", "ghi", "def"]);

        csvec.insert(4, CString::new("mno").unwrap());
        check_cstringvec(csvec.clone(), &["jkl", "abc", "ghi", "def", "mno"]);
    }

    #[test]
    fn test_cstringvec_remove() {
        let mut csvec = CStringVec::new();
        csvec.push(CString::new("abc").unwrap());
        csvec.push(CString::new("def").unwrap());
        csvec.push(CString::new("ghi").unwrap());
        csvec.push(CString::new("jkl").unwrap());
        check_cstringvec(csvec.clone(), &["abc", "def", "ghi", "jkl"]);

        assert_eq!(csvec.remove(2), Some(CString::new("ghi").unwrap()));
        check_cstringvec(csvec.clone(), &["abc", "def", "jkl"]);

        assert_eq!(csvec.remove(0), Some(CString::new("abc").unwrap()));
        check_cstringvec(csvec.clone(), &["def", "jkl"]);

        assert_eq!(csvec.remove(1), Some(CString::new("jkl").unwrap()));
        check_cstringvec(csvec.clone(), &["def"]);

        let mut csvec = unsafe {
            CStringVec::from_vec(vec![
                CString::new("abc").unwrap().into_raw(),
                core::ptr::null_mut(),
                CString::new("def").unwrap().into_raw(),
                core::ptr::null_mut(),
            ])
        };
        assert_eq!(csvec.remove(1), None);
        check_cstringvec(csvec.clone(), &["abc", "def"]);
    }

    #[test]
    fn test_cstringvec_reserve() {
        let mut csvec = CStringVec::new();
        assert!(csvec.capacity() >= 1);
        csvec.reserve(30);
        assert!(csvec.capacity() >= 30);
    }

    #[test]
    fn test_cstringvec_collect() {
        check_cstringvec([].iter().cloned().collect(), &[""; 0]);
        check_cstringvec(
            [CString::new("abc").unwrap()].iter().cloned().collect(),
            &["abc"],
        );
        check_cstringvec(
            [CString::new("abc").unwrap(), CString::new("def").unwrap()]
                .iter()
                .cloned()
                .collect(),
            &["abc", "def"],
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_cstringvec_debug() {
        let mut csvec = CStringVec::new();
        csvec.push(CString::new("abc").unwrap());
        assert_eq!(format!("{:?}", csvec), "[Some(\"abc\"), None]");
    }
}
