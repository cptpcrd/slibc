use crate::internal_prelude::*;

/// Given a master pseudoterminal device, obtain the name of the corresponding slave pseudoterminal
/// device.
///
/// **WARNING**: It is **highly recommended** to use [`ptsname_r()`] instead on platforms where it
/// is supported (like Linux).
///
/// # Safety
///
/// The string returned by this function is only valid until the next call to `ptsname()`. That
/// means that not only is the arbitrary `'a` lifetime not accurate, but calling this in a
/// multithreaded program is not safe.
///
/// Again, it is recommended to use [`ptsname_r()`] instead if it is supported.
#[inline]
pub unsafe fn ptsname<'a>(fd: RawFd) -> Result<&'a CStr> {
    let ptr = libc::ptsname(fd);

    if ptr.is_null() {
        Err(Error::last())
    } else {
        Ok(CStr::from_ptr(ptr))
    }
}

/// Given a master pseudoterminal device, obtain the name of the corresponding slave pseudoterminal
/// device.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn ptsname_r(fd: RawFd, buf: &mut [u8]) -> Result<&CStr> {
    match unsafe { libc::ptsname_r(fd, buf.as_mut_ptr() as *mut _, buf.len()) } {
        0 => Ok(util::cstr_from_buf(buf).unwrap()),
        eno => Err(Error::from_code(eno)),
    }
}

#[cfg(target_os = "linux")]
bitflags::bitflags! {
    /// Flags for [`getrandom()`].
    #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
    #[derive(Default)]
    pub struct GrndFlags: libc::c_uint {
        /// Obtain the random data from the `random` source (i.e. `/dev/random`) instead of the
        /// `urandom` source (i.e. `/dev/urandom`) source.
        ///
        /// The `random` source is more limited in its available entropy, and it may not be able to
        /// fill the supplied buffer if not enough bytes are available.
        const RANDOM = libc::GRND_RANDOM;
        /// Fail with `EAGAIN` isntead of blocking if insufficient entropy is available.
        const NONBLOCK = libc::GRND_NONBLOCK;
    }
}

/// Fill a buffer with random data.
///
/// If this function returns `Ok(n)`, the first `n` bytes of the given `buf` have been filled with
/// random bytes.
///
/// When reading from the `urandom` source (i.e. if [`GrndFlags::RANDOM`] is not specified), reads
/// of up to 256 bytes will not be interrupted by signals and will always return as many bytes as
/// requested. Larger requests, however, may return a partially filled buffer or fail with `EINTR`.
///
/// See [`GrndFlags`] for more information.
///
/// This is only available on Linux 3.17 and newer.
#[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
#[cfg(target_os = "linux")]
#[inline]
pub fn getrandom(buf: &mut [u8], flags: GrndFlags) -> Result<usize> {
    Error::unpack_size(unsafe {
        libc::getrandom(
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            flags.bits(),
        )
    })
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getrandom() {
        let mut buf = [0; 256];
        assert_eq!(getrandom(&mut buf, GrndFlags::default()).unwrap(), 256);
    }
}
