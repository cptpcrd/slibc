use slibc::ffi::prelude::*;
use slibc::ffi::OsStr;
use slibc::{gethostname, uname};

#[cfg(target_os = "linux")]
use slibc::getdomainname;

#[test]
fn test_uname() {
    let utsname = uname().unwrap();

    let mut buf = [0; 4096];

    let hostname = gethostname(&mut buf).unwrap();
    assert_eq!(utsname.nodename(), OsStr::from_bytes(hostname.to_bytes()));

    #[cfg(target_os = "linux")]
    {
        let domainname = getdomainname(&mut buf).unwrap();
        assert_eq!(
            utsname.domainname(),
            OsStr::from_bytes(domainname.to_bytes())
        );
    }
}
