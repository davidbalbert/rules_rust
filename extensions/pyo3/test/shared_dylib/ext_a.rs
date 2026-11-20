//! An extension module that creates `Square`s.

use pyo3::prelude::*;

#[pymodule]
mod ext_a {
    use super::*;

    #[pyfunction]
    fn increment() -> u32 {
        counter::increment()
    }

    #[pyfunction]
    fn make_square(side: u32) -> shapes::Square {
        shapes::Square { side }
    }
}
