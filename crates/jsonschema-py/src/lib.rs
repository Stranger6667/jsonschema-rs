#![allow(clippy::upper_case_acronyms)]

use std::{
    any::Any,
    cell::RefCell,
    panic::{self, AssertUnwindSafe},
};

use jsonschema::{paths::JsonPointer, Draft};
use pyo3::{
    exceptions::{self, PyValueError},
    ffi::PyUnicode_AsUTF8AndSize,
    prelude::*,
    types::{PyAny, PyDict, PyList, PyString, PyType},
    wrap_pyfunction,
};
#[macro_use]
extern crate pyo3_built;

mod ffi;
mod ser;
mod types;

const DRAFT7: u8 = 7;
const DRAFT6: u8 = 6;
const DRAFT4: u8 = 4;
const DRAFT201909: u8 = 19;
const DRAFT202012: u8 = 20;

/// An instance is invalid under a provided schema.
#[pyclass(extends=exceptions::PyValueError, module="jsonschema_rs")]
#[derive(Debug)]
struct ValidationError {
    #[pyo3(get)]
    message: String,
    verbose_message: String,
    #[pyo3(get)]
    schema_path: Py<PyList>,
    #[pyo3(get)]
    instance_path: Py<PyList>,
}

#[pymethods]
impl ValidationError {
    #[new]
    fn new(
        message: String,
        long_message: String,
        schema_path: Py<PyList>,
        instance_path: Py<PyList>,
    ) -> Self {
        ValidationError {
            message,
            verbose_message: long_message,
            schema_path,
            instance_path,
        }
    }
    fn __str__(&self) -> String {
        self.verbose_message.clone()
    }
    fn __repr__(&self) -> String {
        format!("<ValidationError: '{}'>", self.message)
    }
}

#[pyclass]
struct ValidationErrorIter {
    iter: std::vec::IntoIter<PyErr>,
}
#[pymethods]
impl ValidationErrorIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyErr> {
        slf.iter.next()
    }
}

fn into_py_err(py: Python<'_>, error: jsonschema::ValidationError<'_>) -> PyResult<PyErr> {
    let pyerror_type = PyType::new_bound::<ValidationError>(py);
    let message = error.to_string();
    let verbose_message = to_error_message(&error);
    let schema_path = into_path(py, error.schema_path)?;
    let instance_path = into_path(py, error.instance_path)?;
    Ok(PyErr::from_type_bound(
        pyerror_type,
        (message, verbose_message, schema_path, instance_path),
    ))
}

fn into_path(py: Python<'_>, pointer: JsonPointer) -> PyResult<Py<PyList>> {
    let path = PyList::empty_bound(py);
    for chunk in pointer {
        match chunk {
            jsonschema::paths::PathChunk::Property(property) => {
                path.append(property.into_string())?;
            }
            jsonschema::paths::PathChunk::Index(index) => path.append(index)?,
            jsonschema::paths::PathChunk::Keyword(keyword) => path.append(keyword)?,
        };
    }
    Ok(path.unbind())
}

fn get_draft(draft: u8) -> PyResult<Draft> {
    match draft {
        DRAFT4 => Ok(Draft::Draft4),
        DRAFT6 => Ok(Draft::Draft6),
        DRAFT7 => Ok(Draft::Draft7),
        DRAFT201909 => Ok(Draft::Draft201909),
        DRAFT202012 => Ok(Draft::Draft202012),
        _ => Err(exceptions::PyValueError::new_err(format!(
            "Unknown draft: {}",
            draft
        ))),
    }
}

thread_local! {
    static LAST_FORMAT_ERROR: RefCell<Option<PyErr>> = const { RefCell::new(None) };
}

