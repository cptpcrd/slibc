use core::fmt;

#[cfg(feature = "std")]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

use crate::internal_prelude::*;

/// The size of an `inotify_event` struct, not including the optional name at the end.
const EVENT_SIZE: usize = core::mem::size_of::<libc::inotify_event>();

/// The minimum buffer size to read at least one event from an inotify file descriptor.
///
/// It may be desirable to use a buffer of size e.g. `INOTIFY_MIN_BUFSIZE * 8` or larger to read as
/// many events as possible.
pub const INOTIFY_MIN_BUFSIZE: usize = EVENT_SIZE + crate::NAME_MAX + 1;

bitflags::bitflags! {
    pub struct InotifyFlags: libc::c_int {
        const NONBLOCK = libc::IN_NONBLOCK;
        const CLOEXEC = libc::IN_CLOEXEC;
    }
}

bitflags::bitflags! {
    pub struct InotifyMask: u32 {
        const ACCESS = libc::IN_ACCESS;
        const ATTRIB = libc::IN_ATTRIB;
        const CLOSE_WRITE = libc::IN_CLOSE_WRITE;
        const CREATE = libc::IN_CREATE;
        const DELETE = libc::IN_DELETE;
        const DELETE_SELF = libc::IN_DELETE_SELF;
        const MODIFY = libc::IN_MODIFY;
        const MOVE_SELF = libc::IN_MOVE_SELF;
        const MOVED_FROM = libc::IN_MOVED_FROM;
        const MOVED_TO = libc::IN_MOVED_TO;
        const OPEN = libc::IN_OPEN;

        const ALL_EVENTS = libc::IN_ALL_EVENTS;
        const MOVE = libc::IN_MOVE;
        const CLOSE = libc::IN_CLOSE;

        const DONT_FOLLOW = libc::IN_DONT_FOLLOW;
        const ONLYDIR = libc::IN_ONLYDIR;
        const EXCL_UNLINK = 0x04000000;
        const ONESHOT = libc::IN_ONESHOT;
        const MASK_ADD = 0x20000000;
        const MASK_CREATE = 0x10000000;

        const IGNORED = libc::IN_IGNORED;
        const ISDIR = libc::IN_ISDIR;
        const Q_OVERFLOW = libc::IN_Q_OVERFLOW;
        const UNMOUNT = libc::IN_UNMOUNT;
    }
}

#[derive(Copy, Clone)]
pub struct InotifyEvent<'a> {
    event: &'a libc::inotify_event,
    // Invariant: This should either be empty or have exactly one NUL at the end
    name: &'a [u8],
}

impl InotifyEvent<'_> {
    /// The "watch descriptor" of the "watch" that triggered this event.
    ///
    /// This is the value that was returned from [`inotify_add_watch()`].
    ///
    /// If this is -1 and [`Self::mask()`] includes [`InotifyMask::Q_OVERFLOW`], the event queue
    /// overflowed and events may have been dropped.
    #[inline]
    pub fn wd(&self) -> i32 {
        self.event.wd
    }

    /// A mask describing the event.
    ///
    /// This includes both the events that were triggered and other information on the watch.
    #[inline]
    pub fn mask(&self) -> InotifyMask {
        InotifyMask::from_bits_truncate(self.event.mask)
    }

    /// A unique integer that connects related events.
    ///
    /// Currently, this is only used for rename events, allowing an application to connect
    /// `MOVED_FROM` and `MOVED_TO` events. For all other event types it is set to 0.
    #[inline]
    pub fn cookie(&self) -> u32 {
        self.event.cookie
    }

    /// Get the filename associated with the event.
    ///
    /// This is only returned for events on files inside watched directories; it identifies the
    /// file within the directory that triggered the event.
    #[inline]
    pub fn name(&self) -> Option<&OsStr> {
        Some(OsStr::from_bytes(self.name.split_last()?.1))
    }

    /// Get the filename associated with the event as a `CStr`.
    ///
    /// [`Self::name()`] should be preferred unless a `CStr` is specifically needed e.g. for FFI.
    #[inline]
    pub fn name_cstr(&self) -> Option<&CStr> {
        if self.name.is_empty() {
            None
        } else {
            Some(unsafe { CStr::from_bytes_with_nul_unchecked(self.name) })
        }
    }
}

