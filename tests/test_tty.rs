use slibc::{isatty, isatty_raw, openpty, pipe, ptsname, ttyname, ttyname_r};

#[test]
fn test_tty() {
    let r = pipe().unwrap().0;
    let (master, slave) = unsafe { openpty(None) }.unwrap();

    let mut buf1 = [0; 4096];

    // Check isatty()

    assert!(!r.isatty().unwrap());
    assert_eq!(isatty_raw(r.fd()).unwrap_err().code(), libc::ENOTTY);

    assert!(master.isatty().unwrap());
    assert!(slave.isatty().unwrap());
    isatty_raw(master.fd()).unwrap();
    isatty_raw(slave.fd()).unwrap();

    assert_eq!(isatty(libc::c_int::MAX).unwrap_err().code(), libc::EBADF);
    assert_eq!(
        isatty_raw(libc::c_int::MAX).unwrap_err().code(),
        libc::EBADF
    );

    // Check ttyname() and ttyname_r()

    assert_eq!(
        ttyname_r(r.fd(), &mut buf1).unwrap_err().code(),
        libc::ENOTTY
    );
    unsafe {
        assert_eq!(ttyname(r.fd()).unwrap_err().code(), libc::ENOTTY);
    }

    let master_name = ttyname_r(master.fd(), &mut buf1).unwrap();
    unsafe {
        assert_eq!(ttyname(master.fd()).unwrap(), master_name);
    }

    let slave_name = ttyname_r(slave.fd(), &mut buf1).unwrap();
    unsafe {
        assert_eq!(ttyname(slave.fd()).unwrap(), slave_name);
    }

    // ptsname(master) should match ttyname(slave)

    #[cfg(target_os = "linux")]
    {
        let mut buf2 = [0; 4096];
        assert_eq!(
            slibc::ptsname_r(master.fd(), &mut buf2).unwrap(),
            slave_name
        );
    }

    unsafe {
        assert_eq!(ptsname(master.fd()).unwrap(), slave_name);
    }

    // ptsname() should fail on the pipe, and on bad file descriptors
    #[cfg(target_os = "linux")]
    assert_eq!(
        slibc::ptsname_r(r.fd(), &mut buf1).unwrap_err().code(),
        libc::ENOTTY
    );

    unsafe {
        assert!(matches!(
            ptsname(r.fd()).unwrap_err().code(),
            libc::ENOTTY | libc::EINVAL
        ));
    }
}