fn make_options(
    draft: Option<u8>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<jsonschema::ValidationOptions> {
    let mut options = jsonschema::options();
    if let Some(raw_draft_version) = draft {
        options.with_draft(get_draft(raw_draft_version)?);
    }
    if let Some(formats) = formats {
        for (name, callback) in formats.iter() {
            if !callback.is_callable() {
                return Err(exceptions::PyValueError::new_err(format!(
                    "Format checker for '{}' must be a callable",
                    name
                )));
            }
            let callback: Py<PyAny> = callback.clone().unbind();
            let call_py_callback = move |value: &str| {
                Python::with_gil(|py| {
                    let value = PyString::new_bound(py, value);
                    callback.call_bound(py, (value,), None)?.is_truthy(py)
                })
            };
            options.with_format(
                name.to_string(),
                move |value: &str| match call_py_callback(value) {
                    Ok(r) => r,
                    Err(e) => {
                        LAST_FORMAT_ERROR.with(|last| {
                            *last.borrow_mut() = Some(e);
                        });
                        std::panic::set_hook(Box::new(|_| {}));
                        // Should be caught
                        panic!("Format checker failed")
                    }
                },
            );
        }
    }
    Ok(options)
}

fn iter_on_error(
    py: Python<'_>,
    validator: &jsonschema::Validator,
    instance: &Bound<'_, PyAny>,
) -> PyResult<ValidationErrorIter> {
    let instance = ser::to_value(instance)?;
    let mut pyerrors = vec![];

    panic::catch_unwind(AssertUnwindSafe(|| {
        if let Err(errors) = validator.validate(&instance) {
            for error in errors {
                pyerrors.push(into_py_err(py, error)?);
            }
        };
        PyResult::Ok(())
    }))
    .map_err(handle_format_checked_panic)??;
    Ok(ValidationErrorIter {
        iter: pyerrors.into_iter(),
    })
}

fn raise_on_error(
    py: Python<'_>,
    validator: &jsonschema::Validator,
    instance: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let instance = ser::to_value(instance)?;
    let result = panic::catch_unwind(AssertUnwindSafe(|| validator.validate(&instance)))
        .map_err(handle_format_checked_panic)?;
    let error = result
        .err()
        .map(|mut errors| errors.next().expect("Iterator should not be empty"));
    error.map_or_else(|| Ok(()), |err| Err(into_py_err(py, err)?))
}

fn to_error_message(error: &jsonschema::ValidationError<'_>) -> String {
    let mut message = error.to_string();
    message.push('\n');
    message.push('\n');
    message.push_str("Failed validating");

    let push_quoted = |m: &mut String, s: &str| {
        m.push('"');
        m.push_str(s);
        m.push('"');
    };

    let push_chunk = |m: &mut String, chunk: &jsonschema::paths::PathChunk| {
        match chunk {
            jsonschema::paths::PathChunk::Property(property) => push_quoted(m, property),
            jsonschema::paths::PathChunk::Index(index) => m.push_str(&index.to_string()),
            jsonschema::paths::PathChunk::Keyword(keyword) => push_quoted(m, keyword),
        };
    };

    if let Some(last) = error.schema_path.last() {
        message.push(' ');
        push_chunk(&mut message, last);
    }
    message.push_str(" in schema");
    let mut chunks = error.schema_path.iter().peekable();
    while let Some(chunk) = chunks.next() {
        // Skip the last element as it is already mentioned in the message
        if chunks.peek().is_none() {
            break;
        }
        message.push('[');
        push_chunk(&mut message, chunk);
        message.push(']');
    }
    message.push('\n');
    message.push('\n');
    message.push_str("On instance");
    for chunk in &error.instance_path {
        message.push('[');
        match chunk {
            jsonschema::paths::PathChunk::Property(property) => push_quoted(&mut message, property),
            jsonschema::paths::PathChunk::Index(index) => message.push_str(&index.to_string()),
            // Keywords are not used for instances
            jsonschema::paths::PathChunk::Keyword(_) => unreachable!("Internal error"),
        };
        message.push(']');
    }
    message.push(':');
    message.push_str("\n    ");
    message.push_str(&error.instance.to_string());
    message
}

/// is_valid(schema, instance, draft=None, with_meta_schemas=False, formats=None)
///
/// A shortcut for validating the input instance against the schema.
///
///     >>> is_valid({"minimum": 5}, 3)
///     False
///
/// If your workflow implies validating against the same schema, consider using `validator_for(...).is_valid`
/// instead.
#[pyfunction]
#[allow(unused_variables)]
#[pyo3(signature = (schema, instance, draft=None, with_meta_schemas=false, formats=None))]
fn is_valid(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
    instance: &Bound<'_, PyAny>,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<bool> {
    let options = make_options(draft, formats)?;
    let schema = ser::to_value(schema)?;
    match options.build(&schema) {
        Ok(validator) => {
            let instance = ser::to_value(instance)?;
            panic::catch_unwind(AssertUnwindSafe(|| Ok(validator.is_valid(&instance))))
                .map_err(handle_format_checked_panic)?
        }
        Err(error) => Err(into_py_err(py, error)?),
    }
}

/// validate(schema, instance, draft=None, with_meta_schemas=False, formats=None)
///
/// Validate the input instance and raise `ValidationError` in the error case
///
///     >>> validate({"minimum": 5}, 3)
///     ...
///     ValidationError: 3 is less than the minimum of 5
///
/// If the input instance is invalid, only the first occurred error is raised.
/// If your workflow implies validating against the same schema, consider using `validator_for(...).validate`
/// instead.
#[pyfunction]
#[allow(unused_variables)]
#[pyo3(signature = (schema, instance, draft=None, with_meta_schemas=false, formats=None))]
fn validate(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
    instance: &Bound<'_, PyAny>,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    let options = make_options(draft, formats)?;
    let schema = ser::to_value(schema)?;
    match options.build(&schema) {
        Ok(validator) => raise_on_error(py, &validator, instance),
        Err(error) => Err(into_py_err(py, error)?),
    }
}

/// iter_errors(schema, instance, draft=None, with_meta_schemas=False, formats=None)
///
/// Iterate the validation errors of the input instance
///
///     >>> next(iter_errors({"minimum": 5}, 3))
///     ...
///     ValidationError: 3 is less than the minimum of 5
///
/// If your workflow implies validating against the same schema, consider using `validator_for().iter_errors`
/// instead.
#[pyfunction]
#[allow(unused_variables)]
#[pyo3(signature = (schema, instance, draft=None, with_meta_schemas=false, formats=None))]
fn iter_errors(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
    instance: &Bound<'_, PyAny>,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<ValidationErrorIter> {
    let options = make_options(draft, formats)?;
    let schema = ser::to_value(schema)?;
    match options.build(&schema) {
        Ok(validator) => iter_on_error(py, &validator, instance),
        Err(error) => Err(into_py_err(py, error)?),
    }
}

/// JSONSchema(schema, draft=None, with_meta_schemas=False)
///
/// A JSON Schema validator.
///
///     >>> validator = JSONSchema({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
/// By default Draft 7 will be used for compilation.
#[pyclass(module = "jsonschema_rs")]
struct JSONSchema {
    validator: jsonschema::Validator,
    repr: String,
}

const SCHEMA_LENGTH_LIMIT: usize = 32;

fn get_schema_repr(schema: &serde_json::Value) -> String {
    // It could be more efficient, without converting the whole Value to a string
    let mut repr = schema.to_string();
    if repr.len() > SCHEMA_LENGTH_LIMIT {
        repr.truncate(SCHEMA_LENGTH_LIMIT);
        repr.push_str("...}");
    }
    repr
}

fn handle_format_checked_panic(err: Box<dyn Any + Send>) -> PyErr {
    LAST_FORMAT_ERROR.with(|last| {
        if let Some(err) = last.borrow_mut().take() {
            let _ = panic::take_hook();
            err
        } else {
            exceptions::PyRuntimeError::new_err(format!("Validation panicked: {:?}", err))
        }
    })
}

#[pyclass(module = "jsonschema_rs", subclass)]
struct Validator {
    validator: jsonschema::Validator,
    repr: String,
}

/// validator_for(schema, formats=None)
///
/// Create a validator for the input schema with automatic draft detection and default options.
///
///     >>> validator = validator_for({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyfunction]
#[pyo3(signature = (schema, formats=None))]
fn validator_for(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<Validator> {
    validator_for_impl(py, schema, None, formats)
}

fn validator_for_impl(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
    draft: Option<u8>,
    formats: Option<&Bound<'_, PyDict>>,
) -> PyResult<Validator> {
    let obj_ptr = schema.as_ptr();
    let object_type = unsafe { pyo3::ffi::Py_TYPE(obj_ptr) };
    let schema = if unsafe { object_type == types::STR_TYPE } {
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let ptr = unsafe { PyUnicode_AsUTF8AndSize(obj_ptr, &mut str_size) };
        let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), str_size as usize) };
        serde_json::from_slice(slice)
            .map_err(|error| PyValueError::new_err(format!("Invalid string: {}", error)))?
    } else {
        ser::to_value(schema)?
    };
    let options = make_options(draft, formats)?;
    match options.build(&schema) {
        Ok(validator) => Ok(Validator {
            validator,
            repr: get_schema_repr(&schema),
        }),
        Err(error) => Err(into_py_err(py, error)?),
    }
}

