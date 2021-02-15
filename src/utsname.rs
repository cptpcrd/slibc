use crate::internal_prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Utsname(libc::utsname);

macro_rules! utsname_func {
    ($name:ident) => {
        #[inline]
        pub fn $name(&self) -> &OsStr {
            util::osstr_from_buf(util::cvt_char_buf(&self.0.$name))
        }
    };
}

impl Utsname {
    utsname_func!(sysname);
    utsname_func!(nodename);
    utsname_func!(release);
    utsname_func!(version);
    utsname_func!(machine);

    #[cfg(target_os = "linux")]
    #[cfg_attr(docsrs, cfg(target_os = "linux"))]
    utsname_func!(domainname);
}

#[inline]
pub fn uname() -> Result<Utsname> {
    let mut buf = MaybeUninit::uninit();
    Error::unpack_nz(unsafe { libc::uname(buf.as_mut_ptr()) })?;
    Ok(Utsname(unsafe { buf.assume_init() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uname() {
        let utsname = uname().unwrap();

        #[cfg(target_os = "linux")]
        assert_eq!(utsname.sysname(), OsStr::new("Linux"));
        #[cfg(target_os = "freebsd")]
        assert_eq!(utsname.sysname(), OsStr::new("FreeBSD"));

        let mut buf = [0; 4096];
        let hostname = crate::gethostname(&mut buf).unwrap();
        assert_eq!(utsname.nodename(), OsStr::from_bytes(hostname.to_bytes()));

        #[cfg(feature = "alloc")]
        assert_eq!(
            utsname.nodename(),
            OsStr::from_bytes(crate::gethostname_alloc().unwrap().to_bytes())
        );

        #[cfg(target_os = "linux")]
        {
            let domainname = crate::getdomainname(&mut buf).unwrap();
            assert_eq!(
                utsname.domainname(),
                OsStr::from_bytes(domainname.to_bytes())
            );

            #[cfg(feature = "alloc")]
            assert_eq!(
                utsname.domainname(),
                OsStr::from_bytes(crate::getdomainname_alloc().unwrap().to_bytes())
            );
        }
    }
}
