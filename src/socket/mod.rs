use crate::internal_prelude::*;

use core::fmt;

mod inaddr;
mod sockaddr;
pub use inaddr::*;
pub use sockaddr::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddrParseError(());

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid address")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AddrParseError {}

macro_rules! define_enum {
    (
        $ename:ident,
        $repr:ident,
        $ty:path,
        $(
            #[cfg($cfg:meta)]
            $(
                $(#[doc = $doc:literal])*
                $name:ident = $libc_name:ident,
            )+
        )+
    ) => {
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        #[repr($repr)]
        pub enum $ename {
            $($(
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
                #[cfg($cfg)]
                $name = libc::$libc_name as _,
            )+)+
        }

        impl $ename {
            #[allow(dead_code)]
            fn from_raw(raw: $ty) -> Self {
                match raw as _ {
                    $($(
                        #[cfg($cfg)]
                        libc::$libc_name => Self::$name,
                    )+)+
                    _ => unreachable!(),
                }
            }
        }
    };
}

define_enum! {
    SockDomain,
    u32,
    libc::sa_family_t,
    #[cfg(all())]
    INET = AF_INET,
    INET6 = AF_INET6,
    UNIX = AF_UNIX,
    UNSPEC = AF_UNSPEC,
}

define_enum! {
    SockType,
    i32,
    libc::c_int,
    #[cfg(all())]
    STREAM = SOCK_STREAM,
    DGRAM = SOCK_DGRAM,
    RAW = SOCK_RAW,
    SEQPACKET = SOCK_SEQPACKET,
}

define_enum! {
    Shutdown,
    i32,
    libc::c_int,
    #[cfg(all())]
    RD = SHUT_RD,
    WR = SHUT_WR,
    RDWR = SHUT_RDWR,
}

define_enum! {
    SockProto,
    i32,
    libc::c_int,
    #[cfg(all())]
    TCP = IPPROTO_TCP,
    UDP = IPPROTO_UDP,
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    UDPLITE = IPPROTO_UDPLITE,
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    )))
)]
#[cfg(any(linuxlike, freebsdlike, netbsdlike))]
bitflags::bitflags! {
    #[derive(Default)]
    pub struct SockFlag: libc::c_int {
        const CLOEXEC = libc::SOCK_CLOEXEC;
        const NONBLOCK = libc::SOCK_NONBLOCK;
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct MsgFlag: libc::c_int {
        const EOR = libc::MSG_EOR;
        const OOB = libc::MSG_OOB;
        #[cfg_attr(docsrs, doc(cfg(not(any(target_os = "macos", target_os = "ios")))))]
        #[cfg(not(apple))]
        const NOSIGNAL = libc::MSG_NOSIGNAL;
    }
}

#[derive(Debug)]
pub struct Socket(FileDesc);

impl Socket {
    #[inline]
    fn new_imp(domain: libc::c_int, stype: libc::c_int, proto: libc::c_int) -> Result<Self> {
        Ok(Self(unsafe {
            Error::unpack_fdesc(libc::socket(domain, stype, proto))?
        }))
    }

    /// Create a new socket.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slibc::{Socket, SockDomain, SockType};
    /// // IPv4 TCP socket
    /// Socket::new(SockDomain::INET, SockType::STREAM, None).unwrap();
    /// // IPv6 UDP socket
    /// Socket::new(SockDomain::INET6, SockType::DGRAM, None).unwrap();
    /// // UNIX-domain stream socket
    /// Socket::new(SockDomain::UNIX, SockType::STREAM, None).unwrap();
    /// ```
    #[inline]
    pub fn new(domain: SockDomain, stype: SockType, proto: Option<SockProto>) -> Result<Self> {
        Self::new_imp(domain as _, stype as _, proto.map_or(0, |p| p as _))
    }

    /// Identical to [`Self::new()`], but allows passing flags that modify behavior.
    ///
    /// See [`SockFlag`] for a list of flags that can be passed.
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )))
    )]
    #[cfg(any(linuxlike, freebsdlike, netbsdlike))]
    #[inline]
    pub fn new_flags(
        domain: SockDomain,
        stype: SockType,
        flags: SockFlag,
        proto: Option<SockProto>,
    ) -> Result<Self> {
        Self::new_imp(
            domain as _,
            stype as libc::c_int | flags.bits(),
            proto.map_or(0, |p| p as _),
        )
    }

    /// Identical to [`Self::new()`], but sets the close-on-exec flag on the new socket.
    ///
    /// On platforms that support [`Self::new_flags()`], this passes the [`SockFlag::CLOEXEC`]
    /// flag to set the close-on-exec flag atomically. On other platforms, it creates a new socket
    /// with [`Self::new()`] and immediately sets the close-on-exec flag.
    #[inline]
    pub fn new_cloexec(
        domain: SockDomain,
        stype: SockType,
        proto: Option<SockProto>,
    ) -> Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(any(linuxlike, freebsdlike, netbsdlike))] {
                let sock = Self::new_flags(domain, stype, SockFlag::CLOEXEC, proto)?;
            } else {
                let sock = Self::new(domain, stype, proto)?;
                sock.0.set_cloexec(true)?;
            }
        }

        Ok(sock)
    }

    #[inline]
    fn pair_imp(
        domain: libc::c_int,
        stype: libc::c_int,
        proto: libc::c_int,
    ) -> Result<(Self, Self)> {
        unsafe {
            let mut fds = [0; 2];
            Error::unpack_nz(libc::socketpair(domain, stype, proto, fds.as_mut_ptr()))?;

            let fdesc1 = FileDesc::new(fds[0]);
            let fdesc2 = FileDesc::new(fds[1]);

            Ok((Self(fdesc1), Self(fdesc2)))
        }
    }

    /// Create an unbound pair of connected sockets.
    ///
    /// See `socketpair(2)` for more information.
    #[inline]
    pub fn pair(
        domain: SockDomain,
        stype: SockType,
        proto: Option<SockProto>,
    ) -> Result<(Self, Self)> {
        Self::pair_imp(domain as _, stype as _, proto.map_or(0, |p| p as _))
    }

    /// Identical to [`Self::pair()`], but allows passing flags that modify behavior.
    ///
    /// See [`SockFlag`] for a list of flags that can be passed.
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )))
    )]
    #[cfg(any(linuxlike, freebsdlike, netbsdlike))]
    #[inline]
    pub fn pair_flags(
        domain: SockDomain,
        stype: SockType,
        flags: SockFlag,
        proto: Option<SockProto>,
    ) -> Result<(Self, Self)> {
        Self::pair_imp(
            domain as _,
            stype as libc::c_int | flags.bits(),
            proto.map_or(0, |p| p as _),
        )
    }

    /// Identical to [`Self::pair()`], but sets the close-on-exec flag on the new socket.
    ///
    /// This is analagous to [`Self::new_cloexec()`].
    #[inline]
    pub fn pair_cloexec(
        domain: SockDomain,
        stype: SockType,
        proto: Option<SockProto>,
    ) -> Result<(Self, Self)> {
        cfg_if::cfg_if! {
            if #[cfg(any(linuxlike, freebsdlike, netbsdlike))] {
                let (sock1, sock2) = Self::pair_flags(domain, stype, SockFlag::CLOEXEC, proto)?;
            } else {
                let (sock1, sock2) = Self::pair(domain, stype, proto)?;
                sock1.0.set_cloexec(true)?;
                sock2.0.set_cloexec(true)?;
            }
        }

        Ok((sock1, sock2))
    }

    #[inline]
    pub fn bind(&self, addr: &SockAddr) -> Result<()> {
        let (addr, addrlen) = addr.as_raw();
        Error::unpack_nz(unsafe { libc::bind(self.0.fd(), addr, addrlen) })
    }

    #[inline]
    pub fn connect(&self, addr: &SockAddr) -> Result<()> {
        let (addr, addrlen) = addr.as_raw();
        Error::unpack_nz(unsafe { libc::connect(self.0.fd(), addr, addrlen) })
    }

    #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
    #[cfg(target_os = "freebsd")]
    #[inline]
    pub fn bindat(&self, fd: RawFd, addr: &UnixAddr) -> Result<()> {
        let (addr, addrlen) = addr.as_raw();
        Error::unpack_nz(unsafe { sys::bindat(fd, self.0.fd(), addr, addrlen) })
    }

    #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
    #[cfg(target_os = "freebsd")]
    #[inline]
    pub fn connectat(&self, fd: RawFd, addr: &UnixAddr) -> Result<()> {
        let (addr, addrlen) = addr.as_raw();
        Error::unpack_nz(unsafe { sys::connectat(fd, self.0.fd(), addr, addrlen) })
    }

    #[inline]
    pub fn listen(&self, backlog: i32) -> Result<()> {
        Error::unpack_nz(unsafe { libc::listen(self.0.fd(), backlog) })
    }

    #[inline]
    pub fn accept(&self) -> Result<(Self, SockAddr)> {
        let mut addr = MaybeUninit::zeroed();
        let mut addrlen = core::mem::size_of::<libc::sockaddr_storage>() as _;

        let fdesc = unsafe {
            Error::unpack_fdesc(libc::accept(
                self.0.fd(),
                addr.as_mut_ptr() as *mut _,
                &mut addrlen,
            ))?
        };

        Ok((
            Self(fdesc),
            SockAddr::from_raw(unsafe { addr.assume_init() }, addrlen)?,
        ))
    }

    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )))
    )]
    #[cfg(any(linuxlike, freebsdlike, netbsdlike))]
    #[inline]
    pub fn accept4(&self, flags: SockFlag) -> Result<(Self, SockAddr)> {
        let mut addr = MaybeUninit::zeroed();
        let mut addrlen = core::mem::size_of::<libc::sockaddr_storage>() as _;
        let fdesc = unsafe {
            Error::unpack_fdesc(libc::accept4(
                self.0.fd(),
                addr.as_mut_ptr() as *mut _,
                &mut addrlen,
                flags.bits(),
            ))?
        };

        Ok((
            Self(fdesc),
            SockAddr::from_raw(unsafe { addr.assume_init() }, addrlen)?,
        ))
    }

    #[inline]
    pub fn accept_cloexec(&self) -> Result<(Self, SockAddr)> {
        cfg_if::cfg_if! {
            if #[cfg(any(linuxlike, freebsdlike, netbsdlike))] {
                let (sock, addr) = self.accept4(SockFlag::CLOEXEC)?;
            } else {
                let (sock, addr) = self.accept()?;
                sock.0.set_cloexec(true)?;
            }
        }

        Ok((sock, addr))
    }

    #[inline]
    pub fn getsockname(&self) -> Result<SockAddr> {
        let mut addr = MaybeUninit::zeroed();
        let mut addrlen = core::mem::size_of::<libc::sockaddr_storage>() as _;

        Error::unpack_nz(unsafe {
            libc::getsockname(self.0.fd(), addr.as_mut_ptr() as *mut _, &mut addrlen)
        })?;

        SockAddr::from_raw(unsafe { addr.assume_init() }, addrlen)
    }

    #[inline]
    pub fn getpeername(&self) -> Result<SockAddr> {
        let mut addr = MaybeUninit::zeroed();
        let mut addrlen = core::mem::size_of::<libc::sockaddr_storage>() as _;

        Error::unpack_nz(unsafe {
            libc::getpeername(self.0.fd(), addr.as_mut_ptr() as *mut _, &mut addrlen)
        })?;

        SockAddr::from_raw(unsafe { addr.assume_init() }, addrlen)
    }

    #[inline]
    pub fn shutdown(&self, how: Shutdown) -> Result<()> {
        Error::unpack_nz(unsafe { libc::shutdown(self.0.fd(), how as _) })
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }

    #[inline]
    pub fn send(&self, buf: &[u8], flags: MsgFlag) -> Result<usize> {
        Error::unpack_size(unsafe {
            libc::send(
                self.0.fd(),
                buf.as_ptr() as *const _,
                buf.len(),
                flags.bits(),
            )
        })
    }

    #[inline]
    pub fn recv(&self, buf: &mut [u8], flags: MsgFlag) -> Result<usize> {
        Error::unpack_size(unsafe {
            libc::recv(
                self.0.fd(),
                buf.as_mut_ptr() as *mut _,
                buf.len(),
                flags.bits(),
            )
        })
    }

    #[inline]
    pub fn sendto(&self, buf: &[u8], flags: MsgFlag, addr: &SockAddr) -> Result<usize> {
        let (addr, addrlen) = addr.as_raw();
        Error::unpack_size(unsafe {
            libc::sendto(
                self.0.fd(),
                buf.as_ptr() as *const _,
                buf.len(),
                flags.bits(),
                addr,
                addrlen,
            )
        })
    }

    #[inline]
    pub fn recvfrom(&self, buf: &mut [u8], flags: MsgFlag) -> Result<(usize, Option<SockAddr>)> {
        let mut addr = MaybeUninit::zeroed();
        let mut addrlen = core::mem::size_of::<libc::sockaddr_storage>() as _;

        let n = Error::unpack_size(unsafe {
            libc::recvfrom(
                self.0.fd(),
                buf.as_mut_ptr() as *mut _,
                buf.len(),
                flags.bits(),
                addr.as_mut_ptr() as *mut _,
                &mut addrlen,
            )
        })?;

        let addr = if addrlen == 0 {
            None
        } else {
            Some(SockAddr::from_raw(unsafe { addr.assume_init() }, addrlen)?)
        };

        Ok((n, addr))
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `Socket` wrapper around a raw file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must be valid and not in use elsewhere.
    #[inline]
    pub unsafe fn from_fd(&self, fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }

    /// Get the UID and GID of the peer connected to this socket.
    ///
    /// The credentials returned are the credentials from the time that `connect()` or `listen()`
    /// was called.
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "macos",
            target_os = "ios",
        )))
    )]
    #[cfg(bsd)]
    #[inline]
    pub fn getpeereid(&self) -> Result<(u32, u32)> {
        crate::getpeereid(self.0.fd())
    }

    /// Get the value of the given socket argument.
    ///
    /// This is a helper that calls `getsockopt(2)` with the given `level` and `name`. The value of
    /// the option is stored in `buf`, and its length is returned by the function.
    ///
    /// # Safety
    ///
    /// `T` must be the correct type for representing the specified option.
    #[inline]
    pub unsafe fn getsockopt_raw<T>(&self, level: i32, name: i32, buf: &mut [T]) -> Result<usize> {
        let mut len = (buf.len() * core::mem::size_of::<T>()) as _;
        Error::unpack_nz(libc::getsockopt(
            self.0.fd(),
            level,
            name,
            buf.as_mut_ptr() as *mut _ as *mut _,
            &mut len,
        ))?;
        Ok(len as _)
    }

    /// Set the value of the given socket argument.
    ///
    /// This is a helper that calls `setsockopt(2)` with the given `level` and `name`. The value of
    /// the option to be stored is specified in `buf`
    ///
    /// # Safety
    ///
    /// `T` must be the correct type for representing the specified option.
    #[inline]
    pub unsafe fn setsockopt_raw<T>(&self, level: i32, name: i32, value: &[T]) -> Result<()> {
        Error::unpack_nz(libc::setsockopt(
            self.0.fd(),
            level,
            name,
            value.as_ptr() as *const _ as *const _,
            (value.len() * core::mem::size_of::<T>()) as _,
        ))
    }

    #[inline]
    pub fn get_nonblocking(&self) -> Result<bool> {
        self.0.get_nonblocking()
    }

    #[inline]
    pub fn set_nonblocking(&self, nonblock: bool) -> Result<()> {
        self.0.set_nonblocking(nonblock)
    }
}

