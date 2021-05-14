use slibc::{
    getpid, kill, sigset, SigSet, Signal, _exit, fork, sigpending, waitpid, WaitFlags, WaitStatus,
};

// These tests are not thread-safe, so we run them all from one #[test]

fn restore_sigmask<F: FnOnce()>(f: F) {
    let mask = SigSet::thread_get_mask().unwrap();
    f();
    mask.thread_set_mask().unwrap();
}

fn run_child<F: FnOnce() -> slibc::Result<()>>(f: F) -> WaitStatus {
    match unsafe { fork().unwrap() } {
        Some(pid) => waitpid(pid, WaitFlags::empty()).unwrap().unwrap().1,

        None => unsafe {
            if f().is_err() {
                _exit(1);
            }

            _exit(0);
        },
    }
}

#[test]
fn do_tests() {
    restore_sigmask(test_sigmask);
    restore_sigmask(test_kill);

    #[cfg(linuxlike)]
    {
        restore_sigmask(test_tgkill);
        restore_sigmask(test_signalfd);
        restore_sigmask(test_pidfd);
    }
}

fn test_sigmask() {
    let orig_mask = SigSet::thread_get_mask().unwrap();

    // Check that it doesn't change
    orig_mask.thread_set_mask().unwrap();
    assert_eq!(SigSet::thread_get_mask().unwrap(), orig_mask);

    let usrsigs = sigset!(Signal::SIGUSR1, Signal::SIGUSR2);
    let datasigs = sigset!(Signal::SIGIO, Signal::SIGURG);

    usrsigs.thread_block().unwrap();
    assert_eq!(SigSet::thread_get_mask().unwrap(), usrsigs);
    datasigs.thread_block().unwrap();
    assert_eq!(SigSet::thread_get_mask().unwrap(), usrsigs.union(&datasigs));
    usrsigs.thread_unblock().unwrap();
    assert_eq!(SigSet::thread_get_mask().unwrap(), datasigs);
}

fn test_kill() {
    let status = run_child(|| {
        let set = sigset!(Signal::SIGUSR1);
        set.thread_block()?;
        kill(getpid(), Signal::SIGUSR1)?;

        let pending = sigpending()?;
        unsafe {
            _exit(pending.iter().next().map_or(0, |s| s.as_i32()));
        }
    });

    assert_eq!(status, WaitStatus::Exited(libc::SIGUSR1));

    let status = run_child(|| {
        let set = sigset!(Signal::SIGUSR1);
        set.thread_block()?;
        kill(getpid(), Signal::SIGUSR1)?;

        let sig = set.wait()?;
        unsafe {
            _exit(sig.as_i32());
        }
    });

    assert_eq!(status, WaitStatus::Exited(libc::SIGUSR1));
}

#[cfg(linuxlike)]
fn test_tgkill() {
    use slibc::{gettid, tgkill};

    let set = sigset!(Signal::SIGUSR1);
    set.thread_block().unwrap();
    tgkill(getpid(), gettid(), Signal::SIGUSR1).unwrap();
    assert_eq!(set.wait().unwrap(), Signal::SIGUSR1);
}

#[cfg(linuxlike)]
fn test_signalfd() {
    use slibc::{gettid, tgkill, SigFdFlags, SigFdSigInfo, SignalFd};

    fn check_siginfo(sfd: &SignalFd, sig: Signal) {
        let mut sinfo = SigFdSigInfo::zeroed();
        assert_eq!(
            sfd.read_siginfos(std::slice::from_mut(&mut sinfo)).unwrap(),
            1
        );
        assert_eq!(sinfo.signal(), Some(sig));
    }

    let set = sigset!(Signal::SIGUSR1);
    set.thread_block().unwrap();
    let sfd = SignalFd::new(&set, SigFdFlags::CLOEXEC).unwrap();

    tgkill(getpid(), gettid(), Signal::SIGUSR1).unwrap();
    check_siginfo(&sfd, Signal::SIGUSR1);

    let set = sigset!(Signal::SIGUSR2);
    set.thread_block().unwrap();
    sfd.set_mask(&set).unwrap();

    tgkill(getpid(), gettid(), Signal::SIGUSR2).unwrap();
    check_siginfo(&sfd, Signal::SIGUSR2);
}

#[cfg(linuxlike)]
fn test_pidfd() {
    use slibc::{pidfd_send_signal_simple, Errno, PidFd, PidFdOpenFlags, PidFdSignalFlags};

    if pidfd_send_signal_simple(-1, None, PidFdSignalFlags::empty()).unwrap_err() != Errno::EBADF {
        return;
    }

    let status = run_child(|| {
        let pfd = PidFd::open(getpid(), PidFdOpenFlags::empty()).unwrap();

        let set = sigset!(Signal::SIGUSR1);
        set.thread_block().unwrap();
        pfd.send_signal_simple(Signal::SIGUSR1, PidFdSignalFlags::empty())
            .unwrap();

        let sig = set.wait()?;
        unsafe {
            _exit(sig.as_i32());
        }
    });

    assert_eq!(status, WaitStatus::Exited(libc::SIGUSR1));
}
