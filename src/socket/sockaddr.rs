use crate::internal_prelude::*;

use core::fmt;

use super::{AddrParseError, Inet4Addr, Inet6Addr, SockDomain};

/// Represents an IPv4 socket address.
///
/// This wraps a C `sockaddr_in` structure. It contains a 4-octet IPv4 address and a 16-bit port
/// number.
///
/// This structure can be parsed from a string with an address and port number (e.g.
/// `127.0.0.1:80`).
///
/// # Example
///
/// ```
/// # use slibc::{Inet4Addr, Inet4SockAddr};
/// let addr = Inet4SockAddr::new(Inet4Addr::new(127, 0, 0, 1), 80);
/// assert_eq!("127.0.0.1:80".parse(), Ok(addr));
/// ```
#[derive(Copy, Clone)]
pub struct Inet4SockAddr(libc::sockaddr_in);

// INVARIANTS:
// - self.0.sin_family == libc::AF_INET
// - self.0.sin_zero is zeroed
// - (on macOS/*BSD) self.0.sin_len == 0

impl Inet4SockAddr {
    /// Create a new Ipv4 socket address from the given address and port number.
    #[inline]
    pub fn new(ip: Inet4Addr, port: u16) -> Self {
        Self(libc::sockaddr_in {
            sin_family: libc::AF_INET as _,
            sin_port: port,
            sin_zero: [0; 8],
            sin_addr: ip.0,
            #[cfg(bsd)]
            sin_len: 0,
        })
    }

    /// Get the IP address associated with this socket address.
    #[inline]
    pub fn ip(&self) -> Inet4Addr {
        Inet4Addr(self.0.sin_addr)
    }

    /// Set the IP address associated with this socket address.
    #[inline]
    pub fn set_ip(&mut self, ip: Inet4Addr) {
        self.0.sin_addr = ip.0;
    }

    /// Get the port number associated with this address.
    #[inline]
    pub fn port(&self) -> u16 {
        self.0.sin_port
    }

    /// Set the port number associated with this address.
    #[inline]
    pub fn set_port(&mut self, port: u16) {
        self.0.sin_port = port;
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const _,
            core::mem::size_of::<libc::sockaddr_in>() as _,
        )
    }
}

impl AsRef<libc::sockaddr_in> for Inet4SockAddr {
    #[inline]
    fn as_ref(&self) -> &libc::sockaddr_in {
        &self.0
    }
}

impl From<libc::sockaddr_in> for Inet4SockAddr {
    #[inline]
    fn from(mut s: libc::sockaddr_in) -> Self {
        s.sin_family = libc::AF_INET as _;
        s.sin_zero = [0; 8];
        #[cfg(bsd)]
        {
            s.sin_len = 0;
        }

        Self(s)
    }
}

impl PartialEq for Inet4SockAddr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.port() == other.port() && self.ip() == other.ip()
    }
}

impl Eq for Inet4SockAddr {}

impl core::str::FromStr for Inet4SockAddr {
    type Err = AddrParseError;

    #[allow(clippy::many_single_char_names)]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if let Some(i) = s.as_bytes().iter().position(|&ch| ch == b':') {
            let port =
                u16::parse_bytes(&s.as_bytes()[i + 1..], false).map_err(|_| AddrParseError(()))?;

            Ok(Self::new(s[..i].parse()?, port))
        } else {
            Err(AddrParseError(()))
        }
    }
}

impl fmt::Debug for Inet4SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Inet4SockAddr")
            .field("ip", &self.ip())
            .field("port", &self.port())
            .finish()
    }
}

impl fmt::Display for Inet4SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

/// Represents an IPv6 socket address.
///
/// This wraps a C `sockaddr_in6` structure. It contains an 8-segment IPv6 address, a 16-bit port
/// number, and `flowinfo` and `scope_id` fields (see [IETF RFC 2553 Section
/// 3.3](https://tools.ietf.org/html/rfc2553#section-3.3) for details.
///
/// This structure can be parsed from a string with an address and port number (e.g.
/// `::1:80`, `[::]:80`, `2001::db8::1234:80`).
///
/// # Example
///
/// ```
/// # use slibc::{Inet6Addr, Inet6SockAddr};
/// let addr = Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 80, 0, 0);
/// assert_eq!("[::1]:80".parse(), Ok(addr));
/// ```
#[derive(Copy, Clone)]
pub struct Inet6SockAddr(libc::sockaddr_in6);

