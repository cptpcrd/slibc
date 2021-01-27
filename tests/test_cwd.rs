#![cfg(feature = "alloc")]

use std::os::unix::prelude::*;

use slibc::{chdir, fchdir, getcwd, open, OFlag, PATH_MAX};

#[test]
fn test_chdir_getcwd() {
    let mut buf = [0; PATH_MAX];

    chdir("/").unwrap();
    assert_eq!(getcwd(&mut buf).unwrap().to_bytes(), b"/");

    chdir("/bin").unwrap();
    assert_eq!(
        getcwd(&mut buf).unwrap().to_bytes(),
        std::fs::canonicalize("/bin")
            .unwrap()
            .as_os_str()
            .as_bytes()
    );
}

#[test]
fn test_fchdir_getcwd() {
    let mut buf = [0; PATH_MAX];

    let f = open(
        "/",
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        0,
    )
    .unwrap();
    fchdir(f.fd()).unwrap();
    assert_eq!(getcwd(&mut buf).unwrap().to_bytes(), b"/");

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
}
