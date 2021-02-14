fn main() {
    match std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "linux" | "android" => println!("cargo:rustc-cfg=linux_like"),
        "freebsd" | "dragonfly" => println!("cargo:rustc-cfg=bsd\ncargo:rustc-cfg=freebsdlike"),
        "netbsd" | "openbsd" => println!("cargo:rustc-cfg=bsd\ncargo:rustc-cfg=netbsdlike"),
        "macos" | "ios" => println!("cargo:rustc-cfg=bsd\ncargo:rustc-cfg=apple"),
        _ => panic!("Unsupported OS"),
    }
}
