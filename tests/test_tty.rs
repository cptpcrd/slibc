use slibc::{
    ioctl_getwinsz, ioctl_setwinsz, isatty, isatty_raw, openpty, pipe, ptsname, ttyname, ttyname_r,
    Winsize,
};

#[cfg(all(target_os = "linux", feature = "alloc"))]
use slibc::ptsname_alloc;
#[cfg(feature = "alloc")]
use slibc::ttyname_alloc;

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

    #[cfg(feature = "alloc")]
    assert_eq!(ttyname_alloc(r.fd()).unwrap_err().code(), libc::ENOTTY);

    let master_name = ttyname_r(master.fd(), &mut buf1).unwrap();
    unsafe {
        assert_eq!(ttyname(master.fd()).unwrap(), master_name);
    }

    #[cfg(feature = "alloc")]
    assert_eq!(ttyname_alloc(master.fd()).unwrap().as_c_str(), master_name);

    let slave_name = ttyname_r(slave.fd(), &mut buf1).unwrap();
    unsafe {
        assert_eq!(ttyname(slave.fd()).unwrap(), slave_name);
    }

    #[cfg(feature = "alloc")]
    assert_eq!(ttyname_alloc(slave.fd()).unwrap().as_c_str(), slave_name);

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

    #[cfg(all(target_os = "linux", feature = "alloc"))]
    assert_eq!(ptsname_alloc(master.fd()).unwrap().as_c_str(), slave_name);

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

    #[cfg(all(target_os = "linux", feature = "alloc"))]
    assert_eq!(ptsname_alloc(r.fd()).unwrap_err().code(), libc::ENOTTY);

    // Now change the sizes

    let mut winsz = Winsize {
        ws_row: 12,
        ws_col: 20,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let (master, slave) = unsafe { openpty(Some(&winsz.clone())) }.unwrap();
    assert_eq!(ioctl_getwinsz(master.fd()).unwrap(), winsz);

    winsz.ws_row = 40;
    winsz.ws_col = 80;
    ioctl_setwinsz(master.fd(), &winsz.clone()).unwrap();
    assert_eq!(ioctl_getwinsz(slave.fd()).unwrap(), winsz);
}
