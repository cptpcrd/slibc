use crate::internal_prelude::*;

use core::ptr::NonNull;

/// A directory stream open for iterating over the entries in a directory.
///
/// Note that this struct will yield entries for `.` and `..` if they are returned by the OS.
#[derive(Debug)]
pub struct Dir(NonNull<libc::DIR>);

impl Dir {
    /// Create a new directory stream for the directory with the given `path`.
    #[inline]
    pub fn open<P: AsPath>(path: P) -> Result<Self> {
        path.with_cstr(|path| {
            Ok(Self(Error::unpack_ptr(unsafe {
                libc::opendir(path.as_ptr())
            })?))
        })
    }

    /// Create a new directory stream for the directory referred to by the open file descriptor
    /// `fd`.
    ///
    /// # Safety
    ///
    /// `fd` will be consumed by the new directory stream.
    #[inline]
    pub unsafe fn fdopen(fd: RawFd) -> Result<Self> {
        Ok(Self(Error::unpack_ptr(libc::fdopendir(fd))?))
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

            let name = entry_name(entry);
            Some(Ok(Dirent::new(entry, name.len())))
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
    #[inline]
    unsafe fn new(entry: *const libc::dirent, namelen: usize) -> Self {
        Self {
            entry: *entry,
            #[cfg(not(bsd))]
            namelen,
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
    /// [`name()`] should be preferred unless a `CStr` is specifically needed (possibly for use in
    /// later FFI).
    #[inline]
    pub fn name_cstr(&self) -> &CStr {
        // SAFETY: entry_name() makes sure that this length is accurate
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
    /// iteration.
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
        };

        Self { mask }
    }
}

#[inline]
unsafe fn entry_name<'a>(entry: *const libc::dirent) -> &'a [u8] {
    let d_name = (*entry).d_name.as_ptr();

    #[cfg(bsd)]
    let namelen = {
        // Check that the first NUL byte is where the kernel says it is
        debug_assert_eq!(libc::strlen(d_name), entry.d_namlen as usize);
        entry.d_namlen as usize
    };

    #[cfg(not(bsd))]
    let namelen = libc::strlen(d_name);

    core::slice::from_raw_parts(d_name as *mut u8, namelen)
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
            }
        }
    }
}
