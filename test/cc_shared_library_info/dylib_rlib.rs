//! A crate that depends on a Rust ABI dylib.

pub fn rlib_value() -> u32 {
    dylib_with_deps::dylib_value()
}
