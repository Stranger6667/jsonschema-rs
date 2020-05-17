use jsonschema;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyFloat, PyList, PyTuple};
use pyo3::wrap_pyfunction;
use pyo3::Python;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq};
use serde::Serializer;

struct ValueWrapper<'a> {
    obj: &'a PyAny,
}

/// Convert a Python value to serde_json::Value
impl<'a> Serialize for ValueWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        macro_rules! cast {
            ($f:expr) => {
                if let Ok(val) = PyTryFrom::try_from(self.obj) {
                    return $f(val);
                }
            };
        }
        macro_rules! extract {
            ($t:ty) => {
                if let Ok(val) = <$t as FromPyObject>::extract(self.obj) {
                    return val.serialize(serializer);
                }
            };
        }

        cast!(|x: &PyList| {
            let mut seq = serializer.serialize_seq(Some(x.len()))?;
            for element in x {
                seq.serialize_element(&ValueWrapper { obj: element })?
            }
            seq.end()
        });
        cast!(|x: &PyTuple| {
            let mut seq = serializer.serialize_seq(Some(x.len()))?;
            for element in x {
                seq.serialize_element(&ValueWrapper { obj: element })?
            }
            seq.end()
        });
        cast!(|x: &PyDict| {
            let mut map = serializer.serialize_map(Some(x.len()))?;
            for (key, value) in x {
                if key.is_none() {
                    map.serialize_key("null")?;
                } else if let Ok(key) = key.extract::<bool>() {
                    map.serialize_key(if key { "true" } else { "false" })?;
                } else if let Ok(key) = key.str() {
                    let key = key.to_string().unwrap();
                    map.serialize_key(&key)?;
                } else {
                    return Err(ser::Error::custom(format_args!(
                        "Dictionary key is not a string: {:?}",
                        key
                    )));
                }
                map.serialize_value(&ValueWrapper { obj: value })?;
            }
            map.end()
        });
        extract!(String);
        extract!(bool);
        extract!(i64);
        cast!(|x: &PyFloat| {
            let v = x.value();
            if !v.is_finite() {
                return Err(ser::Error::custom(format!("Can't represent {} as JSON", v)));
            }
            v.serialize(serializer)
        });
        if self.obj.is_none() {
            return serializer.serialize_unit();
        }
        match self.obj.repr() {
            Ok(repr) => Err(ser::Error::custom(format_args!(
                "Can't convert to JSON: {}",
                repr,
            ))),
            Err(_) => Err(ser::Error::custom(format_args!(
                "Type is not JSON serializable: {}",
                self.obj.get_type().name().into_owned(),
            ))),
        }
    }
}

#[pyfunction]
fn is_valid(schema: &'static PyAny, instance: &'static PyAny) -> PyResult<bool> {
    let schema = serde_json::to_value(ValueWrapper { obj: schema }).unwrap();
    let instance = serde_json::to_value(ValueWrapper { obj: instance }).unwrap();
    Ok(jsonschema::is_valid(&schema, &instance))
}

#[pyfunction]
fn is_valid_noop(_schema: &'static PyAny, _instance: &'static PyAny) -> PyResult<bool> {
    // To measure call overhead
    Ok(true)
}

#[pymodule]
fn jsonschema_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(is_valid))?;
    m.add_wrapped(wrap_pyfunction!(is_valid_noop))?;
    Ok(())
}
