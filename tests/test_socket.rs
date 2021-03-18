#![cfg(feature = "std")]

use std::io::prelude::*;

use slibc::{SockAddr, SockDomain, SockType, Socket, UnixAddr};

#[test]
fn test_socket_bind_connect_accept() {
    let tmpdir = tempfile::tempdir().unwrap();
    let tmpdir = tmpdir.as_ref();

    let addr = SockAddr::Unix(UnixAddr::new(&tmpdir.join("sock")).unwrap());

    let bsock = Socket::new_cloexec(SockDomain::UNIX, SockType::STREAM, None).unwrap();
    assert!(bsock.as_ref().get_cloexec().unwrap());

    bsock.bind(&addr).unwrap();
    bsock.listen(3).unwrap();

    let mut csock = Socket::new_cloexec(SockDomain::UNIX, SockType::STREAM, None).unwrap();
    assert!(csock.as_ref().get_cloexec().unwrap());
    csock.connect(&addr).unwrap();

    let (mut asock, aaddr) = bsock.accept_cloexec().unwrap();
    assert!(asock.as_ref().get_cloexec().unwrap());

    let aaddr = aaddr.unwrap_unix();
    assert!(aaddr.is_unnamed());
    assert_eq!(aaddr.path(), None);
    #[cfg(linuxlike)]
    assert_eq!(aaddr.abstract_name(), None);

    assert_eq!(
        csock.getpeername().unwrap().unwrap_unix(),
        addr.unwrap_unix()
    );
    assert_eq!(
        bsock.getsockname().unwrap().unwrap_unix(),
        addr.unwrap_unix()
    );
    assert_eq!(
        asock.getsockname().unwrap().unwrap_unix(),
        addr.unwrap_unix()
    );

    let mut buf = [0; 100];

    csock.write_all(b"abc").unwrap();
    asock.read_exact(&mut buf[..3]).unwrap();
    assert_eq!(&buf[..3], b"abc");
}