#[pymethods]
impl Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        validator_for(py, schema, formats)
    }
    /// is_valid(instance)
    ///
    /// Perform fast validation against the schema.
    ///
    ///     >>> validator = validator_for({"minimum": 5})
    ///     >>> validator.is_valid(3)
    ///     False
    ///
    /// The output is a boolean value, that indicates whether the instance is valid or not.
    #[pyo3(text_signature = "(instance)")]
    fn is_valid(&self, instance: &Bound<'_, PyAny>) -> PyResult<bool> {
        let instance = ser::to_value(instance)?;
        panic::catch_unwind(AssertUnwindSafe(|| Ok(self.validator.is_valid(&instance))))
            .map_err(handle_format_checked_panic)?
    }
    /// validate(instance)
    ///
    /// Validate the input instance and raise `ValidationError` in the error case
    ///
    ///     >>> validator = validator_for({"minimum": 5})
    ///     >>> validator.validate(3)
    ///     ...
    ///     ValidationError: 3 is less than the minimum of 5
    ///
    /// If the input instance is invalid, only the first occurred error is raised.
    #[pyo3(text_signature = "(instance)")]
    fn validate(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<()> {
        raise_on_error(py, &self.validator, instance)
    }
    /// iter_errors(instance)
    ///
    /// Iterate the validation errors of the input instance
    ///
    ///     >>> validator = validator_for({"minimum": 5})
    ///     >>> next(validator.iter_errors(3))
    ///     ...
    ///     ValidationError: 3 is less than the minimum of 5
    #[pyo3(text_signature = "(instance)")]
    fn iter_errors(
        &self,
        py: Python<'_>,
        instance: &Bound<'_, PyAny>,
    ) -> PyResult<ValidationErrorIter> {
        iter_on_error(py, &self.validator, instance)
    }
    fn __repr__(&self) -> String {
        let draft = match self.validator.draft() {
            Draft::Draft4 => "Draft4",
            Draft::Draft6 => "Draft6",
            Draft::Draft7 => "Draft7",
            Draft::Draft201909 => "Draft201909",
            Draft::Draft202012 => "Draft202012",
            _ => "Unknown",
        };
        format!("<{draft}Validator: {}>", self.repr)
    }
}