#[cfg(feature = "std")]
impl std::io::Read for Socket {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Socket {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
}

#[cfg(feature = "std")]
impl std::os::unix::io::FromRawFd for Socket {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

#[cfg(feature = "std")]
impl std::os::unix::io::AsRawFd for Socket {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd()
    }
}

#[cfg(feature = "std")]
impl std::os::unix::io::IntoRawFd for Socket {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_fd()
    }
}

impl AsRef<BorrowedFd> for Socket {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socketpair_rw() {
        fn write_all(sock: &Socket, mut data: &[u8]) -> Result<()> {
            while !data.is_empty() {
                let n = sock.write(data)?;
                data = &data[n..];
            }
            Ok(())
        }

        fn read_exact(sock: &Socket, mut data: &mut [u8]) -> Result<()> {
            while !data.is_empty() {
                let n = sock.read(data)?;
                data = &mut data[n..];
            }
            Ok(())
        }

        let mut buf = [0; 100];

        let (a, b) = Socket::pair_cloexec(SockDomain::UNIX, SockType::STREAM, None).unwrap();
        assert!(a.as_ref().get_cloexec().unwrap());
        assert!(b.as_ref().get_cloexec().unwrap());

        write_all(&a, b"abc").unwrap();
        read_exact(&b, &mut buf[..3]).unwrap();
        assert_eq!(&buf[..3], b"abc");

        #[cfg(bsd)]
        assert_eq!(
            a.getpeereid().unwrap(),
            (crate::geteuid(), crate::getegid())
        );
    }
}
