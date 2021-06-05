use std::ffi::CString;
use std::fs;
use std::os::unix::prelude::*;

use slibc::ffi::CStr;
use slibc::{open, openat, Errno, OFlag};

#[test]
fn test_open_openat() {
    let tmpdir = tempfile::tempdir().unwrap();
    let tmpdir = tmpdir.as_ref();

    // Open the directory
    let dir = open(
        CStr::from_bytes_with_nul(
            CString::new(tmpdir.as_os_str().as_bytes())
                .unwrap()
                .as_bytes_with_nul(),
        )
        .unwrap(),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        0,
    )
    .unwrap();

    // Opn the file and write some text into it
    let file = openat(
        dir.fd(),
        CStr::from_bytes_with_nul(b"file\0").unwrap(),
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC,
        0o666,
    )
    .unwrap();

    file.write_all(b"abc").unwrap();

    drop(file);

    // Check that `std` confirms the contents are good
    assert_eq!(fs::read(tmpdir.join("file")).unwrap(), b"abc");

    // Now check that openat() confirms the contents are good
    let file = openat(
        dir.fd(),
        CStr::from_bytes_with_nul(b"file\0").unwrap(),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        0,
    )
    .unwrap();

    let mut buf = [0; 3];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"abc");

    assert_eq!(file.read_exact(&mut [0; 1]).unwrap_err(), Errno::EINVAL);
}

#[cfg(feature = "alloc")]
#[test]
fn test_chdir_getcwd() {
    use slibc::{chdir, fchdir, getcwd, getcwd_alloc, PATH_MAX};

    let mut buf = [0; PATH_MAX];

    // chdir()

    chdir("/").unwrap();
    assert_eq!(getcwd(&mut buf).unwrap().to_bytes(), b"/");
    assert_eq!(getcwd_alloc().unwrap().to_bytes(), b"/");

    chdir("/bin").unwrap();
    assert_eq!(
        getcwd(&mut buf).unwrap().to_bytes(),
        std::fs::canonicalize("/bin")
            .unwrap()
            .as_os_str()
            .as_bytes()
    );
    assert_eq!(
        getcwd_alloc().unwrap().to_bytes(),
        std::fs::canonicalize("/bin")
            .unwrap()
            .as_os_str()
            .as_bytes()
    );

    // fchdir()

    let f = open(
        "/",
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        0,
    )
    .unwrap();
    fchdir(f.fd()).unwrap();
    assert_eq!(getcwd(&mut buf).unwrap().to_bytes(), b"/");
    assert_eq!(getcwd_alloc().unwrap().to_bytes(), b"/");

    let f = open(
        "/bin",
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        0,
    )
    .unwrap();
    fchdir(f.fd()).unwrap();
    assert_eq!(
        getcwd(&mut buf).unwrap().to_bytes(),
        std::fs::canonicalize("/bin")
            .unwrap()
            .as_os_str()
            .as_bytes()
    );
    assert_eq!(
        getcwd_alloc().unwrap().to_bytes(),
        std::fs::canonicalize("/bin")
            .unwrap()
            .as_os_str()
            .as_bytes()
    );
}

