use slibc::{getpid, kill, sigset, SigSet, Signal};

// These tests are not thread-safe, so we run them all from one #[test]

fn restore_sigmask<F: FnOnce()>(f: F) {
    let mask = SigSet::thread_get_mask().unwrap();
    f();
    mask.thread_set_mask().unwrap();
}

#[test]
fn do_tests() {
    restore_sigmask(test_sigmask);
    restore_sigmask(test_kill);

    #[cfg(linuxlike)]
    restore_sigmask(test_tgkill);
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
    use slibc::{_exit, fork, sigpending, waitpid, WaitFlags, WaitStatus};

    fn run_child<F: FnOnce() -> slibc::Result<()>>(f: F) -> WaitStatus {
        match unsafe { fork().unwrap() } {
            Some(pid) => waitpid(pid, WaitFlags::empty()).unwrap().unwrap().1,

            None => unsafe {
                if let Err(_) = f() {
                    _exit(1);
                }

                _exit(0);
            },
        }
    }

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
