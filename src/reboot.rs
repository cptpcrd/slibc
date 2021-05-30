use crate::internal_prelude::*;

/// Change the behavior of the CAD keystroke.
///
/// Requires the `CAP_SYS_BOOT` capability.
///
/// The CAD keystroke defaults to Control+Alt+Delete, but can be changed with `loadkeys(1)`. If
/// `cad` is `true`, the CAD keystroke will immediately restart the system. If it is `false`, the
/// CAD keystroke will cause `SIGINT` to be sent to PID 1.
///
/// This function will fail with `EINVAL` if called inside a non-initial PID namespace.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn reboot_set_cad(cad: bool) -> Result<()> {
    Error::unpack_nz(unsafe {
        sys::reboot(if cad {
            sys::RB_ENABLE_CAD
        } else {
            sys::RB_DISABLE_CAD
        })
    })
}

/// Values that can be passed to [`reboot()`] indicating which reboot/halt/etc. action should be
/// taken.
///
/// # Notes
///
/// - On Linux, see `reboot(2)` for information on behavior inside PID namespaces.
/// - On some of the BSDs, flags such as `RB_HALT` and `RB_POWER_OFF` have slightly different names
///   and/or behavior. For consistency, these "modes" are altered to do the tasks exactly as
///   described here.
///
///   For example, on OpenBSD, [`Self::POWER_OFF`] is set to `RB_POWERDOWN | RB_HALT`.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(i32)]
pub enum RebootMode {
    /// Restart the system.
    AUTOBOOT = sys::RB_AUTOBOOT,

    /// Halt the system.
    HALT_SYSTEM = sys::RB_HALT_SYSTEM,

    /// Attempt to power off the system.
    ///
    /// This may be translated to [`Self::HALT_SYSTEM`] if e.g. the system lacks hardware support
    /// for shutting off the power.
    #[cfg_attr(docsrs, doc(cfg(not(any(target_os = "macos", target_os = "ios")))))]
    #[cfg(not(apple))]
    POWER_OFF = sys::RB_POWER_OFF,

    /// Execute a kernel loaded with `kexec_load(2)`.
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
    #[cfg(linuxlike)]
    KEXEC = sys::RB_KEXEC,

    /// Try to turn off the power and turn it back on.
    ///
    /// This requires hardware support. See `reboot(2)` for more information.
    #[cfg_attr(docsrs, doc(cfg(target_os = "freebsd")))]
    #[cfg(target_os = "freebsd")]
    POWERCYCLE = sys::RB_POWERCYCLE,
}

/// Reboot, halt, etc. the system.
///
/// Requires the `CAP_SYS_BOOT` capability. See [`RebootMode`] for more information on values of
/// `cmd`.
///
/// # WARNING
///
/// This is a very low-level function that talks directly to the kernel. If you want to shutdown or
/// reboot the system normally, you should run e.g. `shutdown -h now` instead.
///
/// If you must call this function directly (e.g. when writing your own init system), make sure to
/// take all appropriate steps (killing processes, unmounting filesystems, calling
/// [`sync()`](./fn.sync.html) etc.) first!
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn reboot(cmd: RebootMode) -> Result<core::convert::Infallible> {
    unsafe {
        sys::reboot(cmd as _);
    }
    Err(Error::last())
}

#[cfg(bsd)]
macro_rules! bsd_declare_reboot_flags {
    ($(
        #[cfg($cfg:meta)]
        $(
            $(#[doc = $doc:literal])*
            $name:ident = $sys_name:ident,
        )+
    )*) => {
        bitflags::bitflags! {
            /// Flags for [`reboot()`].
            ///
            /// See the OS-specific `reboot(2)` for more information on these flags.
            ///
            /// WARNING: The BSDs do little or no validation to ensure that incompatible
            /// flags/modes (see [`RebootMode`]) are not specified together. Be careful!
            #[cfg_attr(
                docsrs,
                doc(cfg(any(
                    target_os = "freebsd",
                    target_os = "dragonfly",
                    target_os = "openbsd",
                    target_os = "netbsd",
                    target_os = "macos",
                    target_os = "ios",
                )))
            )]
            pub struct RebootFlags: libc::c_int {
                $($(
                    #[cfg($cfg)]
                    #[cfg_attr(docsrs, doc(cfg($cfg)))]
                    $(#[doc = $doc])*
                    const $name = sys::$sys_name as _;
                )*)*
            }
        }
    };
}

#[cfg(bsd)]
bsd_declare_reboot_flags! {
    #[cfg(all())]
    /// Prompt the user for the root partition.
    ASKNAME = RB_ASKNAME,
    /// Tell `init` to boot into single user mode.
    SINGLE = RB_SINGLE,
    /// Don't `sync()` the disks before halting/rebooting.
    ///
    /// Unlike Linux, on macOS/\*BSD `reboot(2)` normally *does* sync the disks.
    NOSYNC = RB_NOSYNC,

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    KDB = RB_KDB,
    /// Mount the new root partition as read-only.
    ///
    /// On some of the BSDs, this is the default and this flag is ignored.
    RDONLY = RB_RDONLY,
    DUMP = RB_DUMP,

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    SAFEBOOT = RB_SAFEBOOT,
    UPSDELAY = RB_UPSDELAY,
    QUICK = RB_QUICK,

    #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly"))]
    MINIROOT = RB_MINIROOT,

    #[cfg(any(target_os = "openbsd", target_os = "freebsd", target_os = "dragonfly"))]
    DFLTROOT = RB_DFLTROOT,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    SERIAL = RB_SERIAL,
    CDROM = RB_CDROM,
    GDB = RB_GDB,
    MUTE = RB_MUTE,
    PAUSE = RB_PAUSE,
    VERBOSE = RB_VERBOSE,

    #[cfg(target_os = "freebsd")]
    PROBE = RB_PROBE,
    MULTIPLE = RB_MULTIPLE,
    REROOT = RB_REROOT,

    #[cfg(target_os = "dragonfly")]
    VIDEO = RB_VIDEO,
    SELFTEST = RB_SELFTEST,

    #[cfg(target_os = "openbsd")]
    CONFIG = RB_CONFIG,
    /// Don't update the hardware clock from the system clock.
    TIMEBAD = RB_TIMEBAD,
    SERCONS = RB_SERCONS,
    USERREQ = RB_USERREQ,
    RESET = RB_RESET,
    GOODRANDOM = RB_GOODRANDOM,

    #[cfg(target_os = "netbsd")]
    USERCONF = RB_USERCONF,
}

/// Reboot, halt, etc. the system.
///
/// Can only be called by `root`. See [`RebootMode`] and [`RebootFlags`] for more information on
/// values of `cmd` and `flags`.
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
    )))
)]
#[cfg(bsd)]
#[inline]
pub fn reboot(cmd: RebootMode, flags: RebootFlags) -> Result<core::convert::Infallible> {
    #[cfg(target_os = "netbsd")]
    unsafe {
        sys::reboot(cmd as i32 | flags.bits(), core::ptr::null_mut());
    }
    #[cfg(not(target_os = "netbsd"))]
    unsafe {
        sys::reboot(cmd as i32 | flags.bits());
    }

    Err(Error::last())
}

/// Call `reboot(RB_SW_SUSPEND)`.
///
/// Requires the `CAP_SYS_BOOT` capability.
///
/// This is separate from [`reboot()`] because, unlike for the values listed in [`RebootMode`],
/// `reboot(RB_SW_SUSPEND)` *does* return even if successful.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn reboot_sw_suspend() -> Result<()> {
    Error::unpack_nz(unsafe { sys::reboot(sys::RB_SW_SUSPEND) })
}
