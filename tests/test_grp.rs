#![cfg(feature = "alloc")]
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
    for grp in unsafe { GroupIter::new() } {
        let grp = grp.unwrap();

        assert_eq!(grp, grp.clone());

        #[cfg(feature = "std")]
        assert_eq!(hash(&grp), hash(&grp.clone()));

        // Look up by name and make sure we get the same result
        assert_eq!(grp, Group::lookup_name(grp.name()).unwrap().unwrap());

        #[cfg(feature = "std")]
        assert_eq!(
            hash(&grp),
            hash(&Group::lookup_name(grp.name()).unwrap().unwrap())
        );

        assert_eq!(grp, Group::lookup_gid(grp.gid()).unwrap().unwrap());

        #[cfg(feature = "std")]
        assert_eq!(
            hash(&grp),
            hash(&Group::lookup_gid(grp.gid()).unwrap().unwrap())
        );
    }
}
