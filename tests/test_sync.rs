use slibc::ffi::CStr;
use slibc::{open, OFlag};

#[cfg(not(target_os = "android"))]
#[test]
fn test_sync() {
    // Just make sure it doesn't segfault
    slibc::sync();
}

#[cfg(target_os = "linux")]
#[test]
fn test_syncfs() {
    let f = open(
        CStr::from_bytes_with_nul(b"/\0").unwrap(),
        OFlag::O_RDONLY,
        0,
    )
    .unwrap();
    f.syncfs().unwrap();
}

#[test]
fn test_sync_file() {
    let f = open(
        CStr::from_bytes_with_nul(b"/\0").unwrap(),
        OFlag::O_RDONLY,
        0,
    )
    .unwrap();
    f.sync_all().unwrap();
    f.sync_data().unwrap();
}
