use crate::internal_prelude::*;

use core::fmt;

use super::SockDomain;

#[allow(clippy::many_single_char_names)]
fn parse_v4_octets(s: &[u8]) -> Option<[u8; 4]> {
    let mut items = s
        .split(|&ch| ch == b'.')
        .map(|s| u8::parse_bytes(s, false).ok());

    let a = items.next().flatten()?;
    let b = items.next().flatten()?;
    let c = items.next().flatten()?;
    let d = items.next().flatten()?;

    if items.next().is_none() {
        Some([a, b, c, d])
    } else {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SockAddrParseError(());

impl fmt::Display for SockAddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid address")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SockAddrParseError {}

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
/// # use slibc::Inet4Addr;
/// let addr = Inet4Addr::new(127, 0, 0, 1, 80);
/// assert_eq!("127.0.0.1:80".parse(), Ok(addr));
/// ```
#[derive(Copy, Clone)]
pub struct Inet4Addr(libc::sockaddr_in);

// INVARIANTS:
// - self.0.sin_family == libc::AF_INET
// - self.0.sin_zero is zeroed
// - (on macOS/*BSD) self.0.sin_len == 0

impl Inet4Addr {
    /// Create a new Ipv4 socket address from the given octets and port number.
    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8, port: u16) -> Self {
        Self(libc::sockaddr_in {
            sin_family: libc::AF_INET as _,
            sin_port: port,
            sin_zero: [0; 8],
            sin_addr: libc::in_addr {
                s_addr: u32::from_be_bytes([a, b, c, d]),
            },
            #[cfg(bsd)]
            sin_len: 0,
        })
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

    /// Get the four octets that represent the address portion of this socket address.
    #[inline]
    pub fn octets(&self) -> [u8; 4] {
        self.0.sin_addr.s_addr.to_be_bytes()
    }

    /// Replace the address portion of this socket address with the given octets.
    #[inline]
    pub fn set_octets(&mut self, octets: [u8; 4]) {
        self.0.sin_addr.s_addr = u32::from_be_bytes(octets);
    }

    /// Check whether the address portion of this socket address represents the "unspecified"
    /// address (`0.0.0.0`).
    #[inline]
    pub fn is_unspecified(&self) -> bool {
        self.octets() == [0, 0, 0, 0]
    }

    /// Check whether the address portion of this socket address represents a loopback address
    /// (`127.0.0.1/8`).
    #[inline]
    pub fn is_loopback(&self) -> bool {
        self.octets()[0] == 127
    }

    /// Check whether the address portion of this socket address represents a private address
    /// (`10.0.0.0/8`, `172.16.0.0/12`, or `192.168.0.0/16`).
    #[inline]
    pub fn is_private(&self) -> bool {
        match self.octets() {
            [10, ..] => true,
            [192, 168, ..] => true,
            [172, x, ..] if x & 0xF0 == 16 => true,
            _ => false,
        }
    }

    /// Check whether the address portion of this socket address represents a link-local address
    /// (`169.254.0.0/16`).
    #[inline]
    pub fn is_link_local(&self) -> bool {
        self.octets()[..2] == [169, 254]
    }

    /// Check whether the address portion of this socket address represents the broadcast
    /// address (`255.255.255.255`).
    #[inline]
    pub fn is_broadcast(&self) -> bool {
        self.octets() == [255, 255, 255, 255]
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const _,
            core::mem::size_of::<libc::sockaddr_in>() as _,
        )
    }
}

impl AsRef<libc::sockaddr_in> for Inet4Addr {
    #[inline]
    fn as_ref(&self) -> &libc::sockaddr_in {
        &self.0
    }
}

impl From<libc::sockaddr_in> for Inet4Addr {
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

impl PartialEq for Inet4Addr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.port() == other.port() && self.octets() == other.octets()
    }
}

impl Eq for Inet4Addr {}

impl core::str::FromStr for Inet4Addr {
    type Err = SockAddrParseError;

