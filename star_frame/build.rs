use rustc_version::{version, version_meta, Channel};

fn main() {
    let _version = version().unwrap();
    let version_meta = version_meta().unwrap();
    match version_meta.channel {
        Channel::Dev | Channel::Nightly => {}
        _ => {
            panic!("This crate can only be built with nightly/dev rust");
        }
    }
}