#[pymethods]
impl JSONSchema {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature = (schema, draft=None, with_meta_schemas=false, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        draft: Option<u8>,
        with_meta_schemas: Option<bool>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let options = make_options(draft, formats)?;
        let raw_schema = ser::to_value(schema)?;
        match options.build(&raw_schema) {
            Ok(validator) => Ok(JSONSchema {
                validator,
                repr: get_schema_repr(&raw_schema),
            }),
            Err(error) => Err(into_py_err(py, error)?),
        }
    }
    /// from_str(string, draft=None, with_meta_schemas=False, formats=None)
    ///
    /// Create `JSONSchema` from a serialized JSON string.
    ///
    ///     >>> validator = JSONSchema.from_str('{"minimum": 5}')
    ///
    /// Use it if you have your schema as a string and want to utilize Rust JSON parsing.
    #[classmethod]
    #[allow(unused_variables)]
    #[pyo3(signature = (string, draft=None, with_meta_schemas=false, formats=None))]
    fn from_str(
        _: &Bound<'_, PyType>,
        py: Python<'_>,
        string: &Bound<'_, PyAny>,
        draft: Option<u8>,
        with_meta_schemas: Option<bool>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let obj_ptr = string.as_ptr();
        let object_type = unsafe { pyo3::ffi::Py_TYPE(obj_ptr) };
        if unsafe { object_type != types::STR_TYPE } {
            let type_name =
                unsafe { std::ffi::CStr::from_ptr((*object_type).tp_name).to_string_lossy() };
            Err(PyValueError::new_err(format!(
                "Expected string, got {}",
                type_name
            )))
        } else {
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let ptr = unsafe { PyUnicode_AsUTF8AndSize(obj_ptr, &mut str_size) };
            let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), str_size as usize) };
            let raw_schema = serde_json::from_slice(slice)
                .map_err(|error| PyValueError::new_err(format!("Invalid string: {}", error)))?;
            let options = make_options(draft, formats)?;
            match options.build(&raw_schema) {
                Ok(validator) => Ok(JSONSchema {
                    validator,
                    repr: get_schema_repr(&raw_schema),
                }),
                Err(error) => Err(into_py_err(py, error)?),
            }
        }
    }

    /// is_valid(instance)
    ///
    /// Perform fast validation against the schema.
    ///
    ///     >>> validator = JSONSchema({"minimum": 5})
    ///     >>> validator.is_valid(3)
    ///     False
    ///
    /// The output is a boolean value, that indicates whether the instance is valid or not.
    #[pyo3(text_signature = "(instance)")]
    fn is_valid(&self, instance: &Bound<'_, PyAny>) -> PyResult<bool> {
        let instance = ser::to_value(instance)?;
        panic::catch_unwind(AssertUnwindSafe(|| Ok(self.validator.is_valid(&instance))))
            .map_err(handle_format_checked_panic)?
    }

    /// validate(instance)
    ///
    /// Validate the input instance and raise `ValidationError` in the error case
    ///
    ///     >>> validator = JSONSchema({"minimum": 5})
    ///     >>> validator.validate(3)
    ///     ...
    ///     ValidationError: 3 is less than the minimum of 5
    ///
    /// If the input instance is invalid, only the first occurred error is raised.
    #[pyo3(text_signature = "(instance)")]
    fn validate(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<()> {
        raise_on_error(py, &self.validator, instance)
    }

    /// iter_errors(instance)
    ///
    /// Iterate the validation errors of the input instance
    ///
    ///     >>> validator = JSONSchema({"minimum": 5})
    ///     >>> next(validator.iter_errors(3))
    ///     ...
    ///     ValidationError: 3 is less than the minimum of 5
    #[pyo3(text_signature = "(instance)")]
    fn iter_errors(
        &self,
        py: Python<'_>,
        instance: &Bound<'_, PyAny>,
    ) -> PyResult<ValidationErrorIter> {
        iter_on_error(py, &self.validator, instance)
    }
    fn __repr__(&self) -> String {
        format!("<JSONSchema: {}>", self.repr)
    }
}

