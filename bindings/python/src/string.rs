use pyo3::ffi::{
    PyBytes_AsString, PyTypeObject, PyUnicode_AsUTF8String, PyUnicode_GetLength, Py_hash_t,
    Py_ssize_t,
};
use std::os::raw::c_char;

#[repr(C)]
pub struct PyAsciiObject {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub length: Py_ssize_t,
    pub hash: Py_hash_t,
    pub state: u32,
    pub wstr: *mut c_char,
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub ob_base: PyAsciiObject,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    pub wstr_length: Py_ssize_t,
}

const STATE_ASCII: u32 = 0b0000_0000_0000_0000_0000_0000_0100_0000;
const STATE_COMPACT: u32 = 0b0000_0000_0000_0000_0000_0000_0010_0000;

/// Read a UTF-8 string from a pointer and change the given size.
pub unsafe fn read_utf8_from_str(
    object_pointer: *mut pyo3::ffi::PyObject,
    size: &mut Py_ssize_t,
) -> *const u8 {
    if (*object_pointer.cast::<PyAsciiObject>()).state & STATE_ASCII == STATE_ASCII {
        *size = (*object_pointer.cast::<PyAsciiObject>()).length;
        object_pointer.cast::<PyAsciiObject>().offset(1) as *const u8
    } else if (*object_pointer.cast::<PyAsciiObject>()).state & STATE_COMPACT == STATE_COMPACT
        && !(*object_pointer.cast::<PyCompactUnicodeObject>())
            .utf8
            .is_null()
    {
        *size = (*object_pointer.cast::<PyCompactUnicodeObject>()).utf8_length;
        (*object_pointer.cast::<PyCompactUnicodeObject>()).utf8 as *const u8
    } else {
        *size = PyUnicode_GetLength(object_pointer);
        let bytes = PyUnicode_AsUTF8String(object_pointer);
        PyBytes_AsString(bytes) as *const u8
    }
}
