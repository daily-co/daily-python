use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyList, PyLong, PyString};

#[repr(transparent)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DictValue(pub Value);

impl DictValue {
    fn value_to_object(val: &Value, py: Python<'_>) -> PyObject {
        match val {
            Value::Null => py.None(),
            Value::Bool(b) => b.to_object(py),
            Value::Number(n) => n
                .as_i64()
                .map(|i| i.to_object(py))
                .or_else(|| n.as_u64().map(|i| i.to_object(py)))
                .or_else(|| n.as_f64().map(|i| i.to_object(py)))
                .expect("Invalid number"),
            Value::String(s) => s.to_object(py),
            Value::Array(v) => {
                let inner: Vec<_> = v.iter().map(|x| Self::value_to_object(x, py)).collect();
                inner.to_object(py)
            }
            Value::Object(m) => {
                let inner: HashMap<_, _> = m
                    .iter()
                    .map(|(k, v)| (k, Self::value_to_object(v, py)))
                    .collect();
                inner.to_object(py)
            }
        }
    }
}

impl ToPyObject for DictValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        Self::value_to_object(&self.0, py)
    }
}

impl<'py> FromPyObject<'py> for DictValue {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> Result<Self, PyErr> {
        if let Ok(value) = ob.downcast::<PyBool>() {
            Ok(DictValue(value.is_true().into()))
        } else if let Ok(value) = ob.downcast::<PyLong>() {
            let number: i64 = value.extract().unwrap();
            Ok(DictValue(number.into()))
        } else if let Ok(value) = ob.downcast::<PyFloat>() {
            let number: f64 = value.extract().unwrap();
            Ok(DictValue(number.into()))
        } else if let Ok(value) = ob.downcast::<PyString>() {
            Ok(DictValue(value.to_string().into()))
        } else if let Ok(value) = ob.downcast::<PyList>() {
            let list: Vec<DictValue> = value.extract().unwrap();
            let vec = list.iter().map(|v| v.0.clone()).collect();
            Ok(DictValue(Value::Array(vec)))
        } else if let Ok(value) = ob.downcast::<PyDict>() {
            let dict: HashMap<String, DictValue> = value.extract().unwrap();
            let map = dict.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect();
            Ok(DictValue(Value::Object(map)))
        } else {
            Err(PyErr::new::<PyTypeError, _>("Invalid dictionary"))
        }
    }
}