    #[allow(clippy::many_single_char_names)]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.as_bytes();

        if let Some(i) = s.iter().position(|&ch| ch == b':') {
            let port = u16::parse_bytes(&s[i + 1..], false).map_err(|_| SockAddrParseError(()))?;

            let [a, b, c, d] = parse_v4_octets(&s[..i]).ok_or(SockAddrParseError(()))?;
            Ok(Self::new(a, b, c, d, port))
        } else {
            Err(SockAddrParseError(()))
        }
    }
}

impl fmt::Debug for Inet4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Inet4Addr")
            .field("addr", &Ipv4Octets(self.octets()))
            .field("port", &self.port())
            .finish()
    }
}

impl fmt::Display for Inet4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", Ipv4Octets(self.octets()), self.port())
    }
}

struct Ipv4Octets([u8; 4]);

impl fmt::Debug for Ipv4Octets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl fmt::Display for Ipv4Octets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
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
/// # use slibc::Inet6Addr;
/// let addr = Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0);
/// assert_eq!("[::1]:80".parse(), Ok(addr));
/// ```
#[derive(Copy, Clone)]
pub struct Inet6Addr(libc::sockaddr_in6);

// INVARIANTS:
// - self.0.sin6_family == libc::AF_INET6
// - (on macOS/*BSD) self.0.sin6_len == 0

impl Inet6Addr {
    /// Create a new Ipv6 socket address from eight 16-bit segments, a port number, and the
    /// `flowinfo` and `scope_id` flags.
    ///
    /// See [IETF RFC 2553 Section 3.3](https://tools.ietf.org/html/rfc2553#section-3.3) for more
    /// information on `flowinfo` and `scope_id`.
    #[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
    #[inline]
    pub const fn new(
        a: u16,
        b: u16,
        c: u16,
        d: u16,
        e: u16,
        f: u16,
        g: u16,
        h: u16,
        port: u16,
        flowinfo: u32,
        scope_id: u32,
    ) -> Self {
        let [a1, a2] = a.to_be_bytes();
        let [b1, b2] = b.to_be_bytes();
        let [c1, c2] = c.to_be_bytes();
        let [d1, d2] = d.to_be_bytes();
        let [e1, e2] = e.to_be_bytes();
        let [f1, f2] = f.to_be_bytes();
        let [g1, g2] = g.to_be_bytes();
        let [h1, h2] = h.to_be_bytes();

        Self(libc::sockaddr_in6 {
            sin6_family: libc::AF_INET as _,
            sin6_port: port,
            sin6_flowinfo: flowinfo,
            sin6_scope_id: scope_id,
            sin6_addr: libc::in6_addr {
                s6_addr: [
                    a1, a2, b1, b2, c1, c2, d1, d2, e1, e2, f1, f2, g1, g2, h1, h2,
                ],
            },
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

    /// Get the sixteen octets that represent the address portion of this socket address.
    #[inline]
    pub fn octets(&self) -> [u8; 16] {
        self.0.sin6_addr.s6_addr
    }

    /// Replace the address portion of this socket address with the given octets.
    #[inline]
    pub fn set_octets(&mut self, octets: [u8; 16]) {
        self.0.sin6_addr.s6_addr = octets;
    }

    /// Get the eight 16-bit segments that represent the address portion of this socket address.
    #[inline]
    pub fn segments(&self) -> [u16; 8] {
        let [a1, a2, b1, b2, c1, c2, d1, d2, e1, e2, f1, f2, g1, g2, h1, h2] =
            self.0.sin6_addr.s6_addr;

        [
            u16::from_be_bytes([a1, a2]),
            u16::from_be_bytes([b1, b2]),
            u16::from_be_bytes([c1, c2]),
            u16::from_be_bytes([d1, d2]),
            u16::from_be_bytes([e1, e2]),
            u16::from_be_bytes([f1, f2]),
            u16::from_be_bytes([g1, g2]),
            u16::from_be_bytes([h1, h2]),
        ]
    }

    /// Replace the address portion of this socket address with the given eight 16-bit segments.
    #[inline]
    pub fn set_segments(&mut self, segments: [u16; 8]) {
        let [a1, a2] = segments[0].to_be_bytes();
        let [b1, b2] = segments[1].to_be_bytes();
        let [c1, c2] = segments[2].to_be_bytes();
        let [d1, d2] = segments[3].to_be_bytes();
        let [e1, e2] = segments[4].to_be_bytes();
        let [f1, f2] = segments[5].to_be_bytes();
        let [g1, g2] = segments[6].to_be_bytes();
        let [h1, h2] = segments[7].to_be_bytes();

        self.0.sin6_addr.s6_addr = [
            a1, a2, b1, b2, c1, c2, d1, d2, e1, e2, f1, f2, g1, g2, h1, h2,
        ];
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

    /// Check whether the address portion of this socket address represents the "unspecified"
    /// address (`::`).
    #[inline]
    pub fn is_unspecified(&self) -> bool {
        self.octets() == [0; 16]
    }

    /// Check whether the address portion of this socket address represents a loopback address
    /// (`::1`).
    #[inline]
    pub fn is_loopback(&self) -> bool {
        self.octets() == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    }

    /// Check whether the address portion of this socket address represents a multicast address
    /// (`ff00::/8`).
    #[inline]
    pub fn is_multicast(&self) -> bool {
        self.octets()[0] == 0xFF
    }

    /// If this address is of the form `::a.b.c.d` or `::ffff:a.b.c.d`, return the IPv4 version
    /// (i.e. `a.b.c.d`).
    ///
    /// Note that this will return an IPv4 version of some addresses that might *really* be IPv6
    /// addresses. For example:
    ///
    /// ```
    /// # use slibc::{Inet4Addr, Inet6Addr};
    /// // "::1" -> "0.0.0.1"
    /// assert_eq!(
    ///     Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0).to_ipv4(),
    ///     Some(Inet4Addr::new(0, 0, 0, 1, 80)),
    /// );
    /// ```
    #[inline]
    pub fn to_ipv4(&self) -> Option<Inet4Addr> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, a, b, c, d] => {
                Some(Inet4Addr::new(a, b, c, d, self.port()))
            }
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d] => {
                Some(Inet4Addr::new(a, b, c, d, self.port()))
            }
            _ => None,
        }
    }

