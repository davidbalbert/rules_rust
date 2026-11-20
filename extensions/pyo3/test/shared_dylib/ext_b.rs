//! An extension module that accepts `Square`s created by another one.

use pyo3::prelude::*;

#[pymodule]
mod ext_b {
    use super::*;

    #[pyfunction]
    fn increment() -> u32 {
        counter::increment()
    }

    #[pyfunction]
    fn area(square: PyRef<'_, shapes::Square>) -> u32 {
        square.side * square.side
    }
}