// INVARIANTS:
// - self.0.sin6_family == libc::AF_INET6
// - (on macOS/*BSD) self.0.sin6_len == 0

impl Inet6SockAddr {
    /// Create a new Ipv6 socket address from an IP address, a port number, and the
    /// `flowinfo` and `scope_id` flags.
    ///
    /// See [IETF RFC 2553 Section 3.3](https://tools.ietf.org/html/rfc2553#section-3.3) for more
    /// information on `flowinfo` and `scope_id`.
    #[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
    #[inline]
    pub const fn new(ip: Inet6Addr, port: u16, flowinfo: u32, scope_id: u32) -> Self {
        Self(libc::sockaddr_in6 {
            sin6_family: libc::AF_INET6 as _,
            sin6_port: port,
            sin6_flowinfo: flowinfo,
            sin6_scope_id: scope_id,
            sin6_addr: ip.0,
            #[cfg(bsd)]
            sin6_len: 0,
        })
    }

    /// Get the port number associated with this address.
    #[inline]
    pub fn port(&self) -> u16 {
        self.0.sin6_port
    }

    /// Set the port number associated with this address.
    #[inline]
    pub fn set_port(&mut self, port: u16) {
        self.0.sin6_port = port;
    }

    /// Get the IP address associated with this socket address.
    #[inline]
    pub fn ip(&self) -> Inet6Addr {
        Inet6Addr(self.0.sin6_addr)
    }

    /// Set the IP address associated with this socket address.
    #[inline]
    pub fn set_ip(&mut self, ip: Inet6Addr) {
        self.0.sin6_addr = ip.0;
    }

    /// Get the flow information associated with this address.
    ///
    /// This corresponds to the `sin6_flowinfo` field in the underlying C structure.
    #[inline]
    pub fn flowinfo(&self) -> u32 {
        self.0.sin6_flowinfo
    }

    /// Set the flow information associated with this address.
    #[inline]
    pub fn set_flowinfo(&mut self, flowinfo: u32) {
        self.0.sin6_flowinfo = flowinfo;
    }

    /// Set the scope ID associated with this address.
    ///
    /// This corresponds to the `sin6_scope_id` field in the underlying C structure.
    #[inline]
    pub fn scope_id(&self) -> u32 {
        self.0.sin6_scope_id
    }

    /// Set the scope ID associated with this address.
    #[inline]
    pub fn set_scope_id(&mut self, scope_id: u32) {
        self.0.sin6_scope_id = scope_id;
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const _,
            core::mem::size_of::<libc::sockaddr_in6>() as _,
        )
    }
}

impl AsRef<libc::sockaddr_in6> for Inet6SockAddr {
    #[inline]
    fn as_ref(&self) -> &libc::sockaddr_in6 {
        &self.0
    }
}

impl From<libc::sockaddr_in6> for Inet6SockAddr {
    #[inline]
    fn from(mut s: libc::sockaddr_in6) -> Self {
        s.sin6_family = libc::AF_INET6 as _;
        #[cfg(bsd)]
        {
            s.sin6_len = 0;
        }

        Self(s)
    }
}

impl PartialEq for Inet6SockAddr {
    fn eq(&self, other: &Self) -> bool {
        self.port() == other.port()
            && self.ip() == other.ip()
            && self.flowinfo() == other.flowinfo()
            && self.scope_id() == other.scope_id()
    }
}

impl Eq for Inet6SockAddr {}

impl core::str::FromStr for Inet6SockAddr {
    type Err = AddrParseError;

    fn from_str(mut s: &str) -> core::result::Result<Self, Self::Err> {
        if let Some(i) = s.as_bytes().iter().rposition(|&ch| ch == b':') {
            if let Ok(port) = u16::parse_bytes(&s.as_bytes()[i + 1..], false) {
                s = &s[..i];
                match (s.starts_with('['), s.ends_with(']')) {
                    (true, true) => s = &s[1..s.len() - 1],
                    (true, false) | (false, true) => return Err(AddrParseError(())),
                    _ => (),
                }

                return Ok(Self::new(s.parse()?, port, 0, 0));
            }
        }

        Err(AddrParseError(()))
    }
}

