#![cfg(feature = "alloc")]
// macOS behaves strangely in situations like this
#![cfg(not(target_os = "macos"))]

use slibc::{Passwd, PasswdIter};

#[cfg(feature = "std")]
fn hash(pwd: &Passwd) -> u64 {
    use core::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pwd.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_passwd_iter() {
    for pwd in unsafe { PasswdIter::new() } {
        let pwd = pwd.unwrap();

        assert_eq!(pwd, pwd.clone());

        #[cfg(feature = "std")]
        assert_eq!(hash(&pwd), hash(&pwd.clone()));

        // Look up by name and make sure we get the same result
        assert_eq!(pwd, Passwd::lookup_name(pwd.name()).unwrap().unwrap());

        #[cfg(feature = "std")]
        assert_eq!(
            hash(&pwd),
            hash(&Passwd::lookup_name(pwd.name()).unwrap().unwrap())
        );

        // FreeBSD has a "toor" group which is also UID 0. So don't try to compare to the UID
        // lookup if we see an entry with UID 0 that isn't "root".
        #[cfg(target_os = "freebsd")]
        if pwd.uid() == 0 && pwd.name() != slibc::ffi::OsStr::new("root") {
            continue;
        }

        assert_eq!(pwd, Passwd::lookup_uid(pwd.uid()).unwrap().unwrap());

        #[cfg(feature = "std")]
        assert_eq!(
            hash(&pwd),
            hash(&Passwd::lookup_uid(pwd.uid()).unwrap().unwrap())
        );
    }
}
