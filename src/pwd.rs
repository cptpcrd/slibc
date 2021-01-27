use core::fmt;
use core::hash::{Hash, Hasher};

use crate::internal_prelude::*;

#[inline]
fn init_bufsize() -> usize {
    match unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) } {
        -1 => 1024,
        size => size as usize,
    }
}

const MAX_BUFSIZE: usize = 32768;

macro_rules! osstr_getter {
    ($name:ident, $field_name:ident) => {
        pub fn $name<'a>(&'a self) -> &'a OsStr {
            OsStr::from_bytes(unsafe { CStr::from_ptr(self.pwd.$field_name) }.to_bytes())
        }
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct Passwd {
    pwd: libc::passwd,
    #[allow(dead_code)]
    buf: Vec<u8>,
}

impl Passwd {
    #[inline]
    pub fn uid(&self) -> libc::uid_t {
        self.pwd.pw_uid
    }

    #[inline]
    pub fn gid(&self) -> libc::gid_t {
        self.pwd.pw_gid
    }

    osstr_getter!(name, pw_name);
    osstr_getter!(passwd, pw_passwd);
    osstr_getter!(gecos, pw_gecos);
    osstr_getter!(dir, pw_dir);
    osstr_getter!(shell, pw_shell);

    #[cfg(bsd)]
    osstr_getter!(class, pw_class);

    #[cfg(bsd)]
    #[inline]
    pub fn change(&self) -> libc::time_t {
        self.pwd.pw_change
    }

    #[cfg(bsd)]
    #[inline]
    pub fn expire(&self) -> libc::time_t {
        self.pwd.pw_expire
    }

    #[inline]
    fn lookup<F>(getpw: F) -> Result<Option<Self>>
    where
        F: Fn(*mut libc::passwd, *mut libc::c_char, usize, *mut *mut libc::passwd) -> libc::c_int,
    {
        let mut buf = Vec::with_capacity(init_bufsize());
        let mut pwd = MaybeUninit::uninit();
        let mut result = core::ptr::null_mut();

        loop {
            match getpw(
                pwd.as_mut_ptr(),
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.capacity(),
                &mut result,
            ) {
                0 => {
                    unsafe {
                        buf.set_len(buf.capacity());
                    }

                    return Ok(if result.is_null() {
                        None
                    } else {
                        Some(Self {
                            pwd: unsafe { pwd.assume_init() },
                            buf,
                        })
                    });
                }

                libc::ERANGE if buf.capacity() < MAX_BUFSIZE => buf.reserve(buf.capacity() * 2),

                eno => return Err(Error::from_code(eno)),
            }
        }
    }

    pub fn lookup_uid(uid: libc::uid_t) -> Result<Option<Self>> {
        unsafe {
            Self::lookup(
                |pwd: *mut libc::passwd,
                 buf: *mut libc::c_char,
                 buflen: usize,
                 result: *mut *mut libc::passwd| {
                    libc::getpwuid_r(uid, pwd, buf, buflen, result)
                },
            )
        }
    }

    pub fn lookup_name<N: AsPath>(name: N) -> Result<Option<Self>> {
        name.with_cstr(|name| unsafe {
            Self::lookup(
                |pwd: *mut libc::passwd,
                 buf: *mut libc::c_char,
                 buflen: usize,
                 result: *mut *mut libc::passwd| {
                    libc::getpwnam_r(name.as_ptr(), pwd, buf, buflen, result)
                },
            )
        })
    }
}

impl Clone for Passwd {
    fn clone(&self) -> Self {
        let mut buf = self.buf.clone();

        macro_rules! offset {
            ($ptr:expr) => {
                unsafe {
                    buf.as_mut_ptr()
                        .offset(($ptr as *mut u8).offset_from(self.buf.as_ptr()))
                        as *mut libc::c_char
                }
            };
        }

        let pwd = libc::passwd {
            pw_name: offset!(self.pwd.pw_name),
            pw_passwd: offset!(self.pwd.pw_passwd),
            pw_gecos: offset!(self.pwd.pw_gecos),
            pw_dir: offset!(self.pwd.pw_dir),
            pw_shell: offset!(self.pwd.pw_shell),
            pw_uid: self.pwd.pw_uid,
            pw_gid: self.pwd.pw_gid,
            #[cfg(bsd)]
            pw_class: offset!(self.pwd.pw_class),
            #[cfg(bsd)]
            pw_change: self.pwd.pw_change,
            #[cfg(bsd)]
            pw_expire: self.pwd.pw_expire,
            #[cfg(freebsdlike)]
            pw_fields: self.pwd.pw_fields,
        };

        Passwd { pwd, buf }
    }
}

