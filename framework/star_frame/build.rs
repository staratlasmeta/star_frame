use rustc_version::{version, version_meta, Channel};

fn main() {
    let version = version().unwrap();
    let version_meta = version_meta().unwrap();
    match version_meta.channel {
        Channel::Dev | Channel::Nightly => {}
        _ => {
            panic!("This crate can only be built with nightly/dev rust");
        }
    }
    if version.minor >= 75 {
        println!("cargo:rustc-cfg=rust_1_75");
    }
    if version.minor >= 76 {
        println!("cargo:rustc-cfg=rust_1_76");
    }
}
