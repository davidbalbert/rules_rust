//! A crate depending on a crate tagged `LINKABLE_MORE_THAN_ONCE`.

pub fn rlib_value() -> u32 {
    shared_crate::value()
}