    /// If this address is of the form `::ffff:a.b.c.d`, return the IPv4 version (i.e. `a.b.c.d`).
    #[inline]
    pub fn to_ipv4_mapped(&self) -> Option<Inet4Addr> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, a, b, c, d] => {
                Some(Inet4Addr::new(a, b, c, d, self.port()))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const _,
            core::mem::size_of::<libc::sockaddr_in6>() as _,
        )
    }
}

impl AsRef<libc::sockaddr_in6> for Inet6Addr {
    #[inline]
    fn as_ref(&self) -> &libc::sockaddr_in6 {
        &self.0
    }
}

impl From<libc::sockaddr_in6> for Inet6Addr {
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

impl PartialEq for Inet6Addr {
    fn eq(&self, other: &Self) -> bool {
        self.port() == other.port()
            && self.octets() == other.octets()
            && self.flowinfo() == other.flowinfo()
            && self.scope_id() == other.scope_id()
    }
}

impl Eq for Inet6Addr {}

impl core::str::FromStr for Inet6Addr {
    type Err = SockAddrParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.as_bytes();

        fn parse_addr(mut s: &[u8], port: u16) -> Option<Inet6Addr> {
            match (s.first()?, s.last()?) {
                (&b'[', &b']') => s = &s[1..s.len() - 1],
                (&b'[', _) => return None,
                (_, &b']') => return None,
                _ => (),
            }

            let mut segs_buf = [0; 8];
            let mut segs = &mut segs_buf[..];

            let last_colon_idx = s.iter().rposition(|&ch| ch == b':')?;
            if s[last_colon_idx + 1..].contains(&b'.') {
                // Embedded IPv4 at the end
                let octets = parse_v4_octets(&s[last_colon_idx + 1..])?;
                segs[6] = u16::from_be_bytes([octets[0], octets[1]]);
                segs[7] = u16::from_be_bytes([octets[2], octets[3]]);
                segs = &mut segs_buf[..6];

                s = &s[..last_colon_idx];
                if s == b":" {
                    // The string was "::a.b.c.d". When trimming the last ":", we trimmed it down
                    // to a single ":". Replace it with "::" so it gets parsed correctly.
                    s = b"::";
                }
            }

            // "::" is all zeroes. It's a special case that's messy to handle during parsing, so we
            // just skip parsing altogether in that case.
            if s != b"::" {
                let mut items = s.split(|&ch| ch == b':');

                for (i, item) in items.by_ref().enumerate() {
                    if item.is_empty() {
                        // We have a blank space; probably a "::"

                        if i == 0 {
                            // If the first one was empty, it's something like ":..."
                            // This is only valid for things like "::...", so the next one has to be
                            // empty too
                            if !items.next()?.is_empty() {
                                return None;
                            }
                        }

                        let mut items = items.rev().peekable();

                        // Not allowed to end with a ':'; at least one element must follow
                        items.peek()?;

                        for item in items.rev() {
                            let (last, rest) = segs.split_last_mut()?;
                            *last = u16::parse_bytes_radix(item, 16, false).ok()?;
                            segs = rest;
                        }
                        break;
                    }

                    let (first, rest) = segs.split_first_mut()?;
                    *first = u16::parse_bytes_radix(item, 16, false).ok()?;
                    segs = rest;
                }
            }

            Some(Inet6Addr::new(
                segs_buf[0],
                segs_buf[1],
                segs_buf[2],
                segs_buf[3],
                segs_buf[4],
                segs_buf[5],
                segs_buf[6],
                segs_buf[7],
                port,
                0,
                0,
            ))
        }

