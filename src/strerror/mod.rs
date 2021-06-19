cfg_if::cfg_if! {
    if #[cfg(all(target_os = "linux", any(target_env = "gnu", target_env = "")))] {
        #[path = "linux_glibc.rs"]
        mod imp;
    } else if #[cfg(all(target_os = "linux", target_env = "musl"))] {
        #[path = "linux_musl.rs"]
        mod imp;
    } else if #[cfg(target_os = "android")] {
        #[path = "android.rs"]
        mod imp;
    } else if #[cfg(apple)] {
        #[path = "macos.rs"]
        mod imp;
    } else if #[cfg(target_os = "freebsd")] {
        #[path = "freebsd.rs"]
        mod imp;
    } else if #[cfg(target_os = "netbsd")] {
        #[path = "netbsd.rs"]
        mod imp;
    } else if #[cfg(target_os = "openbsd")] {
        #[path = "openbsd.rs"]
        mod imp;
    } else if #[cfg(target_os = "dragonfly")] {
        #[path = "dragonfly.rs"]
        mod imp;
    } else {
        compile_error!("Unsupported OS");
    }
}

#[cfg(bsd)]
pub(crate) fn strerror(eno: i32) -> &'static str {
    imp::ERRNO_TABLE
        .get(eno as usize)
        .copied()
        .unwrap_or("Unknown error")
}

#[cfg(not(bsd))]
pub(crate) use imp::strerror_imp as strerror;

#[cfg(test)]
mod tests {
    fn libc_strerror<'a>(eno: i32) -> Option<&'a str> {
        let ptr = unsafe { libc::strerror(eno) };
        debug_assert!(!ptr.is_null());
        let msg = core::str::from_utf8(unsafe { crate::util::bytes_from_ptr(ptr) }).unwrap();

        if msg.starts_with("Unknown error")
            || (cfg!(all(target_os = "linux", target_env = "musl"))
                && msg == "No error information"
                && eno != 0)
        {
            None
        } else {
            Some(msg)
        }
    }

    #[test]
    fn test_strerror_correct() {
        use super::*;

        let unknown_error = if cfg!(all(target_os = "linux", target_env = "musl")) {
            "No error information"
        } else {
            "Unknown error"
        };

        for eno in 0..=4096 {
            if let Some(s) = libc_strerror(eno) {
                assert_eq!(strerror(eno), s);
            } else if eno >= 2048 {
                assert_eq!(strerror(eno), unknown_error);
            }
        }

        for i in -4096..0 {
            assert_eq!(strerror(i), unknown_error);
        }
    }
}
