#![feature(core_intrinsics)]
#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::result_unwrap_used,
    clippy::result_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::option_map_unwrap_or_else,
    clippy::option_map_unwrap_or
)]
use jsonschema::Draft;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::{exceptions, wrap_pyfunction, PyObjectProtocol};
use serde_json::Value;

mod ser;
mod string;
mod types;

const MODULE_DOCSTRING: &str = "JSON Schema validation for Python written in Rust.";
const DRAFT7: u8 = 7;
const DRAFT6: u8 = 6;
const DRAFT4: u8 = 4;

#[derive(Debug)]
enum JSONSchemaError {
    Compilation(jsonschema::CompilationError),
}

impl From<JSONSchemaError> for PyErr {
    fn from(error: JSONSchemaError) -> PyErr {
        exceptions::ValueError::py_err(match error {
            JSONSchemaError::Compilation(_) => "Invalid schema",
        })
    }
}

fn get_draft(draft: Option<u8>) -> PyResult<Draft> {
    if let Some(value) = draft {
        match value {
            DRAFT4 => Ok(jsonschema::Draft::Draft4),
            DRAFT6 => Ok(jsonschema::Draft::Draft6),
            DRAFT7 => Ok(jsonschema::Draft::Draft7),
            _ => Err(exceptions::ValueError::py_err(format!(
                "Unknown draft: {}",
                value
            ))),
        }
    } else {
        Ok(jsonschema::Draft::default())
    }
}

/// A shortcut for validating the input instance against the schema.
///
///     >>> is_valid({"minimum": 5}, 3)
///     False
///
/// If your workflow implies validating against the same schema, consider using `JSONSchema.is_valid`
/// instead.
#[pyfunction]
#[text_signature = "(schema, instance, draft=None)"]
fn is_valid(schema: &PyAny, instance: &PyAny, draft: Option<u8>) -> PyResult<bool> {
    let draft = get_draft(draft).map(Some)?;
    let schema = ser::to_value(schema)?;
    let instance = ser::to_value(instance)?;
    let compiled =
        jsonschema::JSONSchema::compile(&schema, draft).map_err(JSONSchemaError::Compilation)?;
    Ok(compiled.is_valid(&instance))
}

/// JSON Schema compiled into a validation tree.
///
///     >>> compiled = JSONSchema({"minimum": 5})
///     >>> compiled.is_valid(3)
///     False
///
/// By default Draft 7 will be used for compilation.
#[pyclass]
#[text_signature = "(schema, draft=None)"]
struct JSONSchema {
    schema: jsonschema::JSONSchema<'static>,
    raw_schema: &'static Value,
}

#[pymethods]
impl JSONSchema {
    #[new]
    fn new(schema: &PyAny, draft: Option<u8>) -> PyResult<Self> {
        let draft = get_draft(draft).map(Some)?;
        let raw_schema = ser::to_value(schema)?;
        // Currently, it is the simplest way to pass a reference to `JSONSchema`
        // It is cleaned up in the `Drop` implementation
        let schema: &'static Value = Box::leak(Box::new(raw_schema));
        Ok(JSONSchema {
            schema: jsonschema::JSONSchema::compile(schema, draft)
                .map_err(JSONSchemaError::Compilation)?,
            raw_schema: schema,
        })
    }

    /// Perform fast validation against the compiled schema.
    ///
    ///     >>> compiled = JSONSchema({"minimum": 5})
    ///     >>> compiled.is_valid(3)
    ///     False
    ///
    /// The output is a boolean value, that indicates whether the instance is valid or not.
    #[text_signature = "(instance)"]
    fn is_valid(&self, instance: &PyAny) -> bool {
        let instance = ser::to_value(instance).unwrap();
        self.schema.is_valid(&instance)
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

#[pymodule]
fn jsonschema_rs(_py: Python, module: &PyModule) -> PyResult<()> {
    types::init();
    module.add_wrapped(wrap_pyfunction!(is_valid))?;
    module.add_class::<JSONSchema>()?;
    module.add("Draft4", DRAFT4)?;
    module.add("Draft6", DRAFT6)?;
    module.add("Draft7", DRAFT7)?;
    module.add("__doc__", MODULE_DOCSTRING)?;
    Ok(())
}