/// Draft4Validator(schema, formats=None)
///
/// A JSON Schema Draft 4 validator.
///
///     >>> validator = Draft4Validator({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyclass(module = "jsonschema_rs", extends=Validator, subclass)]
struct Draft4Validator {}

#[pymethods]
impl Draft4Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<(Self, Validator)> {
        Ok((
            Draft4Validator {},
            validator_for_impl(py, schema, Some(DRAFT4), formats)?,
        ))
    }
}

/// Draft6Validator(schema, formats=None)
///
/// A JSON Schema Draft 6 validator.
///
///     >>> validator = Draft6Validator({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyclass(module = "jsonschema_rs", extends=Validator, subclass)]
struct Draft6Validator {}

#[pymethods]
impl Draft6Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<(Self, Validator)> {
        Ok((
            Draft6Validator {},
            validator_for_impl(py, schema, Some(DRAFT6), formats)?,
        ))
    }
}

/// Draft7Validator(schema, formats=None)
///
/// A JSON Schema Draft 7 validator.
///
///     >>> validator = Draft7Validator({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyclass(module = "jsonschema_rs", extends=Validator, subclass)]
struct Draft7Validator {}

#[pymethods]
impl Draft7Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<(Self, Validator)> {
        Ok((
            Draft7Validator {},
            validator_for_impl(py, schema, Some(DRAFT7), formats)?,
        ))
    }
}

