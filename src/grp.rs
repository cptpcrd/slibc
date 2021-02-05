use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

use crate::internal_prelude::*;

#[inline]
fn init_bufsize() -> usize {
    match unsafe { libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) } {
        -1 => 1024,
        size => size as usize,
    }
}

const MAX_BUFSIZE: usize = 32768;

macro_rules! osstr_getter {
    ($name:ident, $field_name:ident) => {
        pub fn $name<'a>(&'a self) -> &'a OsStr {
            OsStr::from_bytes(unsafe { CStr::from_ptr(self.grp.$field_name) }.to_bytes())
        }
    };
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct Group {
    grp: libc::group,
    #[allow(dead_code)]
    buf: Vec<u8>,
}

impl Group {
    #[inline]
    pub fn gid(&self) -> libc::gid_t {
        self.grp.gr_gid
    }

    osstr_getter!(name, gr_name);
    osstr_getter!(passwd, gr_passwd);

    pub fn members(&self) -> GroupMemberIter<'_> {
        GroupMemberIter {
            mem_ptr: self.grp.gr_mem,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn lookup<F>(getgr: F) -> Result<Option<Self>>
    where
        F: Fn(*mut libc::group, *mut libc::c_char, usize, *mut *mut libc::group) -> libc::c_int,
    {
        let mut buf = Vec::with_capacity(init_bufsize());
        let mut grp = MaybeUninit::uninit();
        let mut result = core::ptr::null_mut();

        loop {
            match getgr(
                grp.as_mut_ptr(),
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
                            grp: unsafe { grp.assume_init() },
                            buf,
                        })
                    });
                }

                libc::ERANGE if buf.capacity() < MAX_BUFSIZE => buf.reserve(buf.capacity() * 2),

                eno => return Err(Error::from_code(eno)),
            }
        }
    }

    pub fn lookup_gid(gid: libc::gid_t) -> Result<Option<Self>> {
        unsafe {
            Self::lookup(
                |grp: *mut libc::group,
                 buf: *mut libc::c_char,
                 buflen: usize,
                 result: *mut *mut libc::group| {
                    libc::getgrgid_r(gid, grp, buf, buflen, result)
                },
            )
        }
    }

    pub fn lookup_name<N: AsPath>(name: N) -> Result<Option<Self>> {
        name.with_cstr(|name| unsafe {
            Self::lookup(
                |grp: *mut libc::group,
                 buf: *mut libc::c_char,
                 buflen: usize,
                 result: *mut *mut libc::group| {
                    libc::getgrnam_r(name.as_ptr(), grp, buf, buflen, result)
                },
            )
        })
    }
}

impl Clone for Group {
    fn clone(&self) -> Self {
        let mut buf = self.buf.clone();

        macro_rules! offset {
            ($ptr:expr) => {
                unsafe {
                    buf.as_mut_ptr()
                        .offset(($ptr as *mut u8).offset_from(self.buf.as_ptr())) as *mut _
                }
            };
        }

        let grp = libc::group {
            gr_name: offset!(self.grp.gr_name),
            gr_passwd: offset!(self.grp.gr_passwd),
            gr_gid: self.grp.gr_gid,
            gr_mem: offset!(self.grp.gr_mem),
        };

        Group { grp, buf }
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.gid() == other.gid()
            && self.name() == other.name()
            && self.passwd() == other.passwd()
            && self.members().eq(other.members())
    }
}

impl Hash for Group {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.gid());
        state.write(self.name().as_bytes());
        state.write(self.passwd().as_bytes());

        for member in self.members() {
            state.write(member.as_bytes());
        }
    }
}

impl Eq for Group {}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Group")
            .field("gid", &self.gid())
            .field("name", &self.name())
            .field("passwd", &self.passwd())
            .field("members", &self.members().collect::<Vec<_>>())
            .finish()
    }
}

/// An iterator over all the members of a given [`Group`].
///
/// This is created by [`Group::members()`]. It yields `OsStr`s representing the name of each user
/// that is a member of the group.
pub struct GroupMemberIter<'a> {
    mem_ptr: *mut *mut libc::c_char,
    phantom: PhantomData<&'a Group>,
}

impl<'a> Iterator for GroupMemberIter<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { *self.mem_ptr }.is_null() {
            return None;
        }

        let member = OsStr::from_bytes(unsafe { CStr::from_ptr(*self.mem_ptr) }.to_bytes());
        self.mem_ptr = unsafe { self.mem_ptr.add(1) };
        Some(member)
    }
}

/// An iterator over the entries in the group database.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct GroupIter(());

impl GroupIter {
    /// Create an iterator over all the group entries in the system.
    ///
    /// # Safety
    ///
    /// From the time this method is called, to the time the object returned goes out of scope (or
    /// is dropped), none of the following actions may be performed (in any thread):
    ///
    /// - Calling this method to create another `GroupIter` object.
    /// - Calling any of the following C functions:
    ///   - `setgrent()`
    ///   - `getgrent()`
    ///   - `getgrent_r()`
    ///   - `endgrent()`
    ///   - `getgrgid()`
    ///   - `getgrnam()`
    #[inline]
    pub unsafe fn new() -> Self {
        libc::setgrent();
        Self(())
    }
}

