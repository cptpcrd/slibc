use core::convert::TryInto;
use core::fmt;

use crate::internal_prelude::*;

bitflags::bitflags! {
    /// Flags passed to [`Regex::compile()`].
    ///
    /// See `regcomp(3)` for more information.
    #[derive(Default)]
    pub struct RegexCFlags: libc::c_int {
        /// Use extended regular expressions.
        ///
        /// e.g. `([A-Z]+)` instead of `\([A-Z]\+\)`.
        const EXTENDED = libc::REG_EXTENDED;
        /// Ignore case when matching.
        const ICASE = libc::REG_ICASE;
        /// Don't enable matching capturing groups.
        ///
        /// If this flag is passed, then [`Regex::match_into()`] will always return `Some(&[])`.
        const NOSUB = libc::REG_NOSUB;
        /// Match-any characters (and `[^...]`) won't match newlines.
        const NEWLINE = libc::REG_NEWLINE;
    }
}

bitflags::bitflags! {
    /// Flags passed to the "match" methods of [`Regex`].
    #[derive(Default)]
    pub struct RegexEFlags: libc::c_int {
        /// See the description of `REG_NOTBOL` in `regcomp(3)`.
        const NOTBOL = libc::REG_NOTBOL;
        /// See the description of `REG_NOTEOL` in `regcomp(3)`.
        const NOTEOL = libc::REG_NOTBOL;
    }
}

const _REGEX_T_SIZE_CHECK: sys::regex_t =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<libc::regex_t>()]) };

/// Represents a compiled regex.
#[derive(Debug)]
pub struct Regex {
    preg: libc::regex_t,
    nosub: bool,
}

impl Regex {
    /// Compile the given regex with the given flags.
    #[inline]
    pub fn compile(regex: &CStr, flags: RegexCFlags) -> core::result::Result<Self, RegexError> {
        // Zero it out to be cautious
        let mut preg = unsafe { core::mem::zeroed() };

        match unsafe { libc::regcomp(&mut preg, regex.as_ptr(), flags.bits()) } {
            0 => Ok(Self {
                preg,
                nosub: flags.contains(RegexCFlags::NOSUB),
            }),
            code => Err(RegexError { code, preg }),
        }
    }

    /// Return the number of parenthesized expressions in the compiled regular expression.
    ///
    /// This returns `None` if [`RegexCFlags::NOSUB`] was passed to [`Self::compile()`].
    #[inline]
    pub fn nsub(&self) -> Option<usize> {
        if self.nosub {
            None
        } else {
            Some(unsafe { *(&self.preg as *const libc::regex_t as *const sys::regex_t) }.re_nsub)
        }
    }

    /// Return whether this expression matches the given text.
    ///
    /// This is equivalent to `self.match_into(text, &mut [], eflags).is_some()`, but it may be
    /// faster.
    #[inline]
    pub fn matches(&self, text: &CStr, eflags: RegexEFlags) -> bool {
        unsafe {
            libc::regexec(
                &self.preg,
                text.as_ptr(),
                0,
                core::ptr::null_mut(),
                eflags.bits(),
            ) == 0
        }
    }

    /// Match this expression to the given text and return the captured groups in a buffer.
    ///
    /// `matchbuf` should be a buffer in which the locations of the groups will be placed. If the
    /// buffer is not long enough to hold all of the matches, only the locations of the first few
    /// groups will be placed in there. (Group 0 is the entire match; groups after that represent
    /// capturing groups that were matched with `\\(\\)` or `()`.)
    ///
    /// If the match succeeded, this function returns a slice containing the initialized items of
    /// `matchbuf` (this will always be at the start).
    ///
    /// Example usage:
    ///
    /// ```
    /// # #![cfg(feature = "std")]
    /// # {
    /// # use slibc::{Regex, RegexMatch, RegexCFlags, RegexEFlags};
    /// # use std::ffi::CStr;
    /// let reg = Regex::compile(
    ///     CStr::from_bytes_with_nul(b"^abc\\([0-9]\\+\\)def$\0").unwrap(),
    ///     RegexCFlags::empty(),
    /// ).unwrap();
    ///
    /// // +1 for the "entire match" group
    /// let mut matches = vec![RegexMatch::uninit(); reg.nsub().unwrap() + 1];
    /// let matches = reg.match_into(
    ///     CStr::from_bytes_with_nul(b"abc123def\0").unwrap(),
    ///     &mut matches,
    ///     RegexEFlags::empty(),
    /// ).unwrap();
    ///
    /// assert_eq!(matches.len(), 2);
    /// assert_eq!(matches[0].start(), 0);
    /// assert_eq!(matches[0].end(), 9);
    /// assert_eq!(matches[1].start(), 3);
    /// assert_eq!(matches[1].end(), 6);
    /// # }
    /// ```
    #[inline]
    pub fn match_into<'a>(
        &self,
        text: &CStr,
        mut matchbuf: &'a mut [RegexMatch],
        eflags: RegexEFlags,
    ) -> Option<&'a [RegexMatch]> {
        if self.nosub {
            matchbuf = &mut matchbuf[..0];
        }

        if unsafe {
            libc::regexec(
                &self.preg,
                text.as_ptr() as *const _,
                matchbuf.len(),
                matchbuf.as_mut_ptr() as *mut _,
                eflags.bits(),
            )
        } != 0
        {
            return None;
        }

        let i = matchbuf
            .iter()
            .position(|m| !m.is_init())
            .unwrap_or_else(|| matchbuf.len());

        Some(&matchbuf[..i])
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "android",
        all(target_os = "linux", not(target_env = "musl"))
    )))
)]
#[cfg(any(
    bsd,
    target_os = "android",
    all(target_os = "linux", not(target_os = "musl"))
))]
impl Regex {
    /// Returns whether this expression matches the given text.
    ///
    /// This takes advantage of a BSD extension (which is also present on Android and Linux with
    /// glibc) that allows e.g. searching strings with embedded NUL bytes. (Note that it still isn't
    /// possible to actually match the NUL bytes).
    ///
    /// Note that on some systems (even some 64-bit systems) this function may not be able to
    /// handle text with more than `2 ** 32` bytes.

