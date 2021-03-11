#[allow(unused_imports)]
use crate::internal_prelude::*;

#[inline]
pub fn sched_yield() {
    let ret = unsafe { libc::sched_yield() };
    debug_assert_eq!(ret, 0);
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[derive(Copy, Clone)]
pub struct CpuSet(libc::cpu_set_t);

#[cfg(target_os = "linux")]
impl CpuSet {
    #[inline]
    pub fn new() -> Self {
        let mut set = unsafe { core::mem::zeroed() };
        unsafe {
            libc::CPU_ZERO(&mut set);
        }
        Self(set)
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            libc::CPU_ZERO(&mut self.0);
        }
    }

    #[inline]
    pub fn count(&self) -> usize {
        unsafe { libc::CPU_COUNT(&self.0) as _ }
    }

    #[inline]
    pub fn add(&mut self, cpu: u32) {
        unsafe {
            libc::CPU_SET(cpu as usize, &mut self.0);
        }
    }

    #[inline]
    pub fn remove(&mut self, cpu: u32) {
        unsafe {
            libc::CPU_CLR(cpu as usize, &mut self.0);
        }
    }

    #[inline]
    pub fn contains(&self, cpu: u32) -> bool {
        if cpu >= core::mem::size_of::<libc::cpu_set_t>() as u32 * 8 {
            return false;
        }

        unsafe { libc::CPU_ISSET(cpu as usize, &self.0) }
    }

    #[inline]
    pub fn iter(&self) -> CpuSetIter {
        self.into_iter()
    }
}

#[cfg(target_os = "linux")]
impl Default for CpuSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl core::fmt::Debug for CpuSet {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(target_os = "linux")]
impl PartialEq for CpuSet {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { libc::CPU_EQUAL(&self.0, &other.0) }
    }
}

#[cfg(target_os = "linux")]
impl Eq for CpuSet {}

#[cfg(target_os = "linux")]
impl IntoIterator for CpuSet {
    type Item = u32;
    type IntoIter = CpuSetIter;

    #[inline]
    fn into_iter(self) -> CpuSetIter {
        CpuSetIter {
            set: self,
            n: self.count(),
            i: 0,
        }
    }
}

#[cfg(target_os = "linux")]
impl core::iter::FromIterator<u32> for CpuSet {
    #[inline]
    fn from_iter<I: IntoIterator<Item = u32>>(it: I) -> Self {
        let mut set = Self::new();
        set.extend(it);
        set
    }
}

#[cfg(target_os = "linux")]
impl Extend<u32> for CpuSet {
    #[inline]
    fn extend<I: IntoIterator<Item = u32>>(&mut self, it: I) {
        for cpu in it.into_iter() {
            self.add(cpu);
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
pub struct CpuSetIter {
    set: CpuSet,
    /// The number of CPUs in the remaining portion of the set
    n: usize,
    /// The current CPU number
    i: u32,
}

#[cfg(target_os = "linux")]
impl Iterator for CpuSetIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        self.n = self.n.checked_sub(1)?;

        loop {
            let i = self.i;
            self.i += 1;

            if self.set.contains(i) {
                return Some(i);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

#[cfg(target_os = "linux")]
impl ExactSizeIterator for CpuSetIter {
    #[inline]
    fn len(&self) -> usize {
        self.n
    }
}

/// Set the CPU affinity mask of the process specified by `pid`.
///
/// If `pid` is 0, this operates on the current process.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn sched_setaffinity(pid: libc::pid_t, mask: &CpuSet) -> Result<()> {
    Error::unpack_nz(unsafe {
        libc::sched_setaffinity(pid, core::mem::size_of::<libc::cpu_set_t>(), &mask.0)
    })
}

/// Get the CPU affinity mask of the process specified by `pid`.
///
/// If `pid` is 0, this operates on the current process.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn sched_getaffinity(pid: libc::pid_t) -> Result<CpuSet> {
    let mut mask = MaybeUninit::uninit();
    Error::unpack_nz(unsafe {
        libc::sched_getaffinity(
            pid,
            core::mem::size_of::<libc::cpu_set_t>(),
            mask.as_mut_ptr(),
        )
    })?;
    Ok(CpuSet(unsafe { mask.assume_init() }))
}

/// Get the CPU that this thread is currently running on.
#[cfg_attr(docsrs, doc(cfg(any(target_os = "linux", target_os = "android"))))]
#[cfg(linuxlike)]
#[inline]
pub fn sched_getcpu() -> Result<u32> {
    Error::unpack(unsafe { libc::sched_getcpu() }).map(|cpu| cpu as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sched_yield() {
        sched_yield();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_cpuset() {
        fn check_empty(set: &CpuSet) {
            assert_eq!(set, &CpuSet::default());

            check_values(set, &[]);
        }

        fn check_values(set: &CpuSet, cpus: &[u32]) {
            for cpu in 0..(core::mem::size_of::<CpuSet>() as u32 * 16) {
                if cpus.contains(&cpu) {
                    assert!(set.contains(cpu), "{}", cpu);
                } else {
                    assert!(!set.contains(cpu), "{}", cpu);
                }
            }

            assert_eq!(set.count(), cpus.len());

            assert_eq!(set.iter().len(), cpus.len());
            assert_eq!(set.iter().count(), cpus.len());
            assert_eq!(set.iter().size_hint(), (cpus.len(), Some(cpus.len())));
            assert!(
                set.iter().eq(cpus.iter().copied()),
                "{:?} != {:?}",
                set,
                cpus
            );
        }

        let mut set;

        set = CpuSet::new();
        check_empty(&set);

        set.add(0);
        check_values(&set, &[0]);
        set.add(100);
        check_values(&set, &[0, 100]);
        set.remove(0);
        check_values(&set, &[100]);

        set.clear();
        check_empty(&set);

        set = [0; 0].iter().cloned().collect::<CpuSet>();
        check_empty(&set);
        set = [0, 1, 10].iter().cloned().collect::<CpuSet>();
        check_values(&set, &[0, 1, 10]);
    }

    #[cfg(all(target_os = "linux", feature = "alloc"))]
    #[test]
    fn test_cpuset_debug() {
        let mut set = CpuSet::new();
        assert_eq!(format!("{:?}", set), "{}");
        set.add(0);
        assert_eq!(format!("{:?}", set), "{0}");
        set.add(10);
        assert_eq!(format!("{:?}", set), "{0, 10}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_sched_affinity() {
        let pid = crate::getpid();

        let affinity = sched_getaffinity(0).unwrap();
        assert_eq!(affinity, sched_getaffinity(pid).unwrap());

        // No change after setting it
        sched_setaffinity(0, &affinity).unwrap();
        assert_eq!(affinity, sched_getaffinity(0).unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_sched_getcpu() {
        let cpu = sched_getcpu().unwrap();
        let affinity = sched_getaffinity(0).unwrap();
        assert!(affinity.contains(cpu));
    }
}