use core::fmt;

use crate::internal_prelude::*;

/// System information returned by [`sysinfo()`].
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[derive(Copy, Clone)]
pub struct SysInfo(libc::sysinfo);

macro_rules! sysinfo_raw_getters {
    ($($(#[doc = $doc:literal])* $name:ident, $type:ty;)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> $type {
                self.0.$name as $type
            }
        )*
    };
}

macro_rules! sysinfo_mem_getters {
    ($($(#[doc = $doc:literal])* $name:ident,)*) => {
        $(
            $(#[doc = $doc])*
            #[inline]
            pub fn $name(&self) -> u64 {
                (self.0.$name as u64) * (self.0.mem_unit as u64)
            }
        )*
    };
}

impl SysInfo {
    sysinfo_raw_getters! {
        /// Get the time in seconds since boot.
        ///
        /// This includes time that the system was suspended.
        uptime, u64;

        /// Get the number of currently running processes.
        procs, u32;
    }

    /// Get the 1, 5, and 15 minute load averages as raw numbers.
    ///
    /// These values are the raw numbers reported by the kernel. It's recommended to use
    /// [`loads()`](#method.loads) instead.
    #[inline]
    pub fn loads_raw(&self) -> (usize, usize, usize) {
        (
            self.0.loads[0] as usize,
            self.0.loads[1] as usize,
            self.0.loads[2] as usize,
        )
    }

    /// Get the 1, 5, and 15 minute load averages as decimal numbers.
    ///
    /// Unlike with [`loads_raw()`](#method.loads_raw), these are on the same scale as the numbers
    /// in `/proc/loadavg` or printed by `uptime`.
    #[inline]
    pub fn loads(&self) -> (f32, f32, f32) {
        (
            self.0.loads[0] as f32 / 65536.0,
            self.0.loads[1] as f32 / 65536.0,
            self.0.loads[2] as f32 / 65536.0,
        )
    }

    sysinfo_mem_getters! {
        /// Get the total amount of usable RAM.
        totalram,
        /// Get the amount of free RAM.
        freeram,
        /// Get the amount of shared memory.
        sharedram,
        /// Get the amount of memory used by buffers.
        bufferram,

        /// Get the amount of available swap space.
        totalswap,
        /// Get the amount of free swap space.
        freeswap,

        /// Get the total amount of ["high memory"].
        ///
        /// ["high memory"]: https://www.kernel.org/doc/html/latest/vm/highmem.html
        totalhigh,
        /// Get the amount of free ["high memory"].
        ///
        /// ["high memory"]: https://www.kernel.org/doc/html/latest/vm/highmem.html
        freehigh,
    }
}

impl fmt::Debug for SysInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SysInfo")
            .field("uptime", &self.uptime())
            .field("loads_raw", &self.loads_raw())
            .field("loads", &self.loads())
            .field("procs", &self.procs())
            .field("totalram", &self.totalram())
            .field("freeram", &self.freeram())
            .field("sharedram", &self.sharedram())
            .field("bufferram", &self.bufferram())
            .field("totalswap", &self.totalswap())
            .field("freeswap", &self.freeswap())
            .field("totalhigh", &self.totalhigh())
            .field("freehigh", &self.freehigh())
            .finish()
    }
}

/// Get information on memory usage/swap usage/system load.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[inline]
pub fn sysinfo() -> Result<SysInfo> {
    let mut buf = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::sysinfo(buf.as_mut_ptr()) })?;
    Ok(SysInfo(unsafe { buf.assume_init() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime() {
        let info = sysinfo().unwrap();

        let boot_time = crate::clock_gettime(crate::ClockId::BOOTTIME).unwrap();

        assert!(
            ((boot_time.tv_sec as i64) - (info.uptime() as i64)).abs() <= 1,
            "{:?} != {}",
            boot_time,
            info.uptime(),
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_totalram() {
        use std::fs;
        use std::io;
        use std::io::prelude::*;

        let f = io::BufReader::new(fs::File::open("/proc/meminfo").unwrap());
        let mut totalram: Option<u64> = None;
        for line in f.lines() {
            let line = line.unwrap();
            if line.starts_with("MemTotal:") && line.ends_with(" kB") {
                totalram = Some(line[9..line.len() - 3].trim().parse().unwrap());
                break;
            }
        }
        let totalram = totalram.unwrap() * 1024;

        let info = sysinfo().unwrap();
        assert_eq!(info.totalram(), totalram);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_loads() {
        let info = sysinfo().unwrap();
        let info_loads = info.loads();

        let line = std::fs::read_to_string("/proc/loadavg").unwrap();
        let mut it = line.split_whitespace();
        let loads = (
            it.next().unwrap().parse().unwrap(),
            it.next().unwrap().parse().unwrap(),
            it.next().unwrap().parse().unwrap(),
        );

        fn isclose(val1: f32, val2: f32) -> bool {
            (val1 - val2).abs() <= 0.01
        }

        assert!(
            isclose(info_loads.0, loads.0),
            "{} != {}",
            info_loads.0,
            loads.0,
        );
        assert!(
            isclose(info_loads.1, loads.1),
            "{} != {}",
            info_loads.0,
            loads.0,
        );
        assert!(
            isclose(info_loads.2, loads.2),
            "{} != {}",
            info_loads.0,
            loads.0,
        );
    }
}
