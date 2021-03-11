extern "C" {
    pub fn setreuid(ruid: libc::uid_t, euid: libc::uid_t) -> libc::c_int;
    pub fn setregid(rgid: libc::gid_t, egid: libc::gid_t) -> libc::c_int;

    pub fn gethostid() -> libc::c_long;

    #[cfg(not(any(target_os = "android", all(target_os = "linux", target_env = "musl"))))]
    pub fn sethostid(hostid: libc::c_long) -> libc::c_int;

    pub fn getpagesize() -> libc::c_int;

    pub fn clock_settime(clockid: libc::clockid_t, tp: *const libc::timespec) -> libc::c_int;
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        extern "C" {
            pub fn syncfs(fd: libc::c_int) -> libc::c_int;

            pub fn mlock2(addr: *const libc::c_void, len: libc::size_t, flags: libc::c_int) -> libc::c_int;
        }

        pub const IOV_MAX: usize = 1024;

        pub const MLOCK_ONFAULT: libc::c_int = 1;

        pub const MCL_ONFAULT: libc::c_int = 4;
    } else if #[cfg(target_os = "android")] {
        pub const IOV_MAX: usize = 1024;
    } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
        pub const CTL_MAXNAME: i32 = 12;

        pub const CLOCK_MONOTONIC_RAW: libc::clockid_t = 4;
        pub const CLOCK_MONOTONIC_RAW_APPROX: libc::clockid_t = 5;
        pub const CLOCK_UPTIME_RAW: libc::clockid_t = 8;
        pub const CLOCK_UPTIME_RAW_APPROX: libc::clockid_t = 9;
    } else if #[cfg(target_os = "netbsd")] {
        pub const SIGRTMIN: libc::c_int = 33;
        pub const SIGRTMAX: libc::c_int = 63;

        pub const CLOCK_VIRTUAL: libc::clockid_t = 1;
        pub const CLOCK_PROF: libc::clockid_t = 2;
        pub const CLOCK_THREAD_CPUTIME_ID: libc::clockid_t = 0x20000000;
        pub const CLOCK_PROCESS_CPUTIME_ID: libc::clockid_t = 0x40000000;
    } else if #[cfg(target_os = "openbsd")] {
        pub const CLOCK_PROCESS_CPUTIME_ID: libc::clockid_t = 2;
        pub const CLOCK_THREAD_CPUTIME_ID: libc::clockid_t = 4;
        pub const CLOCK_UPTIME: libc::clockid_t = 5;
        pub const CLOCK_BOOTTIME: libc::clockid_t = 6;
    } else if #[cfg(target_os = "freebsd")] {
        pub const SIGRTMIN: libc::c_int = 65;
        pub const SIGRTMAX: libc::c_int = 126;

        pub const CTL_MAXNAME: i32 = 24;

        #[cfg(feature = "alloc")]
        extern "C" {
            pub fn mallocx(size: usize, flags: libc::c_int) -> *mut libc::c_void;
            pub fn rallocx(
                ptr: *mut libc::c_void, size: usize, flags: libc::c_int,
            ) -> *mut libc::c_void;
            pub fn sdallocx(ptr: *mut libc::c_void, size: usize, flags: libc::c_int);
        }

        #[cfg(feature = "alloc")]
        pub const MALLOCX_ZERO: libc::c_int = 0x40;

        #[cfg(feature = "alloc")]
        #[allow(non_snake_case)]
        #[inline]
        pub fn MALLOCX_ALIGN(a: usize) -> libc::c_int {
            a.trailing_zeros() as _
        }
    } else if #[cfg(target_os = "dragonfly")] {
        pub const CTL_MAXNAME: i32 = 12;
    }
}

#[cfg(not(netbsdlike))]
pub use libc::{CLOCK_PROCESS_CPUTIME_ID, CLOCK_THREAD_CPUTIME_ID};
#[cfg(freebsdlike)]
pub use libc::{CLOCK_PROF, CLOCK_VIRTUAL};

#[cfg(netbsdlike)]
pub use libc::CTL_MAXNAME;

#[cfg(bsd)]
pub use libc::IOV_MAX;

#[cfg(linuxlike)]
extern "C" {
    pub fn __libc_current_sigrtmin() -> libc::c_int;
    pub fn __libc_current_sigrtmax() -> libc::c_int;
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

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
extern "C" {
    pub fn sigorset(
        dest: *mut libc::sigset_t,
        left: *const libc::sigset_t,
        right: *const libc::sigset_t,
    ) -> libc::c_int;
    pub fn sigandset(
        dest: *mut libc::sigset_t,
        left: *const libc::sigset_t,
        right: *const libc::sigset_t,
    ) -> libc::c_int;
    pub fn sigisemptyset(set: *const libc::sigset_t) -> libc::c_int;
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

    pub fn clock_getcpuclockid(pid: libc::pid_t, clock_id: *mut libc::clockid_t) -> libc::c_int;
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd"))]
extern "C" {
    pub fn pthread_getcpuclockid(
        thread: libc::pthread_t,
        clock_id: *mut libc::clockid_t,
    ) -> libc::c_int;
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

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "macos",
    target_os = "ios",
))]
extern "C" {
    pub fn sysctlnametomib(
        name: *const libc::c_char,
        mibp: *mut libc::c_int,
        sizep: *mut usize,
    ) -> libc::c_int;
}
