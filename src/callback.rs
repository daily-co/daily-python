use pyo3::prelude::*;

#[pyclass(name = "CallbackContext", module = "daily")]
pub struct PyCallbackContext {
    pub callback: PyObject,
}
