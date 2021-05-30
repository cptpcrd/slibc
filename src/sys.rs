#![allow(non_camel_case_types)]

extern "C" {
    pub fn setreuid(ruid: libc::uid_t, euid: libc::uid_t) -> libc::c_int;
    pub fn setregid(rgid: libc::gid_t, egid: libc::gid_t) -> libc::c_int;

    pub fn gethostid() -> libc::c_long;

    #[cfg(not(any(
        target_os = "android",
        all(target_os = "linux", target_env = "musl"),
        apple,
        freebsdlike,
    )))]
    pub fn sethostid(hostid: libc::c_long) -> libc::c_int;
    #[cfg(any(apple, freebsdlike))]
    pub fn sethostid(hostid: libc::c_long);

    pub fn getpagesize() -> libc::c_int;

    pub fn clock_settime(clockid: libc::clockid_t, tp: *const libc::timespec) -> libc::c_int;

    #[cfg(not(target_os = "android"))]
    pub fn confstr(name: libc::c_int, buf: *mut libc::c_char, len: usize) -> usize;
}

#[cfg(not(any(apple, target_os = "freebsd")))]
pub use libc::_PC_2_SYMLINKS;
#[cfg(not(any(apple, target_os = "android")))]
pub use libc::_PC_FILESIZEBITS;
#[cfg(not(any(apple, target_os = "netbsd")))]
pub use libc::{
    _PC_ALLOC_SIZE_MIN, _PC_REC_INCR_XFER_SIZE, _PC_REC_MAX_XFER_SIZE, _PC_REC_MIN_XFER_SIZE,
    _PC_REC_XFER_ALIGN,
};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub use libc::{
            reboot, RB_AUTOBOOT, RB_HALT_SYSTEM, RB_POWER_OFF, RB_KEXEC, RB_ENABLE_CAD,
            RB_DISABLE_CAD, RB_SW_SUSPEND,
        };

        extern "C" {
            pub fn syncfs(fd: libc::c_int) -> libc::c_int;

            pub fn mlock2(addr: *const libc::c_void, len: libc::size_t, flags: libc::c_int) -> libc::c_int;

            pub fn posix_spawn_file_actions_addchdir_np(
                fa: *mut libc::posix_spawn_file_actions_t,
                path: *const libc::c_char,
            ) -> libc::c_int;
            pub fn posix_spawn_file_actions_addfchdir_np(
                fa: *mut libc::posix_spawn_file_actions_t,
                fd: libc::c_int,
            ) -> libc::c_int;
        }

        pub const MLOCK_ONFAULT: libc::c_int = 1;
        pub const MCL_ONFAULT: libc::c_int = 4;

        pub const _CS_PATH: libc::c_int = 0;
        pub const _CS_GNU_LIBC_VERSION: libc::c_int = 2;
        pub const _CS_GNU_LIBPTHREAD_VERSION: libc::c_int = 3;

        pub const NAME_MAX: usize = 255;

        pub const POSIX_SPAWN_SETSID: libc::c_short = 0x80;

        #[cfg(any(target_env = "", target_env = "gnu"))]
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct statfs {
            pub f_type: libc::__fsword_t,
            pub f_bsize: libc::__fsword_t,
            pub f_blocks: libc::fsblkcnt_t,
            pub f_bfree: libc::fsblkcnt_t,
            pub f_bavail: libc::fsblkcnt_t,
            pub f_files: libc::fsblkcnt_t,
            pub f_ffree: libc::fsblkcnt_t,
            pub f_fsid: libc::fsid_t,
            pub f_namelen: libc::__fsword_t,
            pub f_frsize: libc::__fsword_t,
            pub f_flags: libc::__fsword_t,
            pub f_spare: [libc::__fsword_t; 4],
        }

        #[cfg(target_env = "musl")]
        pub use libc::{statfs, O_SEARCH};

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
        extern "C" {
            pub fn reboot(cmd: libc::c_int) -> libc::c_int;
        }

        pub const NAME_MAX: usize = 255;

        pub const _PC_FILESIZEBITS: libc::c_int = 0;

        pub const RB_AUTOBOOT: libc::c_int = libc::LINUX_REBOOT_CMD_RESTART;
        pub const RB_HALT_SYSTEM: libc::c_int = libc::LINUX_REBOOT_CMD_HALT;
        pub const RB_POWER_OFF: libc::c_int = libc::LINUX_REBOOT_CMD_POWER_OFF;
        pub const RB_KEXEC: libc::c_int = libc::LINUX_REBOOT_CMD_KEXEC;
        pub const RB_SW_SUSPEND: libc::c_int = libc::LINUX_REBOOT_CMD_SW_SUSPEND;
        pub const RB_DISABLE_CAD: libc::c_int = libc::LINUX_REBOOT_CMD_CAD_OFF;
        pub const RB_ENABLE_CAD: libc::c_int = libc::LINUX_REBOOT_CMD_CAD_ON;

        pub use libc::statfs;

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            re_magic: libc::c_int,
            pub re_nsub: usize,
            re_endp: *const libc::c_char,
            re_guts: *mut libc::c_void,
        }
    } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
        pub use libc::getfsstat;

        pub const CTL_MAXNAME: i32 = 12;

        pub const CLOCK_MONOTONIC_RAW: libc::clockid_t = 4;
        pub const CLOCK_MONOTONIC_RAW_APPROX: libc::clockid_t = 5;
        pub const CLOCK_UPTIME_RAW: libc::clockid_t = 8;
        pub const CLOCK_UPTIME_RAW_APPROX: libc::clockid_t = 9;

        pub const _CS_PATH: libc::c_int = 1;
        pub const _PC_2_SYMLINKS: libc::c_int = 15;
        pub const _PC_ALLOC_SIZE_MIN: libc::c_int = 16;
        pub const _PC_FILESIZEBITS: libc::c_int = 18;
        pub const _PC_REC_INCR_XFER_SIZE: libc::c_int = 20;
        pub const _PC_REC_MAX_XFER_SIZE: libc::c_int = 21;
        pub const _PC_REC_MIN_XFER_SIZE: libc::c_int = 22;
        pub const _PC_REC_XFER_ALIGN: libc::c_int = 23;

        pub const NAME_MAX: usize = 255;

        pub const POSIX_SPAWN_SETSID: libc::c_short = 0x400;

        pub const MNT_RDONLY: u32 = 0x1;
        pub const MNT_SYNCHRONOUS: u32 = 0x2;
        pub const MNT_NOEXEC: u32 = 0x4;
        pub const MNT_NOSUID: u32 = 0x8;
        pub const MNT_NODEV: u32 = 0x10;
        pub const MNT_UNION: u32 = 0x20;
        pub const MNT_ASYNC: u32 = 0x40;
        pub const MNT_CPROTECT: u32 = 0x80;
        pub const MNT_EXPORTED: u32 = 0x100;
        pub const MNT_REMOVABLE: u32 = 0x200;
        pub const MNT_QUARANTINE: u32 = 0x400;
        pub const MNT_LOCAL: u32 = 0x1000;
        pub const MNT_QUOTA: u32 = 0x2000;
        pub const MNT_ROOTFS: u32 = 0x4000;
        pub const MNT_DOVOLFS: u32 = 0x8000;
        pub const MNT_DONTBROWSE: u32 = 0x100000;
        pub const MNT_IGNORE_OWNERSHIP: u32 = 0x200000;
        pub const MNT_AUTOMOUNTED: u32 = 0x400000;
        pub const MNT_JOURNALED: u32 = 0x800000;
        pub const MNT_NOUSERXATTR: u32 = 0x100000;
        pub const MNT_DEFWRITE: u32 = 0x200000;
        pub const MNT_MULTILABEL: u32 = 0x4000000;
        pub const MNT_NOATIME: u32 = 0x10000000;
        pub const MNT_SNAPSHOT: u32 = 0x40000000;
        pub const MNT_STRICTATIME: u32 = 0x80000000;

        pub const RB_AUTOBOOT: libc::c_int = 0;
        pub const RB_ASKNAME: libc::c_int = 0x01;
        pub const RB_SINGLE: libc::c_int = 0x02;
        pub const RB_NOSYNC: libc::c_int = 0x04;
        pub const RB_HALT: libc::c_int = 0x08;
        // pub const RB_INITNAME: libc::c_int = 0x10;
        // pub const RB_DFLTROOT: libc::c_int = 0x20;
        // pub const RB_ALTBOOT: libc::c_int = 0x40;
        // pub const RB_UNIPROC: libc::c_int = 0x80;
        pub const RB_SAFEBOOT: libc::c_int = 0x100;
        pub const RB_UPSDELAY: libc::c_int = 0x200;
        pub const RB_QUICK: libc::c_int = 0x400;
        //  const RB_PANIC: libc::c_int = 0x800;
        //  const RB_PANIC_ZPRINT: libc::c_int = 0x1000;

        // Aliases to make compatibility easier
        pub const RB_HALT_SYSTEM: libc::c_int = RB_HALT;
    } else if #[cfg(target_os = "netbsd")] {
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct sched_param {
            pub sched_priority: libc::c_int,
        }

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct posix_spawnattr_t {
            sa_flags: libc::c_short,
            sa_pgroup: libc::pid_t,
            sa_schedparam: sched_param,
            sa_schedpolicy: libc::c_int,
            sa_sigdefault: libc::sigset_t,
            sa_sigmask: libc::sigset_t,
        }

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct posix_spawn_file_actions_t {
            size: libc::c_uint,
            len: libc::c_uint,
            fae: *mut libc::c_void,
        }

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

        pub const SWAP_ON: libc::c_int = 1;
        pub const SWAP_OFF: libc::c_int = 2;
        pub const SWAP_NSWAP: libc::c_int = 3;
        pub const SWAP_CTL: libc::c_int = 5;
        pub const SWAP_DUMPDEV: libc::c_int = 7;
        pub const SWAP_GETDUMPDEV: libc::c_int = 8;
        pub const SWAP_DUMPOFF: libc::c_int = 9;
        pub const SWAP_STATS: libc::c_int = 10;

        pub const SWF_INUSE: libc::c_int = 0x1;
        pub const SWF_ENABLE: libc::c_int = 0x2;
        pub const SWF_BUSY: libc::c_int = 0x4;
        pub const SWF_FAKE: libc::c_int = 0x8;

        pub const RB_AUTOBOOT: libc::c_int = 0;
        pub const RB_ASKNAME: libc::c_int = 0x1;
        pub const RB_SINGLE: libc::c_int = 0x2;
        pub const RB_NOSYNC: libc::c_int = 0x4;
        pub const RB_HALT: libc::c_int = 0x8;
        //pub const RB_INITNAME: libc::c_int = 0x10;
        pub const RB_KDB: libc::c_int = 0x40;
        pub const RB_RDONLY: libc::c_int = 0x80;
        pub const RB_DUMP: libc::c_int = 0x100;
        pub const RB_MINIROOT: libc::c_int = 0x200;
        pub const RB_POWERDOWN: libc::c_int = RB_HALT | 0x800;
        pub const RB_USERCONF: libc::c_int = 0x1_000;

        // Aliases to make compatibility easier
        pub const RB_HALT_SYSTEM: libc::c_int = RB_HALT;
        pub const RB_POWER_OFF: libc::c_int = RB_POWERDOWN;

        extern "C" {
            pub fn posix_fallocate(
                fd: libc::c_int,
                offset: libc::off_t,
                len: libc::off_t,
            ) -> libc::c_int;

            pub fn swapon(path: *const libc::c_char) -> libc::c_int;
            pub fn swapctl(
                cmd: libc::c_int, arg: *const libc::c_void, misc: libc::c_int,
            ) -> libc::c_int;

            pub fn reboot(howto: libc::c_int, bootstr: *mut libc::c_char) -> libc::c_int;
        }
    } else if #[cfg(target_os = "openbsd")] {
        extern "C" {
            pub fn getfsstat(
                buf: *mut libc::statfs, bufsize: libc::size_t, flags: libc::c_int
            ) -> libc::c_int;

            pub fn swapctl(
                cmd: libc::c_int, arg: *const libc::c_void, misc: libc::c_int,
            ) -> libc::c_int;
        }

        pub type posix_spawnattr_t = *mut libc::c_void;
        pub type posix_spawn_file_actions_t = *mut libc::c_void;

        pub const CLOCK_PROCESS_CPUTIME_ID: libc::clockid_t = 2;
        pub const CLOCK_THREAD_CPUTIME_ID: libc::clockid_t = 4;
        pub const CLOCK_UPTIME: libc::clockid_t = 5;
        pub const CLOCK_BOOTTIME: libc::clockid_t = 6;

        pub const _CS_PATH: libc::c_int = 1;

        pub const NAME_MAX: usize = 255;

        pub const MNT_RDONLY: u32 = 0x1;
        pub const MNT_SYNCHRONOUS: u32 = 0x2;
        pub const MNT_NOEXEC: u32 = 0x4;
        pub const MNT_NOSUID: u32 = 0x8;
        pub const MNT_NODEV: u32 = 0x10;
        pub const MNT_NOPERM: u32 = 0x20;
        pub const MNT_ASYNC: u32 = 0x40;
        pub const MNT_WXALLOWED: u32 = 0x800;
        pub const MNT_EXRDONLY: u32 = 0x80;
        pub const MNT_EXPORTED: u32 = 0x100;
        pub const MNT_DEFEXPORTED: u32 = 0x200;
        pub const MNT_EXPORTANON: u32 = 0x400;
        pub const MNT_LOCAL: u32 = 0x1000;
        pub const MNT_QUOTA: u32 = 0x2000;
        pub const MNT_ROOTFS: u32 = 0x4000;
        pub const MNT_NOATIME: u32 = 0x8000;

        pub const SWAP_ON: libc::c_int = 1;
        pub const SWAP_OFF: libc::c_int = 2;
        pub const SWAP_NSWAP: libc::c_int = 3;
        pub const SWAP_STATS: libc::c_int = 4;
        pub const SWAP_CTL: libc::c_int = 5;
        pub const SWAP_DUMPDEV: libc::c_int = 7;

        pub const SWF_INUSE: libc::c_int = 0x1;
        pub const SWF_ENABLE: libc::c_int = 0x2;
        pub const SWF_BUSY: libc::c_int = 0x4;
        pub const SWF_FAKE: libc::c_int = 0x8;

        pub const RB_AUTOBOOT: libc::c_int = 0;
        pub const RB_ASKNAME: libc::c_int = 0x1;
        pub const RB_SINGLE: libc::c_int = 0x2;
        pub const RB_NOSYNC: libc::c_int = 0x4;
        pub const RB_HALT: libc::c_int = 0x8;
        //pub const RB_INITNAME: libc::c_int = 0x10;
        pub const RB_DFLTROOT: libc::c_int = 0x20;
        pub const RB_KDB: libc::c_int = 0x40;
        pub const RB_RDONLY: libc::c_int = 0x80;
        pub const RB_DUMP: libc::c_int = 0x100;
        pub const RB_MINIROOT: libc::c_int = 0x200;
        pub const RB_CONFIG: libc::c_int = 0x400;
        pub const RB_TIMEBAD: libc::c_int = 0x800;
        pub const RB_POWERDOWN: libc::c_int = 0x1_000;
        pub const RB_SERCONS: libc::c_int = 0x2_000;
        pub const RB_USERREQ: libc::c_int = 0x4_000;
        pub const RB_RESET: libc::c_int = 0x8_000;
        pub const RB_GOODRANDOM: libc::c_int = 0x1_000;

        // Aliases to make compatibility easier
        pub const RB_HALT_SYSTEM: libc::c_int = RB_HALT;
        pub const RB_POWER_OFF: libc::c_int = RB_POWERDOWN | RB_HALT;
    } else if #[cfg(target_os = "freebsd")] {
        extern "C" {
            #[link_name = "getfsstat@FBSD_1.0"]
            pub fn getfsstat_compat11(
                buf: *mut libc::statfs, bufsize: libc::c_long, mode: libc::c_int
            ) -> libc::c_int;

            pub fn getfsstat(
                buf: *mut libc::statfs, bufsize: libc::c_long, mode: libc::c_int
            ) -> libc::c_int;
        }

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

        pub const MNT_RDONLY: u64 = 0x1;
        pub const MNT_SYNCHRONOUS: u64 = 0x2;
        pub const MNT_NOEXEC: u64 = 0x4;
        pub const MNT_NOSUID: u64 = 0x8;
        pub const MNT_NFS4ACLS: u64 = 0x10;
        pub const MNT_UNION: u64 = 0x20;
        pub const MNT_ASYNC: u64 = 0x40;
        pub const MNT_SUIDDIR: u64 = 0x100000;
        pub const MNT_SOFTDEP: u64 = 0x200000;
        pub const MNT_NOSYMFOLLOW: u64 = 0x400000;
        pub const MNT_GJOURNAL: u64 = 0x2000000;
        pub const MNT_MULTILABEL: u64 = 0x4000000;
        pub const MNT_ACLS: u64 = 0x8000000;
        pub const MNT_NOATIME: u64 = 0x10000000;
        pub const MNT_NOCLUSTERR: u64 = 0x40000000;
        pub const MNT_NOCLUSTERW: u64 = 0x80000000;
        pub const MNT_SUJ: u64 = 0x100000000;
        pub const MNT_AUTOMOUNTED: u64 = 0x200000000;
        pub const MNT_UNTRUSTED: u64 = 0x800000000;
        pub const MNT_EXRDONLY: u64 = 0x80;
        pub const MNT_EXPORTED: u64 = 0x100;
        pub const MNT_DEFEXPORTED: u64 = 0x200;
        pub const MNT_EXPORTANON: u64 = 0x400;
        pub const MNT_EXKERB: u64 = 0x800;
        pub const MNT_EXPUBLIC: u64 = 0x20000000;
        pub const MNT_LOCAL: u64 = 0x1000;
        pub const MNT_QUOTA: u64 = 0x2000;
        pub const MNT_ROOTFS: u64 = 0x4000;
        pub const MNT_USER: u64 = 0x8000;
        pub const MNT_IGNORE: u64 = 0x800000;
        pub const MNT_VERIFIED: u64 = 0x400000000;

        pub const RB_AUTOBOOT: libc::c_int = 0;
        pub const RB_ASKNAME: libc::c_int = 0x1;
        pub const RB_SINGLE: libc::c_int = 0x2;
        pub const RB_NOSYNC: libc::c_int = 0x4;
        pub const RB_HALT: libc::c_int = 0x8;
        // pub const RB_INITNAME: libc::c_int = 0x010;
        pub const RB_DFLTROOT: libc::c_int = 0x20;
        pub const RB_KDB: libc::c_int = 0x40;
        pub const RB_RDONLY: libc::c_int = 0x80;
        pub const RB_DUMP: libc::c_int = 0x100;
        // pub const RB_MINIROOT: libc::c_int = 0x200;
        pub const RB_VERBOSE: libc::c_int = 0x800;
        pub const RB_SERIAL: libc::c_int = 0x1_000;
        pub const RB_CDROM: libc::c_int = 0x2_000;
        pub const RB_POWEROFF: libc::c_int = 0x4_000;
        pub const RB_GDB: libc::c_int = 0x8_000;
        pub const RB_MUTE: libc::c_int = 0x10_000;
        // pub const RB_SELFTEST: libc::c_int = 0x20_000;
        pub const RB_PAUSE: libc::c_int = 0x100_000;
        pub const RB_REROOT: libc::c_int = 0x200_000;
        pub const RB_POWERCYCLE: libc::c_int = 0x400_000;
        pub const RB_PROBE: libc::c_int = 0x10_000_000;
        pub const RB_MULTIPLE: libc::c_int = 0x20_000_000;

        // Aliases to make compatibility easier
        pub const RB_HALT_SYSTEM: libc::c_int = RB_HALT;
        pub const RB_POWER_OFF: libc::c_int = RB_POWEROFF;

        // FreeBSD has historically implemented O_EXEC on directories with *similar* semantics to
        // POSIX's O_SEARCH. FreeBSD 13.0 (also backported to 11.4 and 12.2) a) added O_SEARCH as an
        // alias for O_EXEC and b) cleaned up the behavior a bit.
        // The libc crate doesn't define O_SEARCH on FreeBSD (yet?), so we have an alias here.
        pub const O_SEARCH: libc::c_int = libc::O_EXEC;
    } else if #[cfg(target_os = "dragonfly")] {
        extern "C" {
            pub fn getfsstat(
                buf: *mut libc::statfs, bufsize: libc::c_long, flags: libc::c_int
            ) -> libc::c_int;
        }

        pub type posix_spawnattr_t = *mut libc::c_void;
        pub type posix_spawn_file_actions_t = *mut libc::c_void;

        pub const CTL_MAXNAME: i32 = 12;

        pub const _CS_PATH: libc::c_int = 1;

        pub const NAME_MAX: usize = 255;

        pub const POSIX_FADV_NORMAL: libc::c_int = 0;
        pub const POSIX_FADV_SEQUENTIAL: libc::c_int = 1;
        pub const POSIX_FADV_RANDOM: libc::c_int = 2;
        pub const POSIX_FADV_WILLNEED: libc::c_int = 3;
        pub const POSIX_FADV_DONTNEED: libc::c_int = 4;
        pub const POSIX_FADV_NOREUSE: libc::c_int = 5;

        pub const MNT_RDONLY: i32 = 0x1;
        pub const MNT_SYNCHRONOUS: i32 = 0x2;
        pub const MNT_NOEXEC: i32 = 0x4;
        pub const MNT_NOSUID: i32 = 0x8;
        pub const MNT_NODEV: i32 = 0x10;
        pub const MNT_AUTOMOUNTED: i32 = 0x20;
        pub const MNT_ASYNC: i32 = 0x40;
        pub const MNT_SUIDDIR: i32 = 0x100000;
        pub const MNT_SOFTDEP: i32 = 0x200000;
        pub const MNT_NOSYMFOLLOW: i32 = 0x400000;
        pub const MNT_TRIM: i32 = 0x1000000;
        pub const MNT_NOATIME: i32 = 0x10000000;
        pub const MNT_NOCLUSTERR: i32 = 0x40000000;
        pub const MNT_NOCLUSTERW: i32 = 0x80000000u32 as i32;
        pub const MNT_EXRDONLY: i32 = 0x80;
        pub const MNT_EXPORTED: i32 = 0x100;
        pub const MNT_DEFEXPORTED: i32 = 0x200;
        pub const MNT_EXPORTANON: i32 = 0x400;
        pub const MNT_EXKERB: i32 = 0x800;
        pub const MNT_EXPUBLIC: i32 = 0x20000000;
        pub const MNT_LOCAL: i32 = 0x1000;
        pub const MNT_QUOTA: i32 = 0x2000;
        pub const MNT_ROOTFS: i32 = 0x4000;
        pub const MNT_USER: i32 = 0x8000;
        pub const MNT_IGNORE: i32 = 0x800000;

        pub const RB_AUTOBOOT: libc::c_int = 0;
        pub const RB_ASKNAME: libc::c_int = 0x1;
        pub const RB_SINGLE: libc::c_int = 0x2;
        pub const RB_NOSYNC: libc::c_int = 0x4;
        pub const RB_HALT: libc::c_int = 0x8;
        // pub const RB_INITNAME: libc::c_int = 0x010;
        pub const RB_DFLTROOT: libc::c_int = 0x20;
        pub const RB_KDB: libc::c_int = 0x40;
        pub const RB_RDONLY: libc::c_int = 0x80;
        pub const RB_DUMP: libc::c_int = 0x100;
        pub const RB_MINIROOT: libc::c_int = 0x200;
        pub const RB_VERBOSE: libc::c_int = 0x800;
        pub const RB_SERIAL: libc::c_int = 0x1_000;
        pub const RB_CDROM: libc::c_int = 0x2_000;
        pub const RB_POWEROFF: libc::c_int = 0x4_000;
        pub const RB_GDB: libc::c_int = 0x8_000;
        pub const RB_MUTE: libc::c_int = 0x10_000;
        pub const RB_SELFTEST: libc::c_int = 0x20_000;
        pub const RB_PAUSE: libc::c_int = 0x100_000;
        pub const RB_VIDEO: libc::c_int = 0x20_000_000;

        // Aliases to make compatibility easier
        pub const RB_HALT_SYSTEM: libc::c_int = RB_HALT;
        pub const RB_POWER_OFF: libc::c_int = RB_POWEROFF;
    }
}

cfg_if::cfg_if! {
    if #[cfg(freebsdlike)] {
        extern "C" {
            pub fn swapon(path: *const libc::c_char) -> libc::c_int;
            pub fn swapoff(path: *const libc::c_char) -> libc::c_int;
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(netbsdlike, target_os = "dragonfly"))] {
        extern "C" {
            pub fn posix_spawn(
                pidp: *mut libc::pid_t,
                path: *const libc::c_char,
                file_actions: *const posix_spawn_file_actions_t,
                attrp: *const posix_spawnattr_t,
                argv: *const *mut libc::c_char,
                envp: *const *mut libc::c_char,
            ) -> libc::c_int;
            pub fn posix_spawnp(
                pidp: *mut libc::pid_t,
                path: *const libc::c_char,
                file_actions: *const posix_spawn_file_actions_t,
                attrp: *const posix_spawnattr_t,
                argv: *const *mut libc::c_char,
                envp: *const *mut libc::c_char,
            ) -> libc::c_int;

            pub fn posix_spawn_file_actions_init(
                file_actions: *const posix_spawn_file_actions_t,
            ) -> libc::c_int;
            pub fn posix_spawn_file_actions_destroy(
                file_actions: *const posix_spawn_file_actions_t,
            ) -> libc::c_int;
            pub fn posix_spawn_file_actions_adddup2(
                file_actions: *const posix_spawn_file_actions_t,
                fildes: libc::c_int,
                newfildes: libc::c_int,
            ) -> libc::c_int;
            pub fn posix_spawn_file_actions_addopen(
                file_actions: *const posix_spawn_file_actions_t,
                fildes: libc::c_int,
                path: *const libc::c_char,
                oflag: libc::c_int,
                mode: libc::mode_t,
            ) -> libc::c_int;
            pub fn posix_spawn_file_actions_addclose(
                file_actions: *const posix_spawn_file_actions_t,
                fildes: libc::c_int,
            ) -> libc::c_int;

            pub fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> libc::c_int;
            pub fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> libc::c_int;
            pub fn posix_spawnattr_setflags(
                attr: *mut posix_spawnattr_t,
                flags: libc::c_short,
            ) -> libc::c_int;
            pub fn posix_spawnattr_getflags(
                attr: *const posix_spawnattr_t,
                flags: *mut libc::c_short,
            ) -> libc::c_int;
            pub fn posix_spawnattr_setpgroup(
                attr: *mut posix_spawnattr_t,
                pgroup: libc::pid_t,
            ) -> libc::c_int;
            pub fn posix_spawnattr_getpgroup(
                attr: *const posix_spawnattr_t,
                pgroup: *mut libc::pid_t,
            ) -> libc::c_int;
            pub fn posix_spawnattr_setsigdefault(
                attr: *mut posix_spawnattr_t,
                sigdefault: *const libc::sigset_t,
            ) -> libc::c_int;
            pub fn posix_spawnattr_getsigdefault(
                attr: *const posix_spawnattr_t,
                sigdefault: *mut libc::sigset_t,
            ) -> libc::c_int;
            pub fn posix_spawnattr_setsigmask(
                attr: *mut posix_spawnattr_t,
                sigmask: *const libc::sigset_t,
            ) -> libc::c_int;
            pub fn posix_spawnattr_getsigmask(
                attr: *const posix_spawnattr_t,
                sigmask: *mut libc::sigset_t,
            ) -> libc::c_int;
        }

        pub const POSIX_SPAWN_RESETIDS: libc::c_short = 0x01;
        pub const POSIX_SPAWN_SETPGROUP: libc::c_short = 0x02;
        // pub const POSIX_SPAWN_SETSCHEDPARAM: libc::c_short = 0x04;
        // pub const POSIX_SPAWN_SETSCHEDULER: libc::c_short = 0x08;
        pub const POSIX_SPAWN_SETSIGDEF: libc::c_short = 0x10;
        pub const POSIX_SPAWN_SETSIGMASK: libc::c_short = 0x20;
    }
}

