//! A C ABI shared library using a shared crate.

#[no_mangle]
pub extern "C" fn cdylib_b_value() -> u32 {
    shared_crate::value()
}
