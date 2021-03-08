#![cfg(all(feature = "alloc", not(target_os = "android")))]
// macOS behaves strangely in situations like this
#![cfg(not(target_os = "macos"))]

use slibc::{Group, GroupIter};

#[cfg(feature = "std")]
fn hash(grp: &Group) -> u64 {
    use core::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    grp.hash(&mut hasher);
    hasher.finish()
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
        assert_eq!(hash(&grp), hash(&grp.clone()));

        // Look up by name and make sure we get the same result
        let grp2 = Group::lookup_name(grp.name()).unwrap().unwrap();
        assert_eq!(grp, grp2);
        assert_eq!(format!("{:?}", grp), format!("{:?}", grp2));

        #[cfg(feature = "std")]
        assert_eq!(hash(&grp), hash(&grp2));

        if !duplicate_gids.contains(&grp.gid()) {
            // Look up by GID and make sure we get the same result
            let grp3 = Group::lookup_gid(grp.gid()).unwrap().unwrap();
            assert_eq!(grp, grp3);
            assert_eq!(format!("{:?}", grp), format!("{:?}", grp3));

            #[cfg(feature = "std")]
            assert_eq!(hash(&grp), hash(&grp3));
        }
    }
}
