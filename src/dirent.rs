use crate::internal_prelude::*;

use core::ptr::NonNull;

/// A directory stream open for iterating over the entries in a directory.
///
/// Note that this iterator will yield entries for `.` and `..` if they are returned by the OS.
/// This behavior is intentional and will not be changed.
#[derive(Debug)]
pub struct Dir(NonNull<libc::DIR>);

impl Dir {
    /// Create a new directory stream for the directory with the given `path`.
    #[inline]
    pub fn open<P: AsPath>(path: P) -> Result<Self> {
        path.with_cstr(|path| Error::unpack_ptr(unsafe { libc::opendir(path.as_ptr()) }).map(Self))
    }

    /// Create a new directory stream for the directory referred to by the open file descriptor
    /// `fd`.
    ///
    /// `fd` should be a valid file descriptor open to a directory. It should NOT be open for
    /// "searching" (e.g. whichever of `O_SEARCH`, `O_EXEC`, or `O_PATH` the current platform might
    /// support).
    ///
    /// # Safety
    ///
    /// `fd` will be consumed by the new directory stream.
    #[inline]
    pub unsafe fn fdopen(fd: RawFd) -> Result<Self> {
        Error::unpack_ptr(libc::fdopendir(fd)).map(Self)
    }

    /// Rewind to the start of this directory.
    #[inline]
    pub fn rewind(&mut self) {
        unsafe {
            libc::rewinddir(self.0.as_ptr());
        }
    }

    /// Get the file descriptor used internally by the directory stream.
    #[inline]
    pub fn fd(&self) -> RawFd {
        unsafe { libc::dirfd(self.0.as_ptr()) }
    }

    /// Retrieve information about the directory represented by this directory stream.
    #[inline]
    pub fn stat(&self) -> Result<crate::Stat> {
        crate::fstat(self.fd())
    }

    /// Retrieve information about the given file within this directory.
    #[inline]
    pub fn fstatat<P: AsPath>(&self, path: P, flags: crate::AtFlag) -> Result<crate::Stat> {
        crate::fstatat(self.fd(), path, flags)
    }
}

impl Iterator for Dir {
    type Item = Result<Dirent>;

    #[inline]
    fn next(&mut self) -> Option<Result<Dirent>> {
        unsafe {
            let eno_ptr = util::errno_ptr();
            *eno_ptr = 0;

            let entry = libc::readdir(self.0.as_ptr());

            if entry.is_null() {
                return match *eno_ptr {
                    0 => None,
                    eno => Some(Err(Error::from_code(eno))),
                };
            }

            Some(Ok(Dirent::new(entry)))
        }
    }
}

impl Drop for Dir {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.0.as_ptr());
        }
    }
}

/// An entry yielded by iterating over a `Dir`.
#[derive(Clone, Debug)]
pub struct Dirent {
    entry: libc::dirent,
    #[cfg(not(bsd))]
    namelen: usize,
}

impl Dirent {
    #[allow(unused_variables)]
    #[inline]
    unsafe fn new(raw_entry: *const libc::dirent) -> Self {
        macro_rules! field_ptr {
            ($sptr:expr , $stype:path ; $fname:ident , $ftype:ty) => {{
                (($sptr) as *const u8).add(memoffset::offset_of!($stype, $fname)) as *const $ftype
            }};
        }

        cfg_if::cfg_if! {
            if #[cfg(target_os = "dragonfly")] {
                // DragonFlyBSD doesn't have `d_reclen`, so we have to use
                // `offsetof(dirent, d_namlen) + d_namelen + 1`.

                // Get d_namlen using the same technique as we use for getting d_reclen on other
                // platforms
                let namlen = *field_ptr!(raw_entry, libc::dirent; d_namlen, u16);

                let reclen = memoffset::offset_of!(libc::dirent, d_name) as u16 + namlen + 1;
            } else {
                // This should be whatever type `d_reclen` is on this platform
                type ReclenType = u16;

                // Get the value of `d_reclen` without dereferencing `raw_entry` or constructing a
                // reference
                // That wouldn't be safe because only part of `d_name` might be addressable
                let reclen = *field_ptr!(raw_entry, libc::dirent; d_reclen, u16);
            }
        }

        // There should be enough space for all the fields before `d_name`, plus 2 bytes for at
        // least one charater of the name and a terminating NUL
        debug_assert!(reclen as usize >= memoffset::offset_of!(libc::dirent, d_name) + 2);
        debug_assert!(reclen as usize <= core::mem::size_of::<libc::dirent>());

        // Now only copy out the first `reclen` bytes of the entry
        let mut entry = MaybeUninit::<libc::dirent>::uninit();
        core::ptr::copy_nonoverlapping(
            raw_entry as *const u8,
            entry.as_mut_ptr() as *mut u8,
            reclen as usize,
        );
        let entry = entry.assume_init();

        // Check that a) the type for `reclen` is right and b) the value is right
        #[cfg(not(target_os = "dragonfly"))]
        {
            let _: ReclenType = entry.d_reclen;
            debug_assert_eq!(entry.d_reclen, reclen);
        }

