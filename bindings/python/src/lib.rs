#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::map_unwrap_or,
    clippy::unseparated_literal_suffix,
    clippy::cargo,
    clippy::unwrap_used
)]
#![allow(clippy::upper_case_acronyms)]

use jsonschema::{paths::JSONPointer, Draft};
use pyo3::{
    exceptions,
    prelude::*,
    types::{PyAny, PyList, PyType},
    wrap_pyfunction, AsPyPointer, PyObjectProtocol,
};
use serde_json::Value;

mod ser;
mod string;
mod types;

const DRAFT7: u8 = 7;
const DRAFT6: u8 = 6;
const DRAFT4: u8 = 4;

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
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for ValidationError {
    fn __str__(&'p self) -> PyResult<String> {
        Ok(self.verbose_message.clone())
    }
    fn __repr__(&'p self) -> PyResult<String> {
        Ok(format!("<ValidationError: '{}'>", self.message))
    }
}

fn into_py_err(py: Python, error: jsonschema::ValidationError) -> PyResult<PyErr> {
    let pyerror_type = PyType::new::<ValidationError>(py);
    let message = error.to_string();
    let verbose_message = to_error_message(&error);
    let schema_path = into_path(py, error.schema_path)?;
    let instance_path = into_path(py, error.instance_path)?;
    Ok(PyErr::from_type(
        pyerror_type,
        (message, verbose_message, schema_path, instance_path),
    ))
}

fn into_path(py: Python, pointer: JSONPointer) -> PyResult<Py<PyList>> {
    let path = PyList::empty(py);
    for chunk in pointer {
        match chunk {
            jsonschema::paths::PathChunk::Property(property) => path.append(property)?,
            jsonschema::paths::PathChunk::Index(index) => path.append(index)?,
            jsonschema::paths::PathChunk::Keyword(keyword) => path.append(keyword)?,
        };
    }
    Ok(path.into_py(py))
}

fn get_draft(draft: u8) -> PyResult<Draft> {
    match draft {
        DRAFT4 => Ok(jsonschema::Draft::Draft4),
        DRAFT6 => Ok(jsonschema::Draft::Draft6),
        DRAFT7 => Ok(jsonschema::Draft::Draft7),
        _ => Err(exceptions::PyValueError::new_err(format!(
            "Unknown draft: {}",
            draft
        ))),
    }
}

fn make_options(
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
) -> PyResult<jsonschema::CompilationOptions> {
    let mut options = jsonschema::JSONSchema::options();
    if let Some(raw_draft_version) = draft {
        options.with_draft(get_draft(raw_draft_version)?);
    }
    if let Some(true) = with_meta_schemas {
        options.with_meta_schemas();
    }
    Ok(options)
}

fn raise_on_error(py: Python, compiled: &jsonschema::JSONSchema, instance: &PyAny) -> PyResult<()> {
    let instance = ser::to_value(instance)?;
    let result = compiled.validate(&instance);
    let error = result
        .err()
        .map(|mut errors| errors.next().expect("Iterator should not be empty"));
    error.map_or_else(|| Ok(()), |err| Err(into_py_err(py, err)?))
}

