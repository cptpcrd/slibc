#![cfg(all(feature = "alloc", not(target_os = "android")))]
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
    let passwds: Vec<Passwd> = unsafe { PasswdIter::new() }.map(|g| g.unwrap()).collect();

    for pwd in passwds {
        assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd.clone()));
        assert_eq!(pwd, pwd.clone());

        #[cfg(feature = "std")]
        assert_eq!(hash(&pwd), hash(&pwd.clone()));

        // Look up by name and make sure we get the same result
        let pwd2 = Passwd::lookup_name(pwd.name()).unwrap().unwrap();
        assert_eq!(pwd, pwd2);
        assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd2));

        #[cfg(feature = "std")]
        assert_eq!(hash(&pwd), hash(&pwd2));

        // FreeBSD has a "toor" group which is also UID 0. So don't try to compare to the UID
        // lookup if we see an entry with UID 0 that isn't "root".
        #[cfg(target_os = "freebsd")]
        if pwd.uid() == 0 && pwd.name() != slibc::ffi::OsStr::new("root") {
            continue;
        }

        // Look up by UID and make sure we get the same result
        let pwd3 = Passwd::lookup_uid(pwd.uid()).unwrap().unwrap();
        assert_eq!(pwd, pwd3);
        assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd3));

        #[cfg(feature = "std")]
        assert_eq!(hash(&pwd), hash(&pwd3));
    }
}