    #[inline]
    pub fn matches_bytes(&self, text: &[u8], eflags: RegexEFlags) -> bool {
        let mut pmatch = libc::regmatch_t {
            rm_so: 0,
            rm_eo: text.len().try_into().unwrap(),
        };

        unsafe {
            libc::regexec(
                &self.preg,
                text.as_ptr() as *const _,
                0,
                &mut pmatch,
                eflags.bits() | libc::REG_STARTEND,
            ) == 0
        }
    }

    /// A version of [`Self::match_into()`] that takes a byte slice like [`Self::matches_bytes()`].
    ///
    /// See those methods' documentation for more information.
    #[inline]
    pub fn match_bytes_into<'a>(
        &self,
        text: &[u8],
        mut matchbuf: &'a mut [RegexMatch],
        eflags: RegexEFlags,
    ) -> Option<&'a [RegexMatch]> {
        let mut mstartend;
        let pmatch;
        let nmatch;

        if let Some(first) = matchbuf.first_mut() {
            first.0.rm_so = 0;
            first.0.rm_eo = text.len().try_into().unwrap();
            pmatch = first as *mut _;

            if self.nosub {
                matchbuf = &mut matchbuf[..0];
            }
            nmatch = matchbuf.len();
        } else {
            mstartend = RegexMatch::uninit();
            mstartend.0.rm_so = 0;
            mstartend.0.rm_eo = text.len().try_into().unwrap();

            pmatch = &mut mstartend as *mut _;
            nmatch = 0;
        }

        if unsafe {
            libc::regexec(
                &self.preg,
                text.as_ptr() as *const _,
                nmatch,
                pmatch as *mut _ as *mut _,
                eflags.bits(),
            )
        } != 0
        {
            return None;
        }

        let i = matchbuf
            .iter()
            .position(|m| !m.is_init())
            .unwrap_or_else(|| matchbuf.len());

        Some(&matchbuf[..i])
    }
}

impl Drop for Regex {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::regfree(&mut self.preg);
        }
    }
}

/// Represents an error that occurred while compiling a regex.
pub struct RegexError {
    code: i32,
    preg: libc::regex_t,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0; 100];

        let n = unsafe { libc::regerror(self.code, &self.preg, buf.as_mut_ptr(), buf.len()) };
        if n > buf.len() {
            // Too long
            #[cfg(feature = "alloc")]
            {
                let mut buf = vec![0; n];
                unsafe {
                    libc::regerror(self.code, &self.preg, buf.as_mut_ptr(), buf.len());
                }
                let s = util::osstr_from_buf(util::cvt_char_buf(&buf));
                return f.write_str(s.to_str().unwrap());
            }

            #[cfg(not(feature = "alloc"))]
            f.write_str("(truncated)")?;
        }

        let s = util::osstr_from_buf(util::cvt_char_buf(&buf));
        f.write_str(s.to_str().unwrap())
    }
}

impl fmt::Debug for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RegexError {}

/// Represents the location of a match of a regex against a string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct RegexMatch(libc::regmatch_t);

impl RegexMatch {
    /// Create an "uninitialized" regex match.
    ///
    /// This is intended solely for use in creating buffers to be passed to [`Regex::match_into()`].
    ///
    /// (The result isn't `unsafe` to use, but [`Self::start()`] and [`Self::end()`] will return
    /// strange values, so using it may cause bugs.)
    #[allow(clippy::unnecessary_cast)]
    #[inline]
    pub const fn uninit() -> Self {
        Self(libc::regmatch_t {
            rm_so: -1 as _,
            rm_eo: -1 as _,
        })
    }