/// Draft201909Validator(schema, formats=None)
///
/// A JSON Schema Draft 2019-09 validator.
///
///     >>> validator = Draft201909Validator({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyclass(module = "jsonschema_rs", extends=Validator, subclass)]
struct Draft201909Validator {}

#[pymethods]
impl Draft201909Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<(Self, Validator)> {
        Ok((
            Draft201909Validator {},
            validator_for_impl(py, schema, Some(DRAFT201909), formats)?,
        ))
    }
}

/// Draft202012Validator(schema, formats=None)
///
/// A JSON Schema Draft 2020-12 validator.
///
///     >>> validator = Draft202012Validator({"minimum": 5})
///     >>> validator.is_valid(3)
///     False
///
#[pyclass(module = "jsonschema_rs", extends=Validator, subclass)]
struct Draft202012Validator {}

#[pymethods]
impl Draft202012Validator {
    #[new]
    #[pyo3(signature = (schema, formats=None))]
    fn new(
        py: Python<'_>,
        schema: &Bound<'_, PyAny>,
        formats: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<(Self, Validator)> {
        Ok((
            Draft202012Validator {},
            validator_for_impl(py, schema, Some(DRAFT202012), formats)?,
        ))
    }
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// JSON Schema validation for Python written in Rust.
#[pymodule]
fn jsonschema_rs(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    // To provide proper signatures for PyCharm, all the functions have their signatures as the
    // first line in docstrings. The idea is taken from NumPy.
    types::init();
    module.add_wrapped(wrap_pyfunction!(is_valid))?;
    module.add_wrapped(wrap_pyfunction!(validate))?;
    module.add_wrapped(wrap_pyfunction!(iter_errors))?;
    module.add_wrapped(wrap_pyfunction!(validator_for))?;
    module.add_class::<JSONSchema>()?;
    module.add_class::<Draft4Validator>()?;
    module.add_class::<Draft6Validator>()?;
    module.add_class::<Draft7Validator>()?;
    module.add_class::<Draft201909Validator>()?;
    module.add_class::<Draft202012Validator>()?;
    module.add("ValidationError", py.get_type_bound::<ValidationError>())?;
    module.add("Draft4", DRAFT4)?;
    module.add("Draft6", DRAFT6)?;
    module.add("Draft7", DRAFT7)?;
    module.add("Draft201909", DRAFT201909)?;
    module.add("Draft202012", DRAFT202012)?;

    // Add build metadata to ease triaging incoming issues
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;

    Ok(())
}