impl fmt::Debug for InotifyEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InotifyEvent")
            .field("wd", &self.wd())
            .field("mask", &self.mask())
            .field("cookie", &self.cookie())
            .field("name", &self.name())
            .finish()
    }
}

#[inline]
pub fn inotify_init1(flags: InotifyFlags) -> Result<FileDesc> {
    Error::unpack(unsafe { libc::inotify_init1(flags.bits()) })
        .map(|fd| unsafe { FileDesc::new(fd) })
}

#[inline]
pub fn inotify_add_watch<P: AsPath>(fd: RawFd, path: P, mask: InotifyMask) -> Result<i32> {
    path.with_cstr(|path| {
        Error::unpack(unsafe { libc::inotify_add_watch(fd, path.as_ptr(), mask.bits()) })
    })
}

#[inline]
pub fn inotify_rm_watch(fd: RawFd, wd: i32) -> Result<()> {
    Error::unpack_nz(unsafe { libc::inotify_rm_watch(fd, wd as _) })
}

/// An iterator over events that were `read()` from an inotify file descriptor.
///
/// Currently this can only be obtained by calling [`Inotify::read_events()`].
#[derive(Clone)]
pub struct InotifyEventIter<'a> {
    buf: &'a [u8],
}

impl<'a> InotifyEventIter<'a> {
    /// Read one or more events from the inotify instance specified by `fd` into the given buffer,
    /// and return an iterator over the events.
    ///
    /// Usually, `buf` should be at least [`INOTIFY_MIN_BUFSIZE`] bytes long to ensure that at
    /// least one event can be read.
    ///
    /// For a safe version of this method, use [`Inotify`] and see [`Inotify::read_events()`].
    ///
    /// # Safety
    ///
    /// `fd` MUST refer to an inotify file descriptor. If it does not, the returned iterator will
    /// try to interpret whatever data was read from `fd` as an inotify event, with strange
    /// results.
    ///
    /// Currently, the data is interpreted in such a way that this will not actually trigger UB
    /// (instead, it might panic), but that may be changed in the future.
    #[inline]
    pub unsafe fn read_from(fd: RawFd, buf: &'a mut [u8]) -> Result<Self> {
        let n = crate::read(fd, buf)?;
        Ok(Self { buf: &buf[..n] })
    }
}

impl<'a> Iterator for InotifyEventIter<'a> {
    type Item = InotifyEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // This is everything *after* the next event
        let rest = self.buf.get(EVENT_SIZE..)?;

        // SAFETY: self.buf is >= sizeof(inotify_event) bytes long, and initialized by the kernel
        let event = unsafe { &*(self.buf.as_ptr() as *const libc::inotify_event) };

        // Extract the name, and cut self.buf down to what's remaining
        let (mut name, rest) = rest.split_at(event.len as usize);
        self.buf = rest;

        if !name.is_empty() {
            // `name` may include extra NUL bytes; trim it down
            name = &name[..=crate::memchr(name, 0).unwrap()];
        }

        Some(InotifyEvent { event, name })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Simplistic, but fast and technically accurate
        (
            (self.buf.len() >= EVENT_SIZE) as usize,
            Some(self.buf.len() / EVENT_SIZE),
        )
    }
}

impl<'a> core::iter::FusedIterator for InotifyEventIter<'a> {}

impl fmt::Debug for InotifyEventIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("InotifyEventIter")
            .field(&util::DebugListField(self.clone()))
            .finish()
    }
}

#[derive(Debug)]
pub struct Inotify(FileDesc);

impl Inotify {
    #[inline]
    pub fn new(flags: InotifyFlags) -> Result<Self> {
        inotify_init1(flags).map(Self)
    }

    #[inline]
    pub fn add_watch<P: AsPath>(&self, path: P, mask: InotifyMask) -> Result<i32> {
        inotify_add_watch(self.fd(), path, mask)
    }

    #[inline]
    pub fn rm_watch(&self, wd: i32) -> Result<()> {
        inotify_rm_watch(self.fd(), wd)
    }