cfg_if::cfg_if! {
    if #[cfg(linuxlike)] {
        pub use libc::swapoff;

        extern "C" {
            pub fn __libc_current_sigrtmin() -> libc::c_int;
            pub fn __libc_current_sigrtmax() -> libc::c_int;
        }

        pub const IOV_MAX: usize = 1024;

        pub const MADV_WIPEONFORK: libc::c_int = 18;
        pub const MADV_KEEPONFORK: libc::c_int = 19;
        pub const MADV_COLD: libc::c_int = 20;
        pub const MADV_PAGEOUT: libc::c_int = 21;

        pub const RWF_HIPRI: libc::c_int = 0x1;
        pub const RWF_DSYNC: libc::c_int = 0x2;
        pub const RWF_SYNC: libc::c_int = 0x4;
        pub const RWF_NOWAIT: libc::c_int = 0x8;
        pub const RWF_APPEND: libc::c_int = 0x10;

        pub const TFD_TIMER_CANCEL_ON_SET: libc::c_int = 2;
    } else if #[cfg(bsd)] {
        pub use libc::IOV_MAX;

        #[cfg(not(target_os = "netbsd"))]
        pub use libc::statfs;

        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct regex_t {
            re_magic: libc::c_int,
            pub re_nsub: usize,
            re_endp: *const libc::c_char,
            re_guts: *mut libc::c_void,
        }

        #[cfg(not(target_os = "netbsd"))]
        pub const MNT_WAIT: libc::c_int = 1;
        #[cfg(not(target_os = "netbsd"))]
        pub const MNT_NOWAIT: libc::c_int = 2;

        #[cfg(not(target_os = "netbsd"))]
        extern "C" {
            pub fn reboot(howto: libc::c_int) -> libc::c_int;
        }
    }
}