impl fmt::Debug for Inet6SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Inet6SockAddr")
            .field("ip", &self.ip())
            .field("port", &self.port())
            .field("flowinfo", &self.flowinfo())
            .field("scope_id", &self.scope_id())
            .finish()
    }
}

impl fmt::Display for Inet6SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

/// Represents a Unix socket address.
#[derive(Copy, Clone)]
pub struct UnixAddr(libc::sockaddr_un);

// INVARIANTS:
// - self.0.sun_family == libc::AF_UNIX
// - self.0.sun_path contains a terminating NUL byte (on Linux/Android, there must be a NUL after byte 0)
// - (on macOS/*BSD) self.0.sun_len == 0

impl UnixAddr {
    /// Create a new Unix socket address representing the given `path`.
    pub fn new<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path = path.as_ref().as_bytes();
        if path.contains(&0) {
            return Err(Error::mid_nul());
        }

        let mut addr: libc::sockaddr_un = unsafe { core::mem::zeroed() };
        if path.len() >= addr.sun_path.len() - 1 {
            return Err(Error::from_code(libc::ENAMETOOLONG));
        }

        addr.sun_family = libc::AF_UNIX as _;
        addr.sun_path[..path.len()].copy_from_slice(util::cvt_u8_buf(path));
        debug_assert_eq!(addr.sun_path[path.len()], 0);
        Ok(Self(addr))
    }

    /// Create a new abstract Unix socket address with the given `name`.
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[cfg(linuxlike)]
    pub fn new_abstract<N: AsRef<OsStr>>(name: N) -> Result<Self> {
        let name = name.as_ref().as_bytes();
        if name.contains(&0) {
            return Err(Error::mid_nul());
        }

        let mut addr: libc::sockaddr_un = unsafe { core::mem::zeroed() };
        if name.len() >= addr.sun_path.len() - 2 {
            return Err(Error::from_code(libc::ENAMETOOLONG));
        }

        addr.sun_family = libc::AF_UNIX as _;
        addr.sun_path[1..name.len() + 1].copy_from_slice(util::cvt_u8_buf(name));
        debug_assert_eq!(addr.sun_path[0], 0);
        debug_assert_eq!(addr.sun_path[name.len() + 1], 0);
        Ok(Self(addr))
    }

    #[inline]
    pub fn new_unnamed() -> Self {
        Self(libc::sockaddr_un {
            sun_family: libc::AF_UNIX as _,
            ..unsafe { core::mem::zeroed() }
        })
    }

    /// If this `UnixAddr` represents a path, return the path as an `OsStr`.
    #[inline]
    pub fn path(&self) -> Option<&OsStr> {
        if self.0.sun_path[0] == 0 {
            None
        } else {
            Some(util::osstr_from_buf(util::cvt_char_buf(&self.0.sun_path)))
        }
    }

    /// If this `UnixAddr` represents an abstract address, return the abstract name as an `OsStr`.
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[cfg(linuxlike)]
    #[inline]
    pub fn abstract_name(&self) -> Option<&OsStr> {
        if self.0.sun_path[0] != 0 || self.0.sun_path[1] == 0 {
            None
        } else {
            Some(util::osstr_from_buf(util::cvt_char_buf(
                &self.0.sun_path[1..],
            )))
        }
    }

    /// Check whether this `UnixAddr` represents an unnamed address.
    ///
    /// This method returns `false` if and only if [`Self::path()`] (and `Self::abstract_name()` on
    /// Linux/Android) would both return None.
    #[inline]
    pub fn is_unnamed(&self) -> bool {
        if self.0.sun_path[0] != 0 {
            return false;
        }

        #[cfg(linuxlike)]
        if self.0.sun_path[1] != 0 {
            return false;
        }

        true
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const _,
            core::mem::size_of::<libc::sockaddr_un>() as _,
        )
    }
}

