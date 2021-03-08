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
    let mut passwds: Vec<Passwd> = Vec::new();
    let mut duplicate_uids = Vec::new();
    let mut duplicate_names = Vec::new();

    for passwd in unsafe { PasswdIter::new() } {
        let passwd = passwd.unwrap();
        if passwds.iter().find(|p| p.uid() == passwd.uid()).is_some() {
            duplicate_uids.push(passwd.uid());
        }
        if passwds.iter().find(|p| p.name() == passwd.name()).is_some() {
            duplicate_names.push(passwd.name().to_owned());
        }
        passwds.push(passwd);
    }

    duplicate_uids.sort_unstable();
    duplicate_uids.dedup();

    duplicate_names.sort_unstable();
    duplicate_names.dedup();

    for pwd in passwds {
        assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd.clone()));
        assert_eq!(pwd, pwd.clone());

        #[cfg(feature = "std")]
        assert_eq!(hash(&pwd), hash(&pwd.clone()));

        if duplicate_names.iter().find(|&p| p == pwd.name()).is_none() {
            // Look up by name and make sure we get the same result
            let pwd2 = Passwd::lookup_name(pwd.name()).unwrap().unwrap();
            assert_eq!(pwd, pwd2);
            assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd2));

            #[cfg(feature = "std")]
            assert_eq!(hash(&pwd), hash(&pwd2));
        }

        if !duplicate_uids.contains(&pwd.uid()) {
            // Look up by UID and make sure we get the same result
            let pwd3 = Passwd::lookup_uid(pwd.uid()).unwrap().unwrap();
            assert_eq!(pwd, pwd3);
            assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd3));

            #[cfg(feature = "std")]
            assert_eq!(hash(&pwd), hash(&pwd3));
        }
    }
}