#[cfg(not(netbsdlike))]
pub use libc::{CLOCK_PROCESS_CPUTIME_ID, CLOCK_THREAD_CPUTIME_ID};
#[cfg(freebsdlike)]
pub use libc::{CLOCK_PROF, CLOCK_VIRTUAL};

#[cfg(netbsdlike)]
pub use libc::CTL_MAXNAME;

#[cfg(not(any(target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")))]
pub use libc::{
    posix_spawn, posix_spawn_file_actions_addclose, posix_spawn_file_actions_adddup2,
    posix_spawn_file_actions_addopen, posix_spawn_file_actions_destroy,
    posix_spawn_file_actions_init, posix_spawn_file_actions_t, posix_spawnattr_destroy,
    posix_spawnattr_getflags, posix_spawnattr_getpgroup, posix_spawnattr_getsigdefault,
    posix_spawnattr_getsigmask, posix_spawnattr_init, posix_spawnattr_setflags,
    posix_spawnattr_setpgroup, posix_spawnattr_setsigdefault, posix_spawnattr_setsigmask,
    posix_spawnattr_t, posix_spawnp, POSIX_SPAWN_RESETIDS, POSIX_SPAWN_SETPGROUP,
    POSIX_SPAWN_SETSIGDEF, POSIX_SPAWN_SETSIGMASK,
};

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

#[cfg(any(linuxlike, freebsdlike, netbsdlike))]
extern "C" {
    #[cfg_attr(target_os = "netbsd", link_name = "__pollts50")]
    pub fn ppoll(
        fds: *mut libc::pollfd,
        nfds: libc::nfds_t,
        timep: *const libc::timespec,
        sigmask: *const libc::sigset_t,
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