        if let Some(i) = s.iter().rposition(|&ch| ch == b':') {
            if let Ok(port) = u16::parse_bytes(&s[i + 1..], false) {
                if let Some(addr) = parse_addr(&s[..i], port) {
                    return Ok(addr);
                }
            }
        }

        Err(SockAddrParseError(()))
    }
}

impl fmt::Debug for Inet6Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Inet6Addr")
            .field("addr", &Ipv6Segments(self.segments()))
            .field("port", &self.port())
            .field("flowinfo", &self.flowinfo())
            .field("scope_id", &self.scope_id())
            .finish()
    }
}

impl fmt::Display for Inet6Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]:{}", Ipv6Segments(self.segments()), self.port())
    }
}

struct Ipv6Segments([u16; 8]);

impl fmt::Debug for Ipv6Segments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl fmt::Display for Ipv6Segments {
    #[allow(clippy::many_single_char_names)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut zeroed_range = 0..0;
        let mut first_zero = None;

        for (i, seg) in self.0.iter().copied().enumerate() {
            if seg == 0 {
                if first_zero.is_none() {
                    first_zero = Some(i);
                }
            } else if let Some(first_zero) = first_zero.take() {
                let new_range = first_zero..i;
                if new_range.len() > zeroed_range.len() {
                    zeroed_range = new_range;
                }
            }
        }

        if let Some(first_zero) = first_zero {
            let new_range = first_zero..self.0.len();
            if new_range.len() > zeroed_range.len() {
                zeroed_range = new_range;
            }
        }

        debug_assert!(self.0[zeroed_range.clone()].iter().all(|&ch| ch == 0));
        debug_assert_ne!(zeroed_range.len(), 1);