    #[allow(clippy::unnecessary_cast)]
    #[inline]
    fn is_init(&self) -> bool {
        self.0.rm_so != -1 as _
    }

    /// Return the start index of this match.
    ///
    /// WARNING: This may return a strange value if the match was created with [`Self::uninit()`]!
    /// You should only call this method on items in the slice returned by [`Regex::match_into()`].
    #[inline]
    pub fn start(&self) -> usize {
        self.0.rm_so as usize
    }

    /// Return the end index of this match.
    ///
    /// WARNING: This may return a strange value if the match was created with [`Self::uninit()`]!
    /// You should only call this method on items in the slice returned by [`Regex::match_into()`].
    #[inline]
    pub fn end(&self) -> usize {
        self.0.rm_eo as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_reg_nsub_capture() {
        let reg = Regex::compile(
            CStr::from_bytes_with_nul(b"^abc\\([0-9]\\+\\)def$\0").unwrap(),
            RegexCFlags::empty(),
        )
        .unwrap();

        // +1 for the "entire match" group
        let mut buf = vec![RegexMatch::uninit(); reg.nsub().unwrap() + 1];

        // Matches
        let s = CStr::from_bytes_with_nul(b"abc123def\0").unwrap();
        assert!(reg.matches(s, RegexEFlags::empty()));

        let matches = reg.match_into(s, &mut buf, RegexEFlags::empty()).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start(), 0);
        assert_eq!(matches[0].end(), 9);
        assert_eq!(matches[1].start(), 3);
        assert_eq!(matches[1].end(), 6);

        assert_eq!(
            reg.match_into(s, &mut [], RegexEFlags::empty()),
            Some([].as_ref())
        );

        // Doesn't match
        let s = CStr::from_bytes_with_nul(b"abc123de\0").unwrap();
        assert!(!reg.matches(s, RegexEFlags::empty()));
        assert_eq!(reg.match_into(s, &mut [], RegexEFlags::empty()), None);
        assert_eq!(reg.match_into(s, &mut buf, RegexEFlags::empty()), None);
    }

    #[test]
    fn test_reg_nosub() {
        let reg = Regex::compile(
            CStr::from_bytes_with_nul(b"^abc\\([0-9]\\+\\)def$\0").unwrap(),
            RegexCFlags::NOSUB,
        )
        .unwrap();

        assert_eq!(reg.nsub(), None);

        // Matches
        let s = CStr::from_bytes_with_nul(b"abc123def\0").unwrap();
        assert!(reg.matches(s, RegexEFlags::empty()));
        assert_eq!(
            reg.match_into(s, &mut [], RegexEFlags::empty()),
            Some([].as_ref())
        );
        assert_eq!(
            reg.match_into(s, &mut [RegexMatch::uninit(); 1], RegexEFlags::empty()),
            Some([].as_ref())
        );

        // Doesn't match
        let s = CStr::from_bytes_with_nul(b"abc123de\0").unwrap();
        assert!(!reg.matches(s, RegexEFlags::empty()));
        assert_eq!(reg.match_into(s, &mut [], RegexEFlags::empty()), None);
        assert_eq!(
            reg.match_into(s, &mut [RegexMatch::uninit(); 1], RegexEFlags::empty()),
            None
        );
    }

    #[test]
    fn test_regerror() {
        let err = Regex::compile(
            CStr::from_bytes_with_nul(b"\\\0").unwrap(),
            RegexCFlags::empty(),
        )
        .unwrap_err();

        assert_eq!(err.code, libc::REG_EESCAPE);

        #[cfg(feature = "alloc")]
        {
            let msg = format!("{}", err);
            assert!(msg.to_lowercase().contains("backslash"), "{:?}", msg);
        }
    }

    #[test]
    fn test_match_bytes() {
        let reg = Regex::compile(
            CStr::from_bytes_with_nul(b"^abc\\([0-9]\\+\\)def\0").unwrap(),
            RegexCFlags::empty(),
        )
        .unwrap();

        // +1 for the "entire match" group
        let mut buf = [RegexMatch::uninit(); 3];

        // Matches
        //let s = b"abc123def";
        let s = b"abc123def";
        assert!(reg.matches_bytes(s, RegexEFlags::empty()));

        let matches = reg
            .match_bytes_into(s, &mut buf, RegexEFlags::empty())
            .unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start(), 0);
        assert_eq!(matches[0].end(), 9);
        assert_eq!(matches[1].start(), 3);
        assert_eq!(matches[1].end(), 6);

        assert_eq!(
            reg.match_bytes_into(s, &mut [], RegexEFlags::empty()),
            Some([].as_ref())
        );

        // Doesn't match
        let s = b"abc123de\0";
        assert!(!reg.matches_bytes(s, RegexEFlags::empty()));
        assert_eq!(reg.match_bytes_into(s, &mut [], RegexEFlags::empty()), None);
        assert_eq!(
            reg.match_bytes_into(s, &mut buf, RegexEFlags::empty()),
            None
        );
    }
}