impl PartialEq for Passwd {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(bsd)]
        if self.class() != other.class()
            || self.change() != other.change()
            || self.expire() != other.expire()
        {
            return false;
        }

        self.uid() == other.uid()
            && self.gid() == other.gid()
            && self.name() == other.name()
            && self.passwd() == other.passwd()
            && self.gecos() == other.gecos()
            && self.dir() == other.dir()
            && self.shell() == other.shell()
    }
}

impl Hash for Passwd {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.uid());
        state.write_u32(self.gid());
        state.write(self.name().as_bytes());
        state.write(self.passwd().as_bytes());
        state.write(self.gecos().as_bytes());
        state.write(self.dir().as_bytes());
        state.write(self.shell().as_bytes());

        #[cfg(bsd)]
        {
            state.write(self.class().as_bytes());
            state.write_i64(self.change() as i64);
            state.write_i64(self.expire() as i64);
        }
    }
}

impl Eq for Passwd {}

impl fmt::Debug for Passwd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ds = f.debug_struct("Passwd");

        ds.field("uid", &self.uid())
            .field("gid", &self.gid())
            .field("name", &self.name())
            .field("passwd", &self.passwd())
            .field("gecos", &self.gecos())
            .field("dir", &self.dir())
            .field("shell", &self.shell());

        #[cfg(bsd)]
        ds.field("class", &self.class())
            .field("change", &self.change())
            .field("expire", &self.expire());

        ds.finish()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct PasswdIter(());

impl PasswdIter {
    /// Create an iterator over all the password entries in the system.
    ///
    /// # Safety
    ///
    /// From the time this method is called, to the time the object returned goes out of scope (or
    /// is dropped), none of the following actions may be performed:
    ///
    /// - Calling this method to create another `PasswdIter` object.
    /// - Calling any of the following C functions:
    ///   - `setpwent()`
    ///   - `getpwent()`
    ///   - `getpwent_r()`
    ///   - `endpwent()`
    ///   - `getpwuid()`
    ///   - `getpwnam()`
    #[inline]
    pub unsafe fn new() -> Self {
        libc::setpwent();
        Self(())
    }
}

impl Iterator for PasswdIter {
    type Item = Result<Passwd>;