        // One of the following cases:
        // - ::ffff:<xxxx>:<yyyy> (where xxxx and yyyy can be anything)
        // - ::<xxxx>:<yyyy> (where xxxx is not 0)
        // - ::<xxxx>:<yyyy> (where xxxx is 0 and yyyy isn't either 0 or 1))
        // Embedded IPv4 in xxxx/yyyy
        if (zeroed_range == (0..5) && self.0[5] == 0xffff)
            || zeroed_range == (0..6)
            || (zeroed_range == (0..7) && self.0[7] != 1)
        {
            f.write_str(if self.0[5] == 0xffff { "::ffff:" } else { "::" })?;
            let [a, b] = self.0[6].to_be_bytes();
            let [c, d] = self.0[7].to_be_bytes();
            return fmt::Display::fmt(&Ipv4Octets([a, b, c, d]), f);
        }

        for (i, seg) in self.0.iter().copied().enumerate() {
            if zeroed_range.contains(&i) {
                if i == zeroed_range.start {
                    f.write_str(":")?;
                }
                continue;
            }

            if i != 0 {
                f.write_str(":")?;
            }

            write!(f, "{:x}", seg)?;
        }

        if zeroed_range.contains(&(self.0.len() - 1)) {
            f.write_str(":")?;
        }

        Ok(())
    }
}

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
    Inet4(Inet4Addr),
    Inet6(Inet6Addr),
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

                Ok(Self::Inet4(Inet4Addr::from(unsafe {
                    core::mem::transmute_copy::<_, libc::sockaddr_in>(&storage)
                })))
            }

            libc::AF_INET6 if len >= core::mem::size_of::<libc::sockaddr_in6>() as _ => {
                assert!(
                    core::mem::size_of::<libc::sockaddr_storage>()
                        >= core::mem::size_of::<libc::sockaddr_in>()
                );

                Ok(Self::Inet6(Inet6Addr::from(unsafe {
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

    pub fn unwrap_inet4(self) -> Inet4Addr {
        match self {
            Self::Inet4(addr) => addr,
            _ => panic!("unwrap_inet4() called on a non-inet4 socket"),
        }
    }

    pub fn unwrap_inet6(self) -> Inet6Addr {
        match self {
            Self::Inet6(addr) => addr,
            _ => panic!("unwrap_inet6() called on a non-inet6 socket"),
        }
    }

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

impl From<Inet4Addr> for SockAddr {
    #[inline]
    fn from(a: Inet4Addr) -> Self {
        Self::Inet4(a)
    }
}

impl From<Inet6Addr> for SockAddr {
    #[inline]
    fn from(a: Inet6Addr) -> Self {
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
    fn test_parse_v4_octets() {
        assert_eq!(parse_v4_octets(b"1.1.1.1"), Some([1, 1, 1, 1]));
        assert_eq!(
            parse_v4_octets(b"255.255.255.255"),
            Some([255, 255, 255, 255])
        );

        assert_eq!(parse_v4_octets(b""), None);
        assert_eq!(parse_v4_octets(b"1.1.1."), None);
        assert_eq!(parse_v4_octets(b"1.1.1.1."), None);
        assert_eq!(parse_v4_octets(b"1.1.1.1.1"), None);
        assert_eq!(parse_v4_octets(b"1.1.1.1d"), None);
        assert_eq!(parse_v4_octets(b".1.1.1.1"), None);
        assert_eq!(parse_v4_octets(b":1.1.1.1"), None);
    }

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
        let addr = UnixAddr::new("").unwrap();
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
        let addr = Inet4Addr::new(1, 2, 3, 4, 80);
        assert_eq!(addr.octets(), [1, 2, 3, 4]);
        assert_eq!(addr.port(), 80);

        let mut addr2 = Inet4Addr::new(0, 0, 0, 0, 0);
        addr2.set_octets([1, 2, 3, 4]);
        addr2.set_port(80);

        assert_eq!(addr, addr2);
        assert_eq!(addr.0, addr2.0);
    }

    #[test]
    fn test_inet4addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet4Addr::from_str("0.0.0.0:80").unwrap(),
            Inet4Addr::new(0, 0, 0, 0, 80)
        );
        assert_eq!(
            Inet4Addr::from_str("127.0.0.1:0").unwrap(),
            Inet4Addr::new(127, 0, 0, 1, 0)
        );

        Inet4Addr::from_str("127.0.0.1").unwrap_err();
        Inet4Addr::from_str("127.0.0.1d:80").unwrap_err();
        Inet4Addr::from_str("127.0.0.:80").unwrap_err();
        Inet4Addr::from_str("127.0.0:80").unwrap_err();
        Inet4Addr::from_str("127.0.0.1.1:80").unwrap_err();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_inet4addr_display_debug() {
        assert_eq!(format!("{}", Inet4Addr::new(1, 2, 3, 4, 80)), "1.2.3.4:80");

        assert_eq!(
            format!("{:?}", Inet4Addr::new(1, 2, 3, 4, 80)),
            "Inet4Addr { addr: \"1.2.3.4\", port: 80 }"
        );
    }

    #[test]
    fn test_inet4addr_is() {
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        enum AType {
            Unspec,
            Loopback,
            Private,
            LinkLocal,
            Broadcast,
            Other,
        }

        for (addr, atype) in [
            (Inet4Addr::new(127, 0, 0, 1, 0), AType::Loopback),
            (Inet4Addr::new(0, 0, 0, 0, 0), AType::Unspec),
            (Inet4Addr::new(10, 0, 0, 0, 0), AType::Private),
            (Inet4Addr::new(192, 168, 0, 0, 0), AType::Private),
            (Inet4Addr::new(172, 16, 0, 0, 0), AType::Private),
            (Inet4Addr::new(172, 31, 0, 0, 0), AType::Private),
            (Inet4Addr::new(169, 254, 0, 0, 0), AType::LinkLocal),
            (Inet4Addr::new(255, 255, 255, 255, 0), AType::Broadcast),
            (Inet4Addr::new(172, 15, 0, 0, 0), AType::Other),
            (Inet4Addr::new(172, 32, 0, 0, 0), AType::Other),
        ]
        .iter()
        .copied()
        {
            assert_eq!(addr.is_unspecified(), atype == AType::Unspec, "{:?}", addr);
            assert_eq!(addr.is_loopback(), atype == AType::Loopback, "{:?}", addr);
            assert_eq!(addr.is_private(), atype == AType::Private, "{:?}", addr);
            assert_eq!(
                addr.is_link_local(),
                atype == AType::LinkLocal,
                "{:?}",
                addr
            );
            assert_eq!(addr.is_broadcast(), atype == AType::Broadcast, "{:?}", addr);
        }
    }

    #[test]
    fn test_inet6addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet6Addr::from_str("::1:80").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0)
        );
        assert_eq!(
            Inet6Addr::from_str("[::1]:80").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0)
        );
        assert_eq!(
            Inet6Addr::from_str(":::80").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0)
        );
        assert_eq!(
            Inet6Addr::from_str("1::1:80").unwrap(),
            Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0)
        );
        assert_eq!(
            Inet6Addr::from_str("2001:db8::1:80").unwrap(),
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1, 80, 0, 0)
        );

        assert_eq!(
            Inet6Addr::from_str("[::ffff:192.168.1.2]:80").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 49320, 258, 80, 0, 0)
        );

        assert_eq!(
            Inet6Addr::from_str("[::192.168.1.2]:80").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 49320, 258, 80, 0, 0)
        );

        Inet6Addr::from_str(":1:80").unwrap_err();
        Inet6Addr::from_str("1:::80").unwrap_err();
        Inet6Addr::from_str("1::0:::80").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7::80").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7:::80").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7:8:9:80").unwrap_err();
        Inet6Addr::from_str("::g:80").unwrap_err();
        Inet6Addr::from_str("[::1:80").unwrap_err();
        Inet6Addr::from_str("::1]:80").unwrap_err();
        Inet6Addr::from_str("[::ffff:192.168.1.2:]:80").unwrap_err();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_inet6addr_display_debug() {
        for (addr, s) in [
            // Simple case
            (
                Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8, 80, 0, 0),
                "[1:2:3:4:5:6:7:8]:80",
            ),
            // All zeroes
            (Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0), "[::]:80"),
            // Only one 1 (various positions)
            (Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0), "[::1]:80"),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 1, 0, 0, 80, 0, 0),
                "[::1:0:0]:80",
            ),
            (Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0), "[1::]:80"),
            (
                Inet6Addr::new(0, 1, 0, 0, 0, 0, 0, 0, 80, 0, 0),
                "[0:1::]:80",
            ),
            // The longest string of 0s will be selected for shortening with "::"
            (
                Inet6Addr::new(1, 0, 0, 1, 0, 0, 0, 0, 80, 0, 0),
                "[1:0:0:1::]:80",
            ),
            (
                Inet6Addr::new(1, 0, 0, 1, 0, 0, 0, 1, 80, 0, 0),
                "[1:0:0:1::1]:80",
            ),
            // But if they're the same length, the first one is used
            (
                Inet6Addr::new(1, 0, 0, 1, 0, 0, 1, 0, 80, 0, 0),
                "[1::1:0:0:1:0]:80",
            ),
            // Embedded IPv4
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 65535, 49320, 258, 80, 0, 0),
                "[::ffff:192.168.1.2]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0, 80, 0, 0),
                "[::ffff:0.0.0.0]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1, 80, 0, 0),
                "[::ffff:0.0.0.1]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 49320, 258, 80, 0, 0),
                "[::192.168.1.2]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 258, 80, 0, 0),
                "[::0.0.1.2]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2, 80, 0, 0),
                "[::0.0.0.2]:80",
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 258, 0, 80, 0, 0),
                "[::1.2.0.0]:80",
            ),
        ]
        .iter()
        {
            assert_eq!(format!("{}", addr), *s);
        }

        assert_eq!(
            format!("{:?}", Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8, 80, 0, 0)),
            "Inet6Addr { addr: \"1:2:3:4:5:6:7:8\", port: 80, flowinfo: 0, scope_id: 0 }"
        );
    }

    #[test]
    fn test_inet6addr_is() {
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        enum AType {
            Unspec,
            Loopback,
            Multicast,
            Other,
        }

        for (addr, atype) in [
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0),
                AType::Loopback,
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0),
                AType::Unspec,
            ),
            (
                Inet6Addr::new(65280, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0),
                AType::Multicast,
            ),
            (
                Inet6Addr::new(65280, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0),
                AType::Multicast,
            ),
            (
                Inet6Addr::new(65535, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0),
                AType::Multicast,
            ),
            (
                Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0),
                AType::Other,
            ),
            (
                Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0, 80, 0, 0),
                AType::Other,
            ),
            (
                Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1, 80, 0, 0),
                AType::Other,
            ),
        ]
        .iter()
        .copied()
        {
            assert_eq!(addr.is_unspecified(), atype == AType::Unspec, "{:?}", addr);
            assert_eq!(addr.is_loopback(), atype == AType::Loopback, "{:?}", addr);
            assert_eq!(addr.is_multicast(), atype == AType::Multicast, "{:?}", addr);
        }
    }

    #[test]
    fn test_inet6addr_to_ipv4() {
        assert_eq!(
            Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8, 80, 0, 0).to_ipv4(),
            None
        );
        assert_eq!(
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1, 80, 0, 0).to_ipv4(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 1, 80))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 256, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(1, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 1, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 256, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 1, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 2, 80))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 256, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(1, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 1, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 1, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 256, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 1, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 2, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 2, 80))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1, 80, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 1, 80))
        );
    }

    #[test]
    fn test_inet6addr_to_ipv4_mapped() {
        assert_eq!(
            Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8, 80, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1, 80, 0, 0).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 256, 0, 80, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0, 80, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 256, 80, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2, 80, 0, 0).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 256, 0, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(1, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 1, 0, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 1, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 256, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 1, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 2, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 2, 80))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 0, 80))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1, 80, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 1, 80))
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