impl Iterator for GroupIter {
    type Item = Result<Group>;

    #[allow(clippy::needless_return)]
    fn next(&mut self) -> Option<Result<Group>> {
        cfg_if::cfg_if! {
            if #[cfg(any(
                all(target_os = "linux", any(target_env = "", target_env = "gnu")),
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
            ))] {
                return unsafe {
                    match Group::lookup(
                        |grp: *mut libc::group,
                         buf: *mut libc::c_char,
                         buflen: usize,
                         result: *mut *mut libc::group| {
                            libc::getgrent_r(grp, buf, buflen, result)
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

                    if let Some(grp) = libc::getgrent().as_ref() {
                        let gr_name = CStr::from_ptr(grp.gr_name).to_bytes();
                        let gr_passwd = CStr::from_ptr(grp.gr_passwd).to_bytes();

                        let ptrsize = core::mem::size_of::<*mut libc::c_char>();

                        let mut gr_mem_len = 0;
                        let mut mem_ptr = grp.gr_mem;
                        let mut extra_size = 0;
                        while !(*mem_ptr).is_null() {
                            gr_mem_len += 1;
                            extra_size += CStr::from_ptr(*mem_ptr).to_bytes_with_nul().len();
                            mem_ptr = mem_ptr.add(1);
                        }

                        let buflen = 2
                            + gr_name.len()
                            + gr_passwd.len()
                            + (gr_mem_len + 1) * ptrsize
                            + extra_size;

                        let mut buf = Vec::with_capacity(buflen);
                        buf.resize(buflen, 0);

                        macro_rules! fill_buf {
                            ($offset:expr, $slice:expr) => {{
                                let offset = $offset;
                                let slice = $slice;
                                buf[offset..offset + slice.len()].copy_from_slice(slice);
                            }};
                        }

                        fill_buf!(0, gr_name);
                        fill_buf!(gr_name.len() + 1, gr_passwd);

                        let mut buf_memlist_offset = gr_name.len() + gr_passwd.len() + 2;
                        let mut buf_members_offset = buf_memlist_offset
                            + (gr_mem_len + 1) * ptrsize;

                        let mut mem_ptr = grp.gr_mem;
                        while !(*mem_ptr).is_null() {
                            let ptr = buf.as_ptr().add(buf_members_offset);
                            buf[buf_memlist_offset..buf_memlist_offset + ptrsize]
                                .copy_from_slice(&(ptr as usize).to_ne_bytes());

                            debug_assert!(buf_memlist_offset + ptrsize <= buf.len());
                            debug_assert!(buf_members_offset <= buf.len());

                            let member_bytes = CStr::from_ptr(*mem_ptr).to_bytes_with_nul();
                            buf[buf_members_offset..buf_members_offset + member_bytes.len()]
                                .copy_from_slice(member_bytes);

                            buf_memlist_offset += ptrsize;
                            buf_members_offset += member_bytes.len();
                            mem_ptr = mem_ptr.add(1);
                        }

                        debug_assert_eq!(buf.len(), buf_members_offset);
                        debug_assert_eq!(
                            buf_memlist_offset,
                            gr_name.len() + gr_passwd.len() + 2 + gr_mem_len * ptrsize
                        );
                        debug_assert_eq!(
                            buf[buf_memlist_offset..buf_memlist_offset + ptrsize],
                            [0; core::mem::size_of::<*mut libc::c_char>()],
                        );

                        let new_grp = libc::group {
                            gr_name: buf.as_mut_ptr() as *mut _,
                            gr_passwd: buf.as_mut_ptr().add(gr_name.len() + 1) as *mut _,
                            gr_gid: grp.gr_gid,
                            gr_mem: buf.as_mut_ptr()
                                .add(gr_name.len() + gr_passwd.len() + 2) as *mut _,
                        };

                        let group = Group { grp: new_grp, buf };

                        debug_assert_eq!(group.name().as_bytes(), gr_name);
                        debug_assert_eq!(group.passwd().as_bytes(), gr_passwd);

                        Some(Ok(group))
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

impl Drop for GroupIter {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::endgrent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // macOS behaves strangely in situations like this

    #[test]
    fn test_lookup_cur() {
        let gid = crate::getgid();

        let cur1 = Group::lookup_gid(gid).unwrap().unwrap();
        let cur2 = Group::lookup_name(cur1.name()).unwrap().unwrap();

        #[cfg(feature = "std")]
        fn hash(grp: &Group) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            grp.hash(&mut hasher);
            hasher.finish()
        }

        #[cfg(feature = "std")]
        assert_eq!(hash(&cur1), hash(&cur2), "{:?} != {:?}", cur1, cur2);

        assert_eq!(cur1, cur2);

        for entry in [cur1, cur2].iter() {
            #[cfg(feature = "std")]
            assert_eq!(hash(&entry), hash(&entry));

            assert_eq!(entry, &entry.clone());

            assert_eq!(entry.gid(), gid);
        }
    }

    #[test]
    fn test_lookup_noexist() {
        assert_eq!(Group::lookup_gid(libc::gid_t::MAX).unwrap(), None);
        assert_eq!(Group::lookup_name("NO_SUCH_USER_123456").unwrap(), None);
    }
}