        #[cfg(bsd)]
        debug_assert_eq!(libc::strlen(entry.d_name.as_ptr()), entry.d_namlen as usize);

        Self {
            entry,
            #[cfg(not(bsd))]
            namelen: libc::strlen(entry.d_name.as_ptr()),
        }
    }

    #[inline]
    fn namelen(&self) -> usize {
        #[cfg(bsd)]
        let namelen = self.entry.d_namlen as usize;
        #[cfg(not(bsd))]
        let namelen = self.namelen;

        namelen
    }

    /// Get the name of this entry as an `OsStr`.
    #[inline]
    pub fn name(&self) -> &OsStr {
        OsStr::from_bytes(util::cvt_char_buf(&self.entry.d_name[..self.namelen()]))
    }

    /// Get the name of this entry as a `CStr`.
    ///
    /// [`Self::name()`] should be preferred unless a `CStr` is specifically needed (possibly for
    /// use in later FFI).
    #[inline]
    pub fn name_cstr(&self) -> &CStr {
        // SAFETY: namelen() either comes from d_namelen (set by the kernel) or strlen(d_name)
        unsafe {
            CStr::from_bytes_with_nul_unchecked(util::cvt_char_buf(
                &self.entry.d_name[..self.namelen() + 1],
            ))
        }
    }

    /// Get this entry's inode.
    ///
    /// Note: If this entry refers to a mountpoint (including bind mounts on Linux), this may be
    /// the inode of the *underlying directory* on which the filesystem is mounted. So this may not
    /// match, for example, the inode obtained by `stat()`ing this file.
    #[inline]
    pub fn ino(&self) -> u64 {
        #[cfg(any(freebsdlike, netbsdlike))]
        let ino = self.entry.d_fileno;
        #[cfg(not(any(freebsdlike, netbsdlike)))]
        let ino = self.entry.d_ino;

        ino as u64
    }

    /// Get the type of the file referred to by this entry, if available.
    ///
    /// This will not make any syscalls; it relies solely on information returned by the OS during
    /// iteration. As a result, it may return `None` at any time; in fact, on some platforms this
    /// function may always return `None`.
    #[inline]
    pub fn file_type(&self) -> Option<DirFileType> {
        DirFileType::new(self.entry.d_type)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DirFileType {
    File = libc::DT_REG,
    Directory = libc::DT_DIR,
    Symlink = libc::DT_LNK,
    Socket = libc::DT_SOCK,
    Fifo = libc::DT_FIFO,
    Block = libc::DT_BLK,
    Char = libc::DT_CHR,
}

impl DirFileType {
    #[inline]
    fn new(ftype: u8) -> Option<Self> {
        match ftype {
            libc::DT_REG => Some(Self::File),
            libc::DT_DIR => Some(Self::Directory),
            libc::DT_LNK => Some(Self::Symlink),
            libc::DT_SOCK => Some(Self::Socket),
            libc::DT_FIFO => Some(Self::Fifo),
            libc::DT_BLK => Some(Self::Block),
            libc::DT_CHR => Some(Self::Char),
            _ => None,
        }
    }
}

impl From<DirFileType> for crate::StatFileType {
    #[inline]
    fn from(ftype: DirFileType) -> Self {
        let mask = match ftype {
            DirFileType::File => libc::S_IFREG,
            DirFileType::Directory => libc::S_IFDIR,
            DirFileType::Symlink => libc::S_IFLNK,
            DirFileType::Socket => libc::S_IFSOCK,
            DirFileType::Fifo => libc::S_IFIFO,
            DirFileType::Block => libc::S_IFCHR,
            DirFileType::Char => libc::S_IFBLK,
        } as u32;

        Self { mask }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_root() {
        let mut dir = Dir::open(crate::c_paths::slash()).unwrap();

        let dir_stat = dir.stat().unwrap();

        #[allow(clippy::while_let_on_iterator)]
        while let Some(entry) = dir.next() {
            let entry = entry.unwrap();

            assert_eq!(entry.name().as_bytes(), entry.name_cstr().to_bytes());

            let stat = dir
                .fstatat(entry.name_cstr(), crate::AtFlag::AT_SYMLINK_NOFOLLOW)
                .unwrap();

            if let Some(ftype) = entry.file_type() {
                assert_eq!(crate::StatFileType::from(ftype), stat.file_type());
            }

            if entry.name() == "." {
                assert_eq!(entry.ino(), stat.ino());
                assert_eq!(entry.ino(), dir_stat.ino());

                assert_eq!(dir_stat.dev(), stat.dev());
            }
        }

        assert!(dir.next().is_none());
        dir.rewind();
        assert!(dir.next().is_some());

        let dir2 = unsafe {
            Dir::fdopen(
                crate::open(
                    crate::c_paths::slash(),
                    crate::OFlag::O_RDONLY | crate::OFlag::O_DIRECTORY | crate::OFlag::O_CLOEXEC,
                    0,
                )
                .unwrap()
                .into_fd(),
            )
        }
        .unwrap();
        let dir2_stat = dir2.stat().unwrap();

        assert_eq!(dir_stat.ino(), dir2_stat.ino());
        assert_eq!(dir_stat.dev(), dir2_stat.dev());
    }
}
