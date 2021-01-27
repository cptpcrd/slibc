use crate::internal_prelude::*;

/// Get the current thread's `errno` value.
#[inline]
pub fn get_errno() -> libc::c_int {
    unsafe { *util::errno_ptr() }
}

/// Set the current thread's `errno` value.
#[inline]
pub fn set_errno(eno: libc::c_int) {
    unsafe {
        *util::errno_ptr() = eno;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getset_errno() {
        set_errno(0);
        assert_eq!(get_errno(), 0);
        set_errno(0);
        assert_eq!(get_errno(), 0);

        set_errno(libc::EINVAL);
        assert_eq!(get_errno(), libc::EINVAL);
        set_errno(libc::EINVAL);
        assert_eq!(get_errno(), libc::EINVAL);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_errno_thread() {
        set_errno(0);
        assert_eq!(get_errno(), 0);

        std::thread::spawn(|| {
            set_errno(libc::EINVAL);
            assert_eq!(get_errno(), libc::EINVAL);
        })
        .join()
        .unwrap();

        assert_eq!(get_errno(), 0);
    }
}
