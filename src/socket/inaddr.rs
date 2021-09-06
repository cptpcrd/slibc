use crate::internal_prelude::*;

use core::fmt;

use super::AddrParseError;

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

#[derive(Copy, Clone)]
pub struct Inet4Addr(pub(crate) libc::in_addr);

impl Inet4Addr {
    /// An IP address referring to localhost; i.e. `127.0.0.1`.
    pub const LOCALHOST: Self = Self::new(127, 0, 0, 1);
    /// An IP address representing an unspecified address; i.e. `0.0.0.0`.
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0);
    /// An IP address representing the broadcast address; i.e. `255.255.255.255`.
    pub const BROADCAST: Self = Self::new(255, 255, 255, 255);

    /// Create a new Ipv4 address from the given octets.
    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self(libc::in_addr {
            s_addr: u32::from_ne_bytes([a, b, c, d]),
        })
    }

    /// Get the four octets that make up this address
    #[inline]
    pub const fn octets(&self) -> [u8; 4] {
        self.0.s_addr.to_ne_bytes()
    }

    /// Check whether the address portion of this socket address represents the "unspecified"
    /// address (`0.0.0.0`).
    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        matches!(self.octets(), [0, 0, 0, 0])
    }

    /// Check whether this address represents a loopback address (`127.0.0.1/8`).
    #[inline]
    pub const fn is_loopback(&self) -> bool {
        self.octets()[0] == 127
    }

    /// Check whether this address represents a private address (`10.0.0.0/8`, `172.16.0.0/12`, or
    /// `192.168.0.0/16`).
    #[inline]
    pub const fn is_private(&self) -> bool {
        match self.octets() {
            [10, ..] => true,
            [192, 168, ..] => true,
            [172, x, ..] if x & 0xF0 == 16 => true,
            _ => false,
        }
    }

    /// Check whether this address represents a link-local address (`169.254.0.0/16`).
    #[inline]
    pub const fn is_link_local(&self) -> bool {
        matches!(self.octets(), [169, 254, ..])
    }

    /// Check whether this address represents the broadcast address (`255.255.255.255`).
    #[inline]
    pub const fn is_broadcast(&self) -> bool {
        matches!(self.octets(), [255, 255, 255, 255])
    }

    /// Convert this IPv4 address to an IPv6 address of the form `::ffff:a.b.c.d`.
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub const fn to_ipv6_mapped(&self) -> Inet6Addr {
        let octets = self.octets();
        Inet6Addr::new(
            0,
            0,
            0,
            0,
            0,
            65535,
            u16::from_be_bytes([octets[0], octets[1]]),
            u16::from_be_bytes([octets[2], octets[3]]),
        )
    }
}

impl AsRef<libc::in_addr> for Inet4Addr {
    #[inline]
    fn as_ref(&self) -> &libc::in_addr {
        &self.0
    }
}

impl From<libc::in_addr> for Inet4Addr {
    #[inline]
    fn from(a: libc::in_addr) -> Self {
        Self(a)
    }
}

impl PartialEq for Inet4Addr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.octets() == other.octets()
    }
}

impl Eq for Inet4Addr {}

impl core::str::FromStr for Inet4Addr {
    type Err = AddrParseError;

    #[allow(clippy::many_single_char_names)]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let [a, b, c, d] = parse_v4_octets(s.as_bytes()).ok_or(AddrParseError(()))?;
        Ok(Self::new(a, b, c, d))
    }
}

impl fmt::Debug for Inet4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl fmt::Display for Inet4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let octets = self.octets();
        write!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

#[derive(Copy, Clone)]
pub struct Inet6Addr(pub(crate) libc::in6_addr);

