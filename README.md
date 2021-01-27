# slibc

[![crates.io](https://img.shields.io/crates/v/slibc.svg)](https://crates.io/crates/slibc)
[![Docs](https://docs.rs/slibc/badge.svg)](https://docs.rs/slibc)
[![GitHub Actions](https://github.com/cptpcrd/slibc/workflows/CI/badge.svg?branch=master&event=push)](https://github.com/cptpcrd/slibc/actions?query=workflow%3ACI+branch%3Amaster+event%3Apush)
[![codecov](https://codecov.io/gh/cptpcrd/slibc/branch/master/graph/badge.svg)](https://codecov.io/gh/cptpcrd/slibc)

Simple interfaces to low-level functions in the system libc.

## Advantages over `nix`

- Uses a custom error type that can be converted to an `io::Error` (so `?` works in functions that return `io::Error`)
- Supports `#![no_std]` environments (disable the `std` feature; optionally enable the `alloc` feature)

## Supported systems

- Linux (glibc and musl)
- macOS
- FreeBSD