    #[allow(clippy::needless_return)]
    fn next(&mut self) -> Option<Result<Passwd>> {
        cfg_if::cfg_if! {
            if #[cfg(any(
                all(target_os = "linux", any(target_env = "", target_env = "gnu")),
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
            ))] {
                return unsafe {
                    match Passwd::lookup(
                        |pwd: *mut libc::passwd,
                         buf: *mut libc::c_char,
                         buflen: usize,
                         result: *mut *mut libc::passwd| {
                            libc::getpwent_r(pwd, buf, buflen, result)
                        },
                    ) {
                        Ok(Some(entry)) => Some(Ok(entry)),
                        Ok(None) => None,

                        Err(e) if e.code() == libc::ENOENT => None,
                        Err(e) => Some(Err(e)),
                    }
                };
            } else {
                return unsafe {
                    let eno_ptr = util::errno_ptr();
                    *eno_ptr = 0;

                    if let Some(pwd) = libc::getpwent().as_ref() {
                        let pw_name = CStr::from_ptr(pwd.pw_name).to_bytes();
                        let pw_passwd = CStr::from_ptr(pwd.pw_passwd).to_bytes();
                        let pw_gecos = CStr::from_ptr(pwd.pw_gecos).to_bytes();
                        let pw_dir = CStr::from_ptr(pwd.pw_dir).to_bytes();
                        let pw_shell = CStr::from_ptr(pwd.pw_shell).to_bytes();

                        #[cfg(bsd)]
                        let pw_class = CStr::from_ptr(pwd.pw_class).to_bytes();

                        let buflen = 5
                            + pw_name.len()
                            + pw_passwd.len()
                            + pw_gecos.len()
                            + pw_dir.len()
                            + pw_shell.len();
                        #[cfg(bsd)]
                        let buflen = buflen + pw_class.len() + 1;

                        let mut buf = Vec::with_capacity(buflen);
                        buf.resize(buflen, 0);

                        macro_rules! fill_buf {
                            ($offset:expr, $slice:expr) => {{
                                let offset = $offset;
                                let slice = $slice;
                                buf[offset..offset + slice.len()].copy_from_slice(slice);
                            }};
                        }

                        fill_buf!(0, pw_name);
                        fill_buf!(pw_name.len() + 1, pw_passwd);
                        fill_buf!(pw_name.len() + pw_passwd.len() + 2, pw_gecos);
                        fill_buf!(pw_name.len() + pw_passwd.len() + pw_gecos.len() + 3, pw_dir);
                        fill_buf!(
                            pw_name.len() + pw_passwd.len() + pw_gecos.len() + pw_dir.len() + 4,
                            pw_shell
                        );

                        #[cfg(bsd)]
                        fill_buf!(
                            pw_name.len() + pw_passwd.len() + pw_gecos.len() + pw_dir.len() + pw_shell.len() + 5,
                            pw_class
                        );

                        let new_pwd = libc::passwd {
                            pw_name: buf.as_mut_ptr() as *mut _,
                            pw_passwd: buf.as_mut_ptr().add(pw_name.len() + 1) as *mut _,
                            pw_gecos: buf.as_mut_ptr().add(
                                pw_name.len() + pw_passwd.len() + 2
                            ) as *mut _,
                            pw_dir: buf.as_mut_ptr().add(
                                pw_name.len()
                                + pw_passwd.len()
                                + pw_gecos.len()
                                + 3
                            ) as *mut _,
                            pw_shell: buf.as_mut_ptr().add(
                                pw_name.len()
                                + pw_passwd.len()
                                + pw_gecos.len()
                                + pw_dir.len()
                                + 4
                            ) as *mut _,
                            #[cfg(bsd)]
                            pw_class: buf.as_mut_ptr().add(
                                pw_name.len()
                                + pw_passwd.len()
                                + pw_gecos.len()
                                + pw_dir.len()
                                + pw_shell.len()
                                + 5
                            ) as *mut _,
                            pw_uid: pwd.pw_uid,
                            pw_gid: pwd.pw_gid,
                            #[cfg(bsd)]
                            pw_change: pwd.pw_change,
                            #[cfg(bsd)]
                            pw_expire: pwd.pw_expire,
                        };

                        let passwd = Passwd { pwd: new_pwd, buf };

                        debug_assert_eq!(passwd.name().as_bytes(), pw_name);
                        debug_assert_eq!(passwd.passwd().as_bytes(), pw_passwd);
                        debug_assert_eq!(passwd.gecos().as_bytes(), pw_gecos);
                        debug_assert_eq!(passwd.dir().as_bytes(), pw_dir);
                        debug_assert_eq!(passwd.shell().as_bytes(), pw_shell);

                        #[cfg(bsd)]
                        debug_assert_eq!(passwd.class().as_bytes(), pw_class);

                        Some(Ok(passwd))
                    } else {
                        match *eno_ptr {
                            0 => None,
                            eno => Some(Err(Error::from_code(eno))),
                        }
                    }
                };
            }
        }
    }
}

impl Drop for PasswdIter {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::endpwent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_root() {
        let root1 = Passwd::lookup_uid(0).unwrap().unwrap();
        let root2 = Passwd::lookup_name("root").unwrap().unwrap();

        #[cfg(feature = "std")]
        fn hash(pwd: &Passwd) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            pwd.hash(&mut hasher);
            hasher.finish()
        }

        #[cfg(feature = "std")]
        assert_eq!(hash(&root1), hash(&root2), "{:?} != {:?}", root1, root2);

        assert_eq!(root1, root2);

        for entry in [root1, root2].iter() {
            #[cfg(feature = "std")]
            assert_eq!(hash(&entry), hash(&entry));

            assert_eq!(entry, &entry.clone());

            assert_eq!(entry.uid(), 0);
            assert_eq!(entry.name(), OsStr::new("root"));
        }
    }

    #[test]
    fn test_lookup_noexist() {
        assert_eq!(Passwd::lookup_uid(libc::uid_t::MAX).unwrap(), None);
        assert_eq!(Passwd::lookup_name("NO_SUCH_USER_123456").unwrap(), None);
    }
}
