extern "C" {
    pub fn setreuid(ruid: libc::uid_t, euid: libc::uid_t) -> libc::c_int;
    pub fn setregid(rgid: libc::gid_t, egid: libc::gid_t) -> libc::c_int;

    pub fn getpagesize() -> libc::c_int;

    pub fn clock_settime(clockid: libc::clockid_t, tp: *const libc::timespec) -> libc::c_int;
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        extern "C" {
            pub fn syncfs(fd: libc::c_int) -> libc::c_int;

            pub fn mlock2(addr: *const libc::c_void, len: libc::size_t, flags: libc::c_int) -> libc::c_int;
        }

        pub const MLOCK_ONFAULT: libc::c_int = 1;

        pub const MCL_ONFAULT: libc::c_int = 4;
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "dragonfly",
))]
extern "C" {
    pub fn getresuid(
        ruid: *mut libc::uid_t,
        euid: *mut libc::uid_t,
        suid: *mut libc::uid_t,
    ) -> libc::c_int;

    pub fn getresgid(
        rgid: *mut libc::gid_t,
        egid: *mut libc::gid_t,
        sgid: *mut libc::gid_t,
    ) -> libc::c_int;
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
))]
extern "C" {
    pub fn getlogin_r(name: *mut libc::c_char, len: libc::size_t) -> libc::c_int;
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "macos",
    target_os = "ios",
))]
extern "C" {
    pub fn setlogin(name: *const libc::c_char) -> libc::c_int;
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
extern "C" {
    pub fn dup3(oldd: libc::c_int, newd: libc::c_int, flags: libc::c_int) -> libc::c_int;
}