impl Inet6Addr {
    /// An IP address referring to localhost; i.e. `::1`.
    pub const LOCALHOST: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 1);
    /// An IP address representing an unspecified address; i.e. `::`.
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 0);

    /// Create a new Ipv6 address from eight 16-bit segments.
    #[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        let [a1, a2] = a.to_be_bytes();
        let [b1, b2] = b.to_be_bytes();
        let [c1, c2] = c.to_be_bytes();
        let [d1, d2] = d.to_be_bytes();
        let [e1, e2] = e.to_be_bytes();
        let [f1, f2] = f.to_be_bytes();
        let [g1, g2] = g.to_be_bytes();
        let [h1, h2] = h.to_be_bytes();

        Self(libc::in6_addr {
            s6_addr: [
                a1, a2, b1, b2, c1, c2, d1, d2, e1, e2, f1, f2, g1, g2, h1, h2,
            ],
        })
    }

    /// Get the eight 16-bit segments that represent this Ipv6 address.
    #[inline]
    pub const fn segments(&self) -> [u16; 8] {
        let [a1, a2, b1, b2, c1, c2, d1, d2, e1, e2, f1, f2, g1, g2, h1, h2] = self.0.s6_addr;

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

    /// Get the sixteen octets that represent this IPv6 address.
    #[inline]
    pub const fn octets(&self) -> [u8; 16] {
        self.0.s6_addr
    }

    /// Check whether this address represents the "unspecified" address (`::`).
    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        matches!(
            self.octets(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        )
    }

    /// Check whether this address represents a loopback address (`::1`).
    #[inline]
    pub const fn is_loopback(&self) -> bool {
        matches!(
            self.octets(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        )
    }

    /// Check whether this address represents a multicast address (`ff00::/8`).
    #[inline]
    pub const fn is_multicast(&self) -> bool {
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
    ///     Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_ipv4(),
    ///     Some(Inet4Addr::new(0, 0, 0, 1)),
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub const fn to_ipv4(&self) -> Option<Inet4Addr> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, a, b, c, d] => {
                Some(Inet4Addr::new(a, b, c, d))
            }
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d] => Some(Inet4Addr::new(a, b, c, d)),
            _ => None,
        }
    }

    /// If this address is of the form `::ffff:a.b.c.d`, return the IPv4 version (i.e. `a.b.c.d`).
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub const fn to_ipv4_mapped(&self) -> Option<Inet4Addr> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, a, b, c, d] => {
                Some(Inet4Addr::new(a, b, c, d))
            }
            _ => None,
        }
    }
}

impl AsRef<libc::in6_addr> for Inet6Addr {
    #[inline]
    fn as_ref(&self) -> &libc::in6_addr {
        &self.0
    }
}
impl From<libc::in6_addr> for Inet6Addr {
    #[inline]
    fn from(a: libc::in6_addr) -> Self {
        Self(a)
    }
}

impl PartialEq for Inet6Addr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.octets() == other.octets()
    }
}

impl Eq for Inet6Addr {}

impl core::str::FromStr for Inet6Addr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        fn parse_addr(mut s: &[u8]) -> Option<Inet6Addr> {
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
            ))
        }

        parse_addr(s.as_bytes()).ok_or(AddrParseError(()))
    }
}

impl fmt::Debug for Inet6Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl fmt::Display for Inet6Addr {
    #[allow(clippy::many_single_char_names)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let segments = self.segments();

        let mut zeroed_range = 0..0;
        let mut first_zero = None;

        for (i, seg) in segments.iter().copied().enumerate() {
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
            let new_range = first_zero..segments.len();
            if new_range.len() > zeroed_range.len() {
                zeroed_range = new_range;
            }
        }

        debug_assert!(segments[zeroed_range.clone()].iter().all(|&ch| ch == 0));
        debug_assert_ne!(zeroed_range.len(), 1);

        // One of the following cases:
        // - ::ffff:<xxxx>:<yyyy> (where xxxx and yyyy can be anything)
        // - ::<xxxx>:<yyyy> (where xxxx is not 0)
        // - ::<xxxx>:<yyyy> (where xxxx is 0 and yyyy isn't either 0 or 1))
        // Embedded IPv4 in xxxx/yyyy
        if (zeroed_range == (0..5) && segments[5] == 0xffff)
            || zeroed_range == (0..6)
            || (zeroed_range == (0..7) && segments[7] != 1)
        {
            f.write_str(if segments[5] == 0xffff {
                "::ffff:"
            } else {
                "::"
            })?;
            let [a, b] = segments[6].to_be_bytes();
            let [c, d] = segments[7].to_be_bytes();
            return fmt::Display::fmt(&Inet4Addr::new(a, b, c, d), f);
        }

        for (i, seg) in segments.iter().copied().enumerate() {
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

        if zeroed_range.contains(&(segments.len() - 1)) {
            f.write_str(":")?;
        }

        Ok(())
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
    fn test_inet4addr() {
        let addr = Inet4Addr::new(1, 2, 3, 4);
        assert_eq!(addr.octets(), [1, 2, 3, 4]);
        let addr2 = Inet4Addr::new(1, 2, 3, 4);

        assert_eq!(addr, addr2);
        assert_eq!(addr.0, addr2.0);
    }

    #[test]
    fn test_inet4addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet4Addr::from_str("0.0.0.0").unwrap(),
            Inet4Addr::new(0, 0, 0, 0)
        );
        assert_eq!(
            Inet4Addr::from_str("127.0.0.1").unwrap(),
            Inet4Addr::new(127, 0, 0, 1)
        );