impl AsRef<libc::sockaddr_un> for UnixAddr {
    #[inline]
    fn as_ref(&self) -> &libc::sockaddr_un {
        &self.0
    }
}

impl From<libc::sockaddr_un> for UnixAddr {
    fn from(mut s: libc::sockaddr_un) -> Self {
        #[cfg(linuxlike)]
        assert!(s.sun_path[1..].contains(&0));
        #[cfg(not(linuxlike))]
        assert!(s.sun_path.contains(&0));

        s.sun_family = libc::AF_UNIX as _;
        #[cfg(bsd)]
        {
            s.sun_len = 0;
        }

        Self(s)
    }
}

impl PartialEq for UnixAddr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        #[cfg(linuxlike)]
        {
            // No matter what is in these addresses, the first character of each must match
            if self.0.sun_path[0] != other.0.sun_path[0] {
                return false;
            }

            // Now compare the rest of the bytes up to the first NUL
            // There *has* to be a NUL after the first byte; this is an invariant
            for i in 1..self.0.sun_path.len() {
                if self.0.sun_path[i] != other.0.sun_path[i] {
                    // Mismatch
                    return false;
                } else if self.0.sun_path[i] == 0 {
                    // Terminating NUL
                    return true;
                }
            }
        }

        // Compare the bytes up to the first NUL
        #[cfg(not(linuxlike))]
        for i in 0..self.0.sun_path.len() {
            if self.0.sun_path[i] != other.0.sun_path[i] {
                // Mismatch
                return false;
            } else if self.0.sun_path[i] == 0 {
                // Terminating NUL
                return true;
            }
        }

        true
    }
}

impl Eq for UnixAddr {}

impl fmt::Debug for UnixAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(path) = self.path() {
            return f.debug_tuple("Path").field(&path).finish();
        }

        #[cfg(linuxlike)]
        if let Some(name) = self.abstract_name() {
            return f.debug_tuple("Abstract").field(&name).finish();
        }

        f.write_str("Unnamed")
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SockAddr {
    Inet4(Inet4SockAddr),
    Inet6(Inet6SockAddr),
    Unix(UnixAddr),
}

impl SockAddr {
    #[inline]
    pub fn family(&self) -> SockDomain {
        match self {
            Self::Inet4(_) => SockDomain::INET,
            Self::Inet6(_) => SockDomain::INET6,
            Self::Unix(_) => SockDomain::UNIX,
        }
    }

