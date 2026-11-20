//! A Rust ABI dylib containing other crates.

pub fn dylib_value() -> u32 {
    shared_crate::value() + native_value()
}

extern "C" {
    fn c_value() -> u32;
}

fn native_value() -> u32 {
    unsafe { c_value() }
}
