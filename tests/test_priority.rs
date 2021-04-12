use slibc::{getpriority, nice, setpriority, Errno, PrioWho};

#[test]
fn test_get_prio() {
    assert_eq!(getpriority(PrioWho::Process(0)).unwrap(), nice(0).unwrap());
}

#[test]
fn test_set_prio() {
    let prio = getpriority(PrioWho::Process(0)).unwrap();

    setpriority(PrioWho::Process(0), prio).unwrap();
    assert_eq!(getpriority(PrioWho::Process(0)).unwrap(), prio);

    if prio < 20 {
        setpriority(PrioWho::Process(0), prio + 1).unwrap();
        assert_eq!(getpriority(PrioWho::Process(0)).unwrap(), prio + 1);
    }
}

#[test]
fn test_getset_prio_bad() {
    let bad_pid = libc::pid_t::MAX;

    assert_eq!(
        getpriority(PrioWho::Process(bad_pid)).unwrap_err(),
        Errno::ESRCH,
    );
    assert_eq!(
        setpriority(PrioWho::Process(bad_pid), 0).unwrap_err(),
        Errno::ESRCH,
    );

    if slibc::geteuid() != 0 {
        assert_eq!(
            setpriority(PrioWho::Process(1), 0).unwrap_err(),
            Errno::EPERM,
        );
    }
}

#[test]
fn test_nice() {
    if slibc::geteuid() != 0 {
        assert!(matches!(
            nice(-1).unwrap_err().code(),
            libc::EPERM | libc::EACCES
        ));
    }
}
