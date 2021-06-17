use pyo3::{
    exceptions,
    ffi::{
        PyDictObject, PyFloat_AS_DOUBLE, PyList_GET_ITEM, PyList_GET_SIZE, PyLong_AsLongLong,
        Py_TYPE,
    },
    prelude::*,
    types::PyAny,
    AsPyPointer,
};
use serde::{
    ser::{self, Serialize, SerializeMap, SerializeSeq},
    Serializer,
};

use crate::{string, types};
use std::ffi::CStr;

pub const RECURSION_LIMIT: u8 = 255;

#[derive(Clone)]
pub enum ObjectType {
    Str,
    Int,
    Bool,
    None,
    Float,
    List,
    Dict,
    Unknown(String),
}

pub(crate) struct SerializePyObject {
    object: *mut pyo3::ffi::PyObject,
    object_type: ObjectType,
    recursion_depth: u8,
}

impl SerializePyObject {
    #[inline]
    pub fn new(object: *mut pyo3::ffi::PyObject, recursion_depth: u8) -> Self {
        SerializePyObject {
            object,
            object_type: get_object_type_from_object(object),
            recursion_depth,
        }
    }

    #[inline]
    pub const fn with_obtype(
        object: *mut pyo3::ffi::PyObject,
        object_type: ObjectType,
        recursion_depth: u8,
    ) -> Self {
        SerializePyObject {
            object,
            object_type,
            recursion_depth,
        }
    }
}

fn get_object_type_from_object(object: *mut pyo3::ffi::PyObject) -> ObjectType {
    unsafe {
        let object_type = Py_TYPE(object);
        get_object_type(object_type)
    }
}

#[inline]
pub fn get_object_type(object_type: *mut pyo3::ffi::PyTypeObject) -> ObjectType {
    if object_type == unsafe { types::STR_TYPE } {
        ObjectType::Str
    } else if object_type == unsafe { types::FLOAT_TYPE } {
        ObjectType::Float
    } else if object_type == unsafe { types::BOOL_TYPE } {
        ObjectType::Bool
    } else if object_type == unsafe { types::INT_TYPE } {
        ObjectType::Int
    } else if object_type == unsafe { types::NONE_TYPE } {
        ObjectType::None
    } else if object_type == unsafe { types::LIST_TYPE } {
        ObjectType::List
    } else if object_type == unsafe { types::DICT_TYPE } {
        ObjectType::Dict
    } else {
        let type_name = unsafe { CStr::from_ptr((*object_type).tp_name).to_string_lossy() };
        ObjectType::Unknown(type_name.to_string())
    }
}

/// Convert a Python value to `serde_json::Value`
impl Serialize for SerializePyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.object_type {
            ObjectType::Str => {
                let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                let uni = unsafe { string::read_utf8_from_str(self.object, &mut str_size) };
                let slice = unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        uni,
                        str_size as usize,
                    ))
                };
                serializer.serialize_str(slice)
            }
            ObjectType::Int => serializer.serialize_i64(unsafe { PyLong_AsLongLong(self.object) }),
            ObjectType::Float => {
                serializer.serialize_f64(unsafe { PyFloat_AS_DOUBLE(self.object) })
            }
            ObjectType::Bool => serializer.serialize_bool(self.object == unsafe { types::TRUE }),
            ObjectType::None => serializer.serialize_unit(),
            ObjectType::Dict => {
                if self.recursion_depth == RECURSION_LIMIT {
                    return Err(ser::Error::custom("Recursion limit reached"));
                }
                let length = unsafe { (*self.object.cast::<PyDictObject>()).ma_used } as usize;
                if length == 0 {
                    serializer.serialize_map(Some(0))?.end()
                } else {
                    let mut map = serializer.serialize_map(Some(length))?;
                    let mut pos = 0_isize;
                    let mut str_size: pyo3::ffi::Py_ssize_t = 0;
                    let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
                    for _ in 0..length {
                        unsafe {
                            pyo3::ffi::PyDict_Next(self.object, &mut pos, &mut key, &mut value);
                        }
                        let uni = unsafe { string::read_utf8_from_str(key, &mut str_size) };
                        let slice = unsafe {
                            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                                uni,
                                str_size as usize,
                            ))
                        };
                        #[allow(clippy::integer_arithmetic)]
                        map.serialize_entry(
                            slice,
                            &SerializePyObject::new(value, self.recursion_depth + 1),
                        )?;
                    }
                    map.end()
                }
            }
            ObjectType::List => {
                if self.recursion_depth == RECURSION_LIMIT {
                    return Err(ser::Error::custom("Recursion limit reached"));
                }
                let length = unsafe { PyList_GET_SIZE(self.object) } as usize;
                if length == 0 {
                    serializer.serialize_seq(Some(0))?.end()
                } else {
                    let mut type_ptr = std::ptr::null_mut();
                    let mut ob_type = ObjectType::Str;
                    let mut sequence = serializer.serialize_seq(Some(length))?;
                    for i in 0..length {
                        let elem = unsafe { PyList_GET_ITEM(self.object, i as isize) };
                        let current_ob_type = unsafe { Py_TYPE(elem) };
                        if current_ob_type != type_ptr {
                            type_ptr = current_ob_type;
                            ob_type = get_object_type(current_ob_type);
                        }
                        #[allow(clippy::integer_arithmetic)]
                        sequence.serialize_element(&SerializePyObject::with_obtype(
                            elem,
                            ob_type.clone(),
                            self.recursion_depth + 1,
                        ))?;
                    }
                    sequence.end()
                }
            }
            ObjectType::Unknown(ref type_name) => Err(ser::Error::custom(format!(
                "Unsupported type: '{}'",
                type_name
            ))),
        }
    }
}

#[inline]
pub(crate) fn to_value(object: &PyAny) -> PyResult<serde_json::Value> {
    serde_json::to_value(SerializePyObject::new(object.as_ptr(), 0))
        .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
}
