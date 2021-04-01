extern "C" {
    pub fn setreuid(ruid: libc::uid_t, euid: libc::uid_t) -> libc::c_int;
    pub fn setregid(rgid: libc::gid_t, egid: libc::gid_t) -> libc::c_int;

    pub fn gethostid() -> libc::c_long;

    #[cfg(not(any(target_os = "android", all(target_os = "linux", target_env = "musl"))))]
    pub fn sethostid(hostid: libc::c_long) -> libc::c_int;

    pub fn getpagesize() -> libc::c_int;

    pub fn clock_settime(clockid: libc::clockid_t, tp: *const libc::timespec) -> libc::c_int;

    #[cfg(not(target_os = "android"))]
    pub fn confstr(name: libc::c_int, buf: *mut libc::c_char, len: usize) -> usize;
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        extern "C" {
            pub fn syncfs(fd: libc::c_int) -> libc::c_int;

            pub fn mlock2(addr: *const libc::c_void, len: libc::size_t, flags: libc::c_int) -> libc::c_int;
        }

        pub const MLOCK_ONFAULT: libc::c_int = 1;
        pub const MCL_ONFAULT: libc::c_int = 4;

        pub const _CS_PATH: libc::c_int = 0;

        pub const NAME_MAX: usize = 255;

        #[cfg(target_env = "musl")]
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            pub re_nsub: usize,
            __opaque: *mut libc::c_void,
            __padding: [*mut libc::c_void; 4usize],
            __nsub2: usize,
            __padding2: libc::c_char,
        }

        #[cfg(any(target_env = "", target_env = "gnu"))]
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            buffer: *mut libc::c_void,
            allocated: usize,
            used: usize,
            syntax: libc::c_ulong,
            fastmap: *mut libc::c_char,
            translate: *mut libc::c_char,
            pub re_nsub: usize,
            bitfields: u8,
        }
    } else if #[cfg(target_os = "android")] {
        pub const NAME_MAX: usize = 255;

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            re_magic: libc::c_int,
            pub re_nsub: usize,
            re_endp: *const libc::c_char,
            re_guts: *mut libc::c_void,
        }
    } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
        pub const CTL_MAXNAME: i32 = 12;

        pub const CLOCK_MONOTONIC_RAW: libc::clockid_t = 4;
        pub const CLOCK_MONOTONIC_RAW_APPROX: libc::clockid_t = 5;
        pub const CLOCK_UPTIME_RAW: libc::clockid_t = 8;
        pub const CLOCK_UPTIME_RAW_APPROX: libc::clockid_t = 9;

        pub const _CS_PATH: libc::c_int = 1;

        pub const NAME_MAX: usize = 255;
    } else if #[cfg(target_os = "netbsd")] {
        pub const SIGRTMIN: libc::c_int = 33;
        pub const SIGRTMAX: libc::c_int = 63;

        pub const CLOCK_VIRTUAL: libc::clockid_t = 1;
        pub const CLOCK_PROF: libc::clockid_t = 2;
        pub const CLOCK_THREAD_CPUTIME_ID: libc::clockid_t = 0x20000000;
        pub const CLOCK_PROCESS_CPUTIME_ID: libc::clockid_t = 0x40000000;

        pub const _CS_PATH: libc::c_int = 1;

        pub const POSIX_FADV_NORMAL: libc::c_int = 0;
        pub const POSIX_FADV_RANDOM: libc::c_int = 1;
        pub const POSIX_FADV_SEQUENTIAL: libc::c_int = 2;
        pub const POSIX_FADV_WILLNEED: libc::c_int = 3;
        pub const POSIX_FADV_DONTNEED: libc::c_int = 4;
        pub const POSIX_FADV_NOREUSE: libc::c_int = 5;

        pub const NAME_MAX: usize = 511;

        extern "C" {
            pub fn posix_fallocate(
                fd: libc::c_int,
                offset: libc::off_t,
                len: libc::off_t,
            ) -> libc::c_int;
        }
    } else if #[cfg(target_os = "openbsd")] {
        pub const CLOCK_PROCESS_CPUTIME_ID: libc::clockid_t = 2;
        pub const CLOCK_THREAD_CPUTIME_ID: libc::clockid_t = 4;
        pub const CLOCK_UPTIME: libc::clockid_t = 5;
        pub const CLOCK_BOOTTIME: libc::clockid_t = 6;

        pub const _CS_PATH: libc::c_int = 1;

        pub const NAME_MAX: usize = 255;
    } else if #[cfg(target_os = "freebsd")] {
        pub const SIGRTMIN: libc::c_int = 65;
        pub const SIGRTMAX: libc::c_int = 126;

        pub const CTL_MAXNAME: i32 = 24;

        pub const NAME_MAX: usize = 255;

        extern "C" {
            pub fn bindat(
                fd: libc::c_int,
                s: libc::c_int,
                addr: *const libc::sockaddr,
                addrlen: libc::socklen_t,
            ) -> libc::c_int;

            pub fn connectat(
                fd: libc::c_int,
                s: libc::c_int,
                name: *const libc::sockaddr,
                namelen: libc::socklen_t,
            ) -> libc::c_int;
        }

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

        pub const _CS_PATH: libc::c_int = 1;
    } else if #[cfg(target_os = "dragonfly")] {
        pub const CTL_MAXNAME: i32 = 12;

        pub const _CS_PATH: libc::c_int = 1;

        pub const NAME_MAX: usize = 255;

        pub const POSIX_FADV_NORMAL: libc::c_int = 0;
        pub const POSIX_FADV_SEQUENTIAL: libc::c_int = 1;
        pub const POSIX_FADV_RANDOM: libc::c_int = 2;
        pub const POSIX_FADV_WILLNEED: libc::c_int = 3;
        pub const POSIX_FADV_DONTNEED: libc::c_int = 4;
        pub const POSIX_FADV_NOREUSE: libc::c_int = 5;
    }
}

