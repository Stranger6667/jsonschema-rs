use std::os::raw::c_char;

/// ``ob_type`` is not exposed by default as is needed to check if something inherits from a enum.
#[repr(C)]
#[derive(Debug)]
pub struct PyTypeObject {
    pub ob_refcnt: pyo3::ffi::Py_ssize_t,
    pub ob_type: *mut pyo3::ffi::PyTypeObject,
    pub tp_name: *const c_char,
}
