#!/usr/bin/env python3
import re
import os
import sys

# How to get the source file for each OS:
# linux_musl:
#   friendly: https://git.etalabs.net/cgit/musl/tree/src/errno/__strerror.h
#   raw: https://git.etalabs.net/cgit/musl/plain/src/errno/__strerror.h
# linux_glibc (BROKEN):
#   friendly: https://sourceware.org/git/?p=glibc.git;a=blob;f=sysdeps/gnu/errlist.h;h=6329e5f393edac2689e7304f04cfa81ce080242c;hb=HEAD
#   raw: https://sourceware.org/git/?p=glibc.git;a=blob_plain;f=sysdeps/gnu/errlist.h;h=6329e5f393edac2689e7304f04cfa81ce080242c;hb=HEAD
# android:
#   friendly: https://android.googlesource.com/platform/bionic/+/master/libc/bionic/strerror.cpp
#   base64: https://android.googlesource.com/platform/bionic/+/master/libc/bionic/strerror.cpp?format=TEXT
# freebsd:
#   friendly: https://github.com/freebsd/freebsd-src/blob/main/lib/libc/gen/errlst.c
#   raw: https://github.com/freebsd/freebsd-src/raw/main/lib/libc/gen/errlst.c
# netbsd:
#   friendly: https://github.com/NetBSD/src/blob/trunk/lib/libc/compat/gen/compat_errlist.c
#   raw: https://github.com/NetBSD/src/raw/trunk/lib/libc/compat/gen/compat_errlist.c
# openbsd:
#   friendly: https://github.com/openbsd/src/blob/master/lib/libc/gen/errlist.c
#   raw: https://github.com/openbsd/src/raw/master/lib/libc/gen/errlist.c
# dragonfly:
#   friendly: https://github.com/DragonFlyBSD/DragonFlyBSD/blob/master/lib/libc/gen/errlst.c
#   raw: https://github.com/DragonFlyBSD/DragonFlyBSD/raw/master/lib/libc/gen/errlst.c
# macos:
#   Go to https://opensource.apple.com/source/Libc; select the latest version, and navigate to gen/errlst.c

BSD_RE = re.compile(
    r"^([ \t]*\"(?P<msg>[^\"]*)\"[ \t]*,[ \t]*\/\*[ \t]*(?P<num>[0-9]+)[ \t]*-[ \t]*(?P<name>E[A-Z0-9]+)[ \t]*\*\/[ \t]*)"
    r"|([ \t]*\/\*[ \t]*(?P<num2>[0-9]+)[ \t]*-[ \t]*(?P<name2>E[A-Z0-9]+)[ \t]*\*\/[ \t]*\n[ \t]*\"(?P<msg2>[^\"]*)\"[ \t]*,[ \t]*)"
    r"$",
    re.M,
)

LINUX_MUSL_RE = re.compile(
    r"^E\((?P<name>(0|E[A-Z0-9]+)),\s*\"(?P<msg>[^\"]*)\"\s*\)\s*$"
)
LINUX_GLIBC_RE = re.compile(
    r"^_S\((?P<name>(0|E[A-Z0-9]+)),\s*N_\(\s*\"(?P<msg>[^\"]*)\"\s*\)\s*\)\s*$"
)

ANDROID_RE = re.compile(
    r"^\s*\[(?P<name>(0|E[A-Z0-9]+))\]\s*=\s*\"(?P<msg>[^\"]*)\"\s*,\s*$"
)


def do_parse_linuxlike(file, libc):
    assert libc != "glibc", "glibc support is currently broken"

    regex = {"glibc": LINUX_GLIBC_RE, "musl": LINUX_MUSL_RE, "android": ANDROID_RE}[
        libc
    ]

    yield "#[inline]"
    yield "pub(crate) fn strerror_imp(eno: i32) -> &'static str {"
    yield "    match eno {"

    errnos = {}
    for line in file:
        line = line.rstrip("\n")
        match = regex.match(line)
        if match is not None:
            name = match.group("name")
            msg = match.group("msg")

            if name == "0":
                yield '        0 => "{}",'.format(msg)
                continue

            yield '        libc::{} => "{}",'.format(name, msg)

    yield '        _ => "Unknown error",'
    yield "    }"
    yield "}"


def do_parse_bsd(file):
    errnos = {}

    text = file.read()
    for match in BSD_RE.finditer(text):
        if match.group("num"):
            errnos[int(match.group("num"))] = match.group("msg")
        else:
            errnos[int(match.group("num2"))] = match.group("msg2")

    elast = max(errnos)

    yield "pub(crate) const ERRNO_TABLE: [&'static str; {}] = [".format(elast + 1)

    for eno in range(0, elast + 1):
        if eno in errnos:
            yield '    "{}",'.format(errnos[eno])
        else:
            yield '    "Unknown error",'

    yield "];"


def main(args) -> None:
    if len(args) != 2:
        print("Usage: {} <OS name> <path to source file>".format(sys.argv[0]))
        sys.exit(1)

    repo_path = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))

    os_name = args[0]
    source_path = args[1]

    assert os_name in (
        "linux_glibc",
        "linux_musl",
        "android",
        "macos",
        "freebsd",
        "netbsd",
        "openbsd",
        "dragonfly",
    )

    with open(source_path) as file:
        with open(
            os.path.join(repo_path, "src/strerror", os_name + ".rs"), "w"
        ) as ofile:
            if os_name.startswith("linux_"):
                it = do_parse_linuxlike(file, os_name[6:])
            elif os_name == "android":
                it = do_parse_linuxlike(file, "android")
            else:
                it = do_parse_bsd(file)

            for line in it:
                ofile.write(line + "\n")


if "__main__" == __name__:
    main(sys.argv[1:])
