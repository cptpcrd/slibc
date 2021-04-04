use crate::internal_prelude::*;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum FlockOp {
    LOCK_SH = libc::LOCK_SH,
    LOCK_EX = libc::LOCK_EX,
    LOCK_UN = libc::LOCK_UN,
}

#[inline]
pub fn flock(fd: RawFd, op: FlockOp) -> Result<()> {
    Error::unpack_nz(unsafe { libc::flock(fd, op as _) })
}