    /// Construct a `SockAddr` from the given initialized socket address storage space.
    ///
    /// # Panics
    ///
    /// May panic if the given address is not properly initialized (e.g. the path for a Unix
    /// address contains a NUL byte).
    #[inline]
    pub fn from_raw(storage: libc::sockaddr_storage, len: libc::socklen_t) -> Result<Self> {
        match storage.ss_family as _ {
            libc::AF_INET if len >= core::mem::size_of::<libc::sockaddr_in>() as _ => {
                assert!(
                    core::mem::size_of::<libc::sockaddr_storage>()
                        >= core::mem::size_of::<libc::sockaddr_in6>()
                );

                Ok(Self::Inet4(Inet4SockAddr::from(unsafe {
                    core::mem::transmute_copy::<_, libc::sockaddr_in>(&storage)
                })))
            }

            libc::AF_INET6 if len >= core::mem::size_of::<libc::sockaddr_in6>() as _ => {
                assert!(
                    core::mem::size_of::<libc::sockaddr_storage>()
                        >= core::mem::size_of::<libc::sockaddr_in>()
                );

                Ok(Self::Inet6(Inet6SockAddr::from(unsafe {
                    core::mem::transmute_copy::<_, libc::sockaddr_in6>(&storage)
                })))
            }

            libc::AF_UNIX
                if len
                    >= (core::mem::size_of::<libc::sockaddr_un>()
                        - unsafe { core::mem::zeroed::<libc::sockaddr_un>() }
                            .sun_path
                            .len()) as _ =>
            {
                assert!(
                    core::mem::size_of::<libc::sockaddr_storage>()
                        >= core::mem::size_of::<libc::sockaddr_un>()
                );

                Ok(Self::Unix(UnixAddr::from(unsafe {
                    core::mem::transmute_copy::<_, libc::sockaddr_un>(&storage)
                })))
            }

            _ => Err(Error::from_code(libc::EINVAL)),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        match self {
            Self::Inet4(addr) => addr.as_raw(),
            Self::Inet6(addr) => addr.as_raw(),
            Self::Unix(addr) => addr.as_raw(),
        }
    }

    #[inline]
    pub fn unwrap_inet4(self) -> Inet4SockAddr {
        match self {
            Self::Inet4(addr) => addr,
            _ => panic!("unwrap_inet4() called on a non-inet4 socket"),
        }
    }

    #[inline]
    pub fn unwrap_inet6(self) -> Inet6SockAddr {
        match self {
            Self::Inet6(addr) => addr,
            _ => panic!("unwrap_inet6() called on a non-inet6 socket"),
        }
    }

    #[inline]
    pub fn unwrap_unix(self) -> UnixAddr {
        match self {
            Self::Unix(addr) => addr,
            _ => panic!("unwrap_unix() called on a non-unix socket"),
        }
    }

    /// Returns the port number associated with this address (if it has one).
    ///
    /// # Examples
    /// ```
    /// # use slibc::{SockAddr, UnixAddr};
    /// assert_eq!(SockAddr::Inet4("127.0.0.1:8080".parse().unwrap()).port(), Some(8080));
    /// assert_eq!(SockAddr::Inet6("::1:8080".parse().unwrap()).port(), Some(8080));
    /// assert_eq!(SockAddr::Unix(UnixAddr::new("/tmp/sock").unwrap()).port(), None);
    /// ```
    #[inline]
    pub fn port(&self) -> Option<u16> {
        match self {
            Self::Inet4(addr) => Some(addr.port()),
            Self::Inet6(addr) => Some(addr.port()),
            Self::Unix(_) => None,
        }
    }

    /// Returns whether this `SockAddr` contains an IPv4 address.
    #[inline]
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Inet4(_))
    }

    /// Returns whether this `SockAddr` contains an IPv6 address.
    #[inline]
    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Inet6(_))
    }

    /// Returns whether this `SockAddr` contains a Unix domain address.
    #[inline]
    pub fn is_unix(&self) -> bool {
        matches!(self, Self::Unix(_))
    }
}

impl From<Inet4SockAddr> for SockAddr {
    #[inline]
    fn from(a: Inet4SockAddr) -> Self {
        Self::Inet4(a)
    }
}

impl From<Inet6SockAddr> for SockAddr {
    #[inline]
    fn from(a: Inet6SockAddr) -> Self {
        Self::Inet6(a)
    }
}

impl From<UnixAddr> for SockAddr {
    #[inline]
    fn from(a: UnixAddr) -> Self {
        Self::Unix(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unixaddr_path() {
        let addr = UnixAddr::new("abc").unwrap();
        assert_eq!(addr.path().unwrap(), "abc");
        #[cfg(linuxlike)]
        assert_eq!(addr.abstract_name(), None);
        assert!(!addr.is_unnamed());

        assert_eq!(UnixAddr::new("abc\0def").unwrap_err(), Errno::EINVAL);

        assert_eq!(addr, UnixAddr::new("abc").unwrap());
        assert_ne!(addr, UnixAddr::new("").unwrap());
        #[cfg(linuxlike)]
        assert_ne!(addr, UnixAddr::new_abstract("abc").unwrap());

        #[cfg(linuxlike)]
        {
            UnixAddr::new(OsStr::from_bytes(&[b'a'; 106])).unwrap();
            assert_eq!(
                UnixAddr::new(OsStr::from_bytes(&[b'a'; 107])).unwrap_err(),
                Errno::ENAMETOOLONG
            );
            assert_eq!(
                UnixAddr::new(OsStr::from_bytes(&[b'a'; 108])).unwrap_err(),
                Errno::ENAMETOOLONG
            );
        }
    }

    #[cfg(linuxlike)]
    #[test]
    fn test_unixaddr_abstract() {
        let addr = UnixAddr::new_abstract("abc").unwrap();
        assert_eq!(addr.abstract_name().unwrap(), "abc");
        assert_eq!(addr.path(), None);
        assert!(!addr.is_unnamed());

        assert_eq!(addr, UnixAddr::new_abstract("abc").unwrap());
        assert_ne!(addr, UnixAddr::new("abc").unwrap());
        #[cfg(linuxlike)]
        assert_ne!(addr, UnixAddr::new("").unwrap());

        assert_eq!(
            UnixAddr::new_abstract("abc\0def").unwrap_err(),
            Errno::EINVAL
        );

        UnixAddr::new_abstract(OsStr::from_bytes(&[b'a'; 105])).unwrap();
        assert_eq!(
            UnixAddr::new_abstract(OsStr::from_bytes(&[b'a'; 106])).unwrap_err(),
            Errno::ENAMETOOLONG
        );
        assert_eq!(
            UnixAddr::new_abstract(OsStr::from_bytes(&[b'a'; 107])).unwrap_err(),
            Errno::ENAMETOOLONG
        );
    }

    #[test]
    fn test_unixaddr_unnamed() {
        let addr = UnixAddr::new_unnamed();
        assert_eq!(addr.path(), None);
        #[cfg(linuxlike)]
        assert_eq!(addr.abstract_name(), None);
        assert!(addr.is_unnamed());

        assert_eq!(addr, UnixAddr::new("").unwrap());
        assert_ne!(addr, UnixAddr::new("abc").unwrap());
        #[cfg(linuxlike)]
        assert_ne!(addr, UnixAddr::new_abstract("abc").unwrap());
    }

    #[test]
    fn test_inet4addr() {
        let addr = Inet4SockAddr::new(Inet4Addr::new(1, 2, 3, 4), 80);
        assert_eq!(addr.ip(), Inet4Addr::new(1, 2, 3, 4));
        assert_eq!(addr.port(), 80);

        let mut addr2 = Inet4SockAddr::new(Inet4Addr::LOCALHOST, 0);
        addr2.set_ip(Inet4Addr::new(1, 2, 3, 4));
        addr2.set_port(80);

        assert_eq!(addr, addr2);
        assert_eq!(addr.0, addr2.0);
    }

    #[test]
    fn test_inet4addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet4SockAddr::from_str("0.0.0.0:80").unwrap(),
            Inet4SockAddr::new(Inet4Addr::new(0, 0, 0, 0), 80)
        );
        assert_eq!(
            Inet4SockAddr::from_str("127.0.0.1:0").unwrap(),
            Inet4SockAddr::new(Inet4Addr::new(127, 0, 0, 1), 0)
        );

        Inet4SockAddr::from_str("127.0.0.1").unwrap_err();
        Inet4SockAddr::from_str("127.0.0.1d:80").unwrap_err();
        Inet4SockAddr::from_str("127.0.0.:80").unwrap_err();
        Inet4SockAddr::from_str("127.0.0:80").unwrap_err();
        Inet4SockAddr::from_str("127.0.0.1.1:80").unwrap_err();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_inet4addr_display_debug() {
        assert_eq!(
            format!("{}", Inet4SockAddr::new(Inet4Addr::new(1, 2, 3, 4), 80)),
            "1.2.3.4:80"
        );

        assert_eq!(
            format!("{:?}", Inet4SockAddr::new(Inet4Addr::new(1, 2, 3, 4), 80)),
            "Inet4SockAddr { ip: \"1.2.3.4\", port: 80 }"
        );
    }

    #[test]
    fn test_inet6addr() {
        let addr = Inet6SockAddr::new(Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1), 80, 0, 0);
        assert_eq!(addr.ip(), Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1));
        assert_eq!(addr.port(), 80);

