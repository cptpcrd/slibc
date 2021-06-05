#![cfg(all(feature = "alloc", not(target_os = "android")))]
// macOS behaves strangely in situations like this
#![cfg(not(target_os = "macos"))]

use slibc::{Group, GroupIter, Passwd, PasswdIter};

#[cfg(feature = "std")]
fn hash_pwd(pwd: &Passwd) -> u64 {
    use core::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pwd.hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "std")]
fn hash_grp(grp: &Group) -> u64 {
    use core::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    grp.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_passwd_iter() {
    let mut passwds: Vec<Passwd> = Vec::new();
    let mut duplicate_uids = Vec::new();
    let mut duplicate_names = Vec::new();

    for passwd in unsafe { PasswdIter::new() } {
        let passwd = passwd.unwrap();
        if passwds.iter().any(|p| p.uid() == passwd.uid()) {
            duplicate_uids.push(passwd.uid());
        }
        if passwds.iter().any(|p| p.name() == passwd.name()) {
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
        assert_eq!(hash_pwd(&pwd), hash_pwd(&pwd.clone()));

        if duplicate_names.iter().find(|&p| p == pwd.name()).is_none() {
            // Look up by name and make sure we get the same result
            let pwd2 = Passwd::lookup_name(pwd.name()).unwrap().unwrap();
            assert_eq!(pwd, pwd2);
            assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd2));

            #[cfg(feature = "std")]
            assert_eq!(hash_pwd(&pwd), hash_pwd(&pwd2));
        }

        if !duplicate_uids.contains(&pwd.uid()) {
            // Look up by UID and make sure we get the same result
            let pwd3 = Passwd::lookup_uid(pwd.uid()).unwrap().unwrap();
            assert_eq!(pwd, pwd3);
            assert_eq!(format!("{:?}", pwd), format!("{:?}", pwd3));

            #[cfg(feature = "std")]
            assert_eq!(hash_pwd(&pwd), hash_pwd(&pwd3));
        }
    }
}

#[test]
fn test_group_iter() {
    let mut groups: Vec<Group> = Vec::new();
    let mut duplicate_gids = Vec::new();

    for group in unsafe { GroupIter::new() } {
        let group = group.unwrap();
        if groups.iter().any(|g| g.gid() == group.gid()) {
            duplicate_gids.push(group.gid());
        }
        groups.push(group);
    }

    duplicate_gids.sort_unstable();
    duplicate_gids.dedup();

    for grp in groups {
        assert_eq!(format!("{:?}", grp), format!("{:?}", grp.clone()));
        assert_eq!(grp, grp.clone());

        #[cfg(feature = "std")]
        assert_eq!(hash_grp(&grp), hash_grp(&grp.clone()));

        // Look up by name and make sure we get the same result
        let grp2 = Group::lookup_name(grp.name()).unwrap().unwrap();
        assert_eq!(grp, grp2);
        assert_eq!(format!("{:?}", grp), format!("{:?}", grp2));

        #[cfg(feature = "std")]
        assert_eq!(hash_grp(&grp), hash_grp(&grp2));

        if !duplicate_gids.contains(&grp.gid()) {
            // Look up by GID and make sure we get the same result
            let grp3 = Group::lookup_gid(grp.gid()).unwrap().unwrap();
            assert_eq!(grp, grp3);
            assert_eq!(format!("{:?}", grp), format!("{:?}", grp3));

            #[cfg(feature = "std")]
            assert_eq!(hash_grp(&grp), hash_grp(&grp3));
        }
    }
}