    /// Read one or more events from this inotify instance into the given buffer, and return an
    /// iterator over the events.
    ///
    /// `buf` should be at least [`INOTIFY_MIN_BUFSIZE`] bytes long to ensure that at least one
    /// event can be read.
    #[inline]
    pub fn read_events<'a>(&self, buf: &'a mut [u8]) -> Result<InotifyEventIter<'a>> {
        unsafe { InotifyEventIter::read_from(self.fd(), buf) }
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.0.fd()
    }

    #[inline]
    pub fn into_fd(self) -> RawFd {
        self.0.into_fd()
    }

    /// Create a new `Inotify` wrapper around the given inotify file descriptor.
    ///
    /// # Safety
    ///
    /// The given file descriptor must refer to a valid inotify instance, and it must not be in use
    /// by other code.
    #[inline]
    pub unsafe fn from_fd(fd: RawFd) -> Self {
        Self(FileDesc::new(fd))
    }
}

impl AsRef<BorrowedFd> for Inotify {
    #[inline]
    fn as_ref(&self) -> &BorrowedFd {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl AsRawFd for Inotify {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(feature = "std")]
impl IntoRawFd for Inotify {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(feature = "std")]
impl FromRawFd for Inotify {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_fd(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags() {
        let i = Inotify::new(InotifyFlags::empty()).unwrap();
        assert!(!i.as_ref().get_cloexec().unwrap());
        assert!(!i.as_ref().get_nonblocking().unwrap());

        let i = Inotify::new(InotifyFlags::CLOEXEC).unwrap();
        assert!(i.as_ref().get_cloexec().unwrap());
        assert!(!i.as_ref().get_nonblocking().unwrap());

        let i = Inotify::new(InotifyFlags::NONBLOCK).unwrap();
        assert!(!i.as_ref().get_cloexec().unwrap());
        assert!(i.as_ref().get_nonblocking().unwrap());

        let i = Inotify::new(InotifyFlags::CLOEXEC | InotifyFlags::NONBLOCK).unwrap();
        assert!(i.as_ref().get_cloexec().unwrap());
        assert!(i.as_ref().get_nonblocking().unwrap());
    }

    #[test]
    fn test_eventiter_empty() {
        let mut it = InotifyEventIter { buf: &[] };
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
        #[cfg(feature = "alloc")]
        assert_eq!(format!("{:?}", it), "InotifyEventIter([])");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_tempdir() {
        let tmpdir = tempfile::tempdir().unwrap();

        let i = Inotify::new(InotifyFlags::CLOEXEC).unwrap();
        let wd = i
            .add_watch(
                tmpdir.as_ref(),
                InotifyMask::CREATE | InotifyMask::DELETE_SELF | InotifyMask::ONLYDIR,
            )
            .unwrap();

        std::fs::File::create(tmpdir.as_ref().join("file")).unwrap();
        drop(tmpdir);

        let mut buf = [0; INOTIFY_MIN_BUFSIZE * 3];
        let mut events = i.read_events(&mut buf).unwrap();

        assert_eq!(events.size_hint().0, 1, "{:?}", events);

        let event = events.next().unwrap();
        assert_eq!(event.wd(), wd);
        assert_eq!(event.mask(), InotifyMask::CREATE);
        assert_eq!(event.name(), Some("file".as_ref()));
        assert_eq!(
            event.name_cstr(),
            Some(CStr::from_bytes_with_nul(b"file\0").unwrap())
        );

        assert_eq!(events.size_hint(), (1, Some(2)), "{:?}", events);

        let event = events.next().unwrap();
        assert_eq!(event.wd(), wd);
        assert_eq!(event.mask(), InotifyMask::DELETE_SELF);
        assert_eq!(event.name(), None);
        assert_eq!(event.name_cstr(), None);

        assert_eq!(events.size_hint(), (1, Some(1)), "{:?}", events);

        let event = events.next().unwrap();
        assert_eq!(event.wd(), wd);
        assert_eq!(event.mask(), InotifyMask::IGNORED);
        assert_eq!(event.name(), None);
        assert_eq!(event.name_cstr(), None);

        assert_eq!(events.size_hint(), (0, Some(0)), "{:?}", events);
        assert!(events.next().is_none());
    }
}