        let mut addr2 = Inet6SockAddr::new(Inet6Addr::LOCALHOST, 0, 0, 0);
        addr2.set_ip(Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1));
        addr2.set_port(80);

        assert_eq!(addr, addr2);
        assert_eq!(addr.0, addr2.0);
    }

    #[test]
    fn test_inet6addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet6SockAddr::from_str("::1:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 80, 0, 0)
        );
        assert_eq!(
            Inet6SockAddr::from_str("[::1]:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 80, 0, 0)
        );
        assert_eq!(
            Inet6SockAddr::from_str(":::80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 80, 0, 0)
        );
        assert_eq!(
            Inet6SockAddr::from_str("1::1:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 1), 80, 0, 0)
        );
        assert_eq!(
            Inet6SockAddr::from_str("2001:db8::1:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1), 80, 0, 0)
        );

        assert_eq!(
            Inet6SockAddr::from_str("[::ffff:192.168.1.2]:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 65535, 49320, 258), 80, 0, 0)
        );

        assert_eq!(
            Inet6SockAddr::from_str("[::192.168.1.2]:80").unwrap(),
            Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 49320, 258), 80, 0, 0)
        );

        Inet6SockAddr::from_str(":1:80").unwrap_err();
        Inet6SockAddr::from_str("1:::80").unwrap_err();
        Inet6SockAddr::from_str("1::0:::80").unwrap_err();
        Inet6SockAddr::from_str("1:2:3:4:5:6:7::80").unwrap_err();
        Inet6SockAddr::from_str("1:2:3:4:5:6:7:::80").unwrap_err();
        Inet6SockAddr::from_str("1:2:3:4:5:6:7:8:9:80").unwrap_err();
        Inet6SockAddr::from_str("::g:80").unwrap_err();
        Inet6SockAddr::from_str("[::1:80").unwrap_err();
        Inet6SockAddr::from_str("::1]:80").unwrap_err();
        Inet6SockAddr::from_str("[::ffff:192.168.1.2:]:80").unwrap_err();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_inet6addr_display_debug() {
        for (addr, s) in [
            // Simple case
            (
                Inet6SockAddr::new(Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 80, 0, 0),
                "[1:2:3:4:5:6:7:8]:80",
            ),
            // All zeroes
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 80, 0, 0),
                "[::]:80",
            ),
            // Only one 1 (various positions)
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 80, 0, 0),
                "[::1]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 1, 0, 0), 80, 0, 0),
                "[::1:0:0]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 0), 80, 0, 0),
                "[1::]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 1, 0, 0, 0, 0, 0, 0), 80, 0, 0),
                "[0:1::]:80",
            ),
            // The longest string of 0s will be selected for shortening with "::"
            (
                Inet6SockAddr::new(Inet6Addr::new(1, 0, 0, 1, 0, 0, 0, 0), 80, 0, 0),
                "[1:0:0:1::]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(1, 0, 0, 1, 0, 0, 0, 1), 80, 0, 0),
                "[1:0:0:1::1]:80",
            ),
            // But if they're the same length, the first one is used
            (
                Inet6SockAddr::new(Inet6Addr::new(1, 0, 0, 1, 0, 0, 1, 0), 80, 0, 0),
                "[1::1:0:0:1:0]:80",
            ),
            // Embedded IPv4
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 65535, 49320, 258), 80, 0, 0),
                "[::ffff:192.168.1.2]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0), 80, 0, 0),
                "[::ffff:0.0.0.0]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1), 80, 0, 0),
                "[::ffff:0.0.0.1]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 49320, 258), 80, 0, 0),
                "[::192.168.1.2]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 258), 80, 0, 0),
                "[::0.0.1.2]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2), 80, 0, 0),
                "[::0.0.0.2]:80",
            ),
            (
                Inet6SockAddr::new(Inet6Addr::new(0, 0, 0, 0, 0, 0, 258, 0), 80, 0, 0),
                "[::1.2.0.0]:80",
            ),
        ]
        .iter()
        {
            assert_eq!(format!("{}", addr), *s);
        }

        assert_eq!(
            format!(
                "{:?}",
                Inet6SockAddr::new(Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 80, 0, 0)
            ),
            "Inet6SockAddr { ip: \"1:2:3:4:5:6:7:8\", port: 80, flowinfo: 0, scope_id: 0 }"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_unixaddr_debug() {
        assert_eq!(format!("{:?}", UnixAddr::new("").unwrap()), "Unnamed");
        assert_eq!(
            format!("{:?}", UnixAddr::new("abc").unwrap()),
            "Path(\"abc\")"
        );

        #[cfg(linuxlike)]
        assert_eq!(
            format!("{:?}", UnixAddr::new_abstract("abc").unwrap()),
            "Abstract(\"abc\")"
        );
    }
}
