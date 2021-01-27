#![cfg(feature = "alloc")]

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

        for pwd2 in [
            Passwd::lookup_uid(pwd.uid()).unwrap().unwrap(),
            Passwd::lookup_name(pwd.name()).unwrap().unwrap(),
        ]
        .iter()
        {
            assert_eq!(&pwd, pwd2);

            #[cfg(feature = "std")]
            assert_eq!(hash(&pwd), hash(&pwd2));
        }
    }
}