fn to_error_message(error: &jsonschema::ValidationError) -> String {
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
        push_chunk(&mut message, last)
    }
    message.push_str(" in schema");
    for chunk in &error.schema_path {
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

/// is_valid(schema, instance, draft=None, with_meta_schemas=False)
///
/// A shortcut for validating the input instance against the schema.
///
///     >>> is_valid({"minimum": 5}, 3)
///     False
///
/// If your workflow implies validating against the same schema, consider using `JSONSchema.is_valid`
/// instead.
#[pyfunction]
#[pyo3(text_signature = "(schema, instance, draft=None, with_meta_schemas=False)")]
fn is_valid(
    py: Python,
    schema: &PyAny,
    instance: &PyAny,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
) -> PyResult<bool> {
    let options = make_options(draft, with_meta_schemas)?;
    let schema = ser::to_value(schema)?;
    match options.compile(&schema) {
        Ok(compiled) => {
            let instance = ser::to_value(instance)?;
            Ok(compiled.is_valid(&instance))
        }
        Err(error) => Err(into_py_err(py, error)?),
    }
}

/// validate(schema, instance, draft=None, with_meta_schemas=False)
///
/// Validate the input instance and raise `ValidationError` in the error case
///
///     >>> validate({"minimum": 5}, 3)
///     ...
///     ValidationError: 3 is less than the minimum of 5
///
/// If the input instance is invalid, only the first occurred error is raised.
/// If your workflow implies validating against the same schema, consider using `JSONSchema.validate`
/// instead.
#[pyfunction]
#[pyo3(text_signature = "(schema, instance, draft=None, with_meta_schemas=False)")]
fn validate(
    py: Python,
    schema: &PyAny,
    instance: &PyAny,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
) -> PyResult<()> {
    let options = make_options(draft, with_meta_schemas)?;
    let schema = ser::to_value(schema)?;
    match options.compile(&schema) {
        Ok(compiled) => raise_on_error(py, &compiled, instance),
        Err(error) => Err(into_py_err(py, error)?),
    }
}

/// JSONSchema(schema, draft=None, with_meta_schemas=False)
///
/// JSON Schema compiled into a validation tree.
///
///     >>> compiled = JSONSchema({"minimum": 5})
///     >>> compiled.is_valid(3)
///     False
///
/// By default Draft 7 will be used for compilation.
#[pyclass]
#[pyo3(text_signature = "(schema, draft=None, with_meta_schemas=False)")]
struct JSONSchema {
    schema: jsonschema::JSONSchema<'static>,
    raw_schema: &'static Value,
}

#[pymethods]
impl JSONSchema {
    #[new]
    fn new(
        py: Python,
        pyschema: &PyAny,
        draft: Option<u8>,
        with_meta_schemas: Option<bool>,
    ) -> PyResult<Self> {
        let options = make_options(draft, with_meta_schemas)?;
        // Currently, it is the simplest way to pass a reference to `JSONSchema`
        // It is cleaned up in the `Drop` implementation
        let raw_schema = ser::to_value(pyschema)?;
        let schema: &'static Value = Box::leak(Box::new(raw_schema));
        match options.compile(schema) {
            Ok(compiled) => Ok(JSONSchema {
                schema: compiled,
                raw_schema: schema,
            }),
            Err(error) => Err(into_py_err(py, error)?),
        }
    }

    /// is_valid(instance)
    ///
    /// Perform fast validation against the compiled schema.
    ///
    ///     >>> compiled = JSONSchema({"minimum": 5})
    ///     >>> compiled.is_valid(3)
    ///     False
    ///
    /// The output is a boolean value, that indicates whether the instance is valid or not.
    #[pyo3(text_signature = "(instance)")]
    fn is_valid(&self, instance: &PyAny) -> PyResult<bool> {
        let instance = ser::to_value(instance)?;
        Ok(self.schema.is_valid(&instance))
    }

    /// validate(instance)
    ///
    /// Validate the input instance and raise `ValidationError` in the error case
    ///
    ///     >>> compiled = JSONSchema({"minimum": 5})
    ///     >>> compiled.validate(3)
    ///     ...
    ///     ValidationError: 3 is less than the minimum of 5
    ///
    /// If the input instance is invalid, only the first occurred error is raised.
    #[pyo3(text_signature = "(instance)")]
    fn validate(&self, py: Python, instance: &PyAny) -> PyResult<()> {
        raise_on_error(py, &self.schema, instance)
    }
}

const SCHEMA_LENGTH_LIMIT: usize = 32;

#[pyproto]
impl<'p> PyObjectProtocol<'p> for JSONSchema {
    fn __repr__(&self) -> PyResult<String> {
        let mut schema = self.raw_schema.to_string();
        if schema.len() > SCHEMA_LENGTH_LIMIT {
            schema.truncate(SCHEMA_LENGTH_LIMIT);
            schema = format!("{}...}}", schema);
        }
        Ok(format!("<JSONSchema: {}>", schema))
    }
}

impl Drop for JSONSchema {
    fn drop(&mut self) {
        // Since `self.raw_schema` is not used anywhere else, there should be no double-free
        unsafe { Box::from_raw(self.raw_schema as *const _ as *mut Value) };
    }
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// JSON Schema validation for Python written in Rust.
#[pymodule]
fn jsonschema_rs(py: Python, module: &PyModule) -> PyResult<()> {
    // To provide proper signatures for PyCharm, all the functions have their signatures as the
    // first line in docstrings. The idea is taken from NumPy.
    types::init();
    module.add_wrapped(wrap_pyfunction!(is_valid))?;
    module.add_wrapped(wrap_pyfunction!(validate))?;
    module.add_class::<JSONSchema>()?;
    module.add("ValidationError", py.get_type::<ValidationError>())?;
    module.add("Draft4", DRAFT4)?;
    module.add("Draft6", DRAFT6)?;
    module.add("Draft7", DRAFT7)?;

    // Allow deprecated, unless `pyo3-built` is updated
    #[allow(deprecated)]
    // Add build metadata to ease triaging incoming issues
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;

    Ok(())
}
