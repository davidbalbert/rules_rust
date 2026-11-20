//! A `#[pyclass]` shared by several extension modules.

use pyo3::prelude::*;

#[pyclass]
pub struct Square {
    pub side: u32,
}
