use pyo3::prelude::*;
use vortex::{Encoding, EncodingId};

use crate::arrays::{EncodingSubclass, PyArray};

/// Base class for defining custom Vortex encodings in Python.
#[pyclass(name = "PyEncoding", module = "vortex", extends=PyArray, frozen)]
pub(crate) struct PyEncoding;

impl EncodingSubclass for PyEncoding {
    type Encoding = PyEncoding;
}

#[pymethods]
impl PyEncoding {
    #[new]
    fn new(array: Bound<PyArray>) -> PyResult<Bound<Self>> {
        PyArray::init_encoding(array, PyEncoding)
    }
}

impl Encoding for PyEncoding {
    const ID: EncodingId = ();
    type Array = ();
    type Metadata = ();
}