        Inet4Addr::from_str("127.0.0.1d").unwrap_err();
        Inet4Addr::from_str("127.0.0.").unwrap_err();
        Inet4Addr::from_str("127.0.0").unwrap_err();
        Inet4Addr::from_str("127.0.0.1.1").unwrap_err();
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_inet4addr_display_debug() {
        assert_eq!(format!("{}", Inet4Addr::new(1, 2, 3, 4)), "1.2.3.4");

        assert_eq!(format!("{:?}", Inet4Addr::new(1, 2, 3, 4)), "\"1.2.3.4\"");
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
            (Inet4Addr::new(127, 0, 0, 1), AType::Loopback),
            (Inet4Addr::new(0, 0, 0, 0), AType::Unspec),
            (Inet4Addr::new(10, 0, 0, 0), AType::Private),
            (Inet4Addr::new(192, 168, 0, 0), AType::Private),
            (Inet4Addr::new(172, 16, 0, 0), AType::Private),
            (Inet4Addr::new(172, 31, 0, 0), AType::Private),
            (Inet4Addr::new(169, 254, 0, 0), AType::LinkLocal),
            (Inet4Addr::new(255, 255, 255, 255), AType::Broadcast),
            (Inet4Addr::new(172, 15, 0, 0), AType::Other),
            (Inet4Addr::new(172, 32, 0, 0), AType::Other),
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
    fn test_inet4addr_tofrom() {
        for addr in [
            Inet4Addr::UNSPECIFIED,
            Inet4Addr::LOCALHOST,
            Inet4Addr::BROADCAST,
            Inet4Addr::new(1, 2, 3, 4),
        ]
        .iter()
        .copied()
        {
            assert_eq!(Inet4Addr::from(*addr.as_ref()), addr);
        }
    }

    #[test]
    fn test_inet4addr_to_ipv6_mapped() {
        for addr in [
            Inet4Addr::UNSPECIFIED,
            Inet4Addr::LOCALHOST,
            Inet4Addr::BROADCAST,
            Inet4Addr::new(1, 2, 3, 4),
        ]
        .iter()
        .copied()
        {
            assert_eq!(addr.to_ipv6_mapped().to_ipv4_mapped(), Some(addr));
        }
    }

    #[test]
    fn test_inet6addr_parse() {
        use core::str::FromStr;

        assert_eq!(
            Inet6Addr::from_str("::1").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(
            Inet6Addr::from_str("::1").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(
            Inet6Addr::from_str("::").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
        );
        assert_eq!(
            Inet6Addr::from_str("1::1").unwrap(),
            Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(
            Inet6Addr::from_str("2001:db8::1").unwrap(),
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1)
        );

        assert_eq!(
            Inet6Addr::from_str("::ffff:192.168.1.2").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 49320, 258)
        );

        assert_eq!(
            Inet6Addr::from_str("::192.168.1.2").unwrap(),
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 49320, 258)
        );

        Inet6Addr::from_str(":1").unwrap_err();
        Inet6Addr::from_str("[::1]").unwrap_err();
        Inet6Addr::from_str("1::").unwrap_err();
        Inet6Addr::from_str("1::0::").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7:").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7::").unwrap_err();
        Inet6Addr::from_str("1:2:3:4:5:6:7:8:9").unwrap_err();
        Inet6Addr::from_str("::g").unwrap_err();
        Inet6Addr::from_str("::ffff:192.168.1.2:").unwrap_err();
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
            (Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), AType::Loopback),
            (Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), AType::Unspec),
            (Inet6Addr::new(65280, 0, 0, 0, 0, 0, 0, 0), AType::Multicast),
            (Inet6Addr::new(65280, 0, 0, 0, 0, 0, 0, 0), AType::Multicast),
            (Inet6Addr::new(65535, 0, 0, 0, 0, 0, 0, 1), AType::Multicast),
            (Inet6Addr::new(1, 0, 0, 0, 0, 0, 0, 0), AType::Other),
            (Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0), AType::Other),
            (Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1), AType::Other),
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
        assert_eq!(Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8).to_ipv4(), None);
        assert_eq!(Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1).to_ipv4(), None);

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 1))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 256, 0).to_ipv4(),
            Some(Inet4Addr::new(1, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 1, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 256).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 1, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 2))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 256, 0).to_ipv4(),
            Some(Inet4Addr::new(1, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 1, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 1, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 256).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 1, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 2).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 2))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1).to_ipv4(),
            Some(Inet4Addr::new(0, 0, 0, 1))
        );
    }

    #[test]
    fn test_inet6addr_to_ipv4_mapped() {
        assert_eq!(
            Inet6Addr::new(1, 2, 3, 4, 5, 6, 7, 8).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 256, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 1, 0).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 256).to_ipv4_mapped(),
            None
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 0, 0, 2).to_ipv4_mapped(),
            None
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 256, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(1, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 1, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 1, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 256).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 1, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 2).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 2))
        );

        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 0).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 0))
        );
        assert_eq!(
            Inet6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1).to_ipv4_mapped(),
            Some(Inet4Addr::new(0, 0, 0, 1))
        );
    }

    #[test]
    fn test_inet6addr_tofrom() {
        for addr in [
            Inet6Addr::UNSPECIFIED,
            Inet6Addr::LOCALHOST,
            Inet6Addr::new(8193, 3512, 0, 0, 0, 0, 0, 1),
        ]
        .iter()
        .copied()
        {
            assert_eq!(Inet6Addr::from(*addr.as_ref()), addr);
        }
    }
}
