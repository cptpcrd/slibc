# slibc

[![crates.io](https://img.shields.io/crates/v/slibc.svg)](https://crates.io/crates/slibc)
[![Docs](https://docs.rs/slibc/badge.svg)](https://docs.rs/slibc)
[![GitHub Actions](https://github.com/cptpcrd/slibc/workflows/CI/badge.svg?branch=master&event=push)](https://github.com/cptpcrd/slibc/actions?query=workflow%3ACI+branch%3Amaster+event%3Apush)
[![Cirrus CI](https://api.cirrus-ci.com/github/cptpcrd/slibc.svg?branch=master)](https://cirrus-ci.com/github/cptpcrd/slibc)
[![codecov](https://codecov.io/gh/cptpcrd/slibc/branch/master/graph/badge.svg)](https://codecov.io/gh/cptpcrd/slibc)

Simple interfaces to low-level functions in the system libc.

## Advantages over `nix`

- Uses a custom error type that can be converted to an `io::Error` (so `?` works in functions that return `io::Error`)
- Supports `#![no_std]` environments (disable the `std` feature; optionally enable the `alloc` feature)

## Supported platforms

### "Tier 1"

Tests are run on the following platforms:

- Linux (glibc and musl)
- macOS
- FreeBSD

`slibc` should work normally on these platforms.

### "Tier 2"

Builds (but not tests) are run for the following platforms:

- NetBSD
- Android

`slibc` should build on these platforms, but there may be bugs that cause even the test cases to fail.

### "Tier 3"

Builds without the `std` or `alloc` features are run for the following platforms:

- OpenBSD
- DragonFlyBSD

`slibc` may not even properly build on these platforms if the `std` or `alloc` features are enabled.
