use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyLong, PyString};

#[repr(transparent)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DictValue(Value);

impl DictValue {
    fn value_to_object(val: &Value, py: Python<'_>) -> PyObject {
        match val {
            Value::Null => py.None(),
            Value::Bool(b) => b.to_object(py),
            Value::Number(n) => {
                let oi64 = n.as_i64().map(|i| i.to_object(py));
                let ou64 = n.as_u64().map(|i| i.to_object(py));
                let of64 = n.as_f64().map(|i| i.to_object(py));
                oi64.or(ou64).or(of64).expect("invalid number")
            }
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

impl<'a> FromPyObject<'a> for DictValue {
    fn extract(ob: &'a PyAny) -> Result<Self, PyErr> {
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
        } else if let Ok(value) = ob.downcast::<PyDict>() {
            let dict: HashMap<String, DictValue> = value.extract().unwrap();
            let map = dict.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect();
            Ok(DictValue(Value::Object(map)))
        } else {
            Err(PyErr::new::<PyTypeError, _>("Invalid dictionary"))
        }
    }
}