cfg_if::cfg_if! {
    if #[cfg(linuxlike)] {
        extern "C" {
            pub fn __libc_current_sigrtmin() -> libc::c_int;
            pub fn __libc_current_sigrtmax() -> libc::c_int;
        }

        pub const IOV_MAX: usize = 1024;

        pub const MADV_WIPEONFORK: libc::c_int = 18;
        pub const MADV_KEEPONFORK: libc::c_int = 19;
        pub const MADV_COLD: libc::c_int = 20;
        pub const MADV_PAGEOUT: libc::c_int = 21;
    } else if #[cfg(bsd)] {
        pub use libc::IOV_MAX;

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            re_magic: libc::c_int,
            pub re_nsub: usize,
            re_endp: *const libc::c_char,
            re_guts: *mut libc::c_void,
        }
    }
}

#[cfg(not(netbsdlike))]
pub use libc::{CLOCK_PROCESS_CPUTIME_ID, CLOCK_THREAD_CPUTIME_ID};
#[cfg(freebsdlike)]
pub use libc::{CLOCK_PROF, CLOCK_VIRTUAL};

#[cfg(netbsdlike)]
pub use libc::CTL_MAXNAME;

#[cfg(any(linuxlike, target_os = "freebsd"))]
pub use libc::{
    posix_fadvise, posix_fallocate, POSIX_FADV_DONTNEED, POSIX_FADV_NOREUSE, POSIX_FADV_NORMAL,
    POSIX_FADV_RANDOM, POSIX_FADV_SEQUENTIAL, POSIX_FADV_WILLNEED,
};

#[cfg(any(target_os = "dragonfly", target_os = "netbsd"))]
extern "C" {
    pub fn posix_fadvise(
        fd: libc::c_int,
        offset: libc::off_t,
        len: libc::off_t,
        advice: libc::c_int,
    ) -> libc::c_int;
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
