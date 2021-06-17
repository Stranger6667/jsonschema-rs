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
    clippy::unwrap_used
)]
#![allow(clippy::upper_case_acronyms)]
use jsonschema::Draft;
use pyo3::{
    create_exception, exceptions, prelude::*, types::PyAny, wrap_pyfunction, PyObjectProtocol,
};
use serde_json::Value;

mod ser;
mod string;
mod types;

const VALIDATION_ERROR_DOCSTRING: &str = "An error that can occur during validation";
const DRAFT7: u8 = 7;
const DRAFT6: u8 = 6;
const DRAFT4: u8 = 4;

create_exception!(jsonschema_rs, ValidationError, exceptions::PyValueError);

struct ValidationErrorWrapper<'a>(jsonschema::ValidationError<'a>);

impl<'a> From<ValidationErrorWrapper<'a>> for PyErr {
    fn from(error: ValidationErrorWrapper<'a>) -> PyErr {
        ValidationError::new_err(to_error_message(&error.0))
    }
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

fn raise_on_error(compiled: &jsonschema::JSONSchema, instance: &PyAny) -> PyResult<()> {
    let instance = ser::to_value(instance)?;
    let result = compiled.validate(&instance);
    let error = if let Some(mut errors) = result.err() {
        // If we have `Err` case, then the iterator is not empty
        Some(errors.next().expect("Iterator should not be empty"))
    } else {
        None
    };
    error.map_or_else(
        || Ok(()),
        |err| {
            let message = to_error_message(&err);
            Err(ValidationError::new_err(message))
        },
    )
}

fn to_error_message(error: &jsonschema::ValidationError) -> String {
    let mut message = error.to_string();
    message.push('\n');
    message.push('\n');
    message.push_str("On instance");
    for chunk in &error.instance_path {
        message.push('[');
        match chunk {
            jsonschema::paths::PathChunk::Property(property) => {
                message.push('"');
                message.push_str(property);
                message.push('"');
            }
            jsonschema::paths::PathChunk::Index(index) => message.push_str(&index.to_string()),
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
#[text_signature = "(schema, instance, draft=None, with_meta_schemas=False)"]
fn is_valid(
    schema: &PyAny,
    instance: &PyAny,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
) -> PyResult<bool> {
    let options = make_options(draft, with_meta_schemas)?;
    let schema = ser::to_value(schema)?;
    let compiled = options.compile(&schema).map_err(ValidationErrorWrapper)?;
    let instance = ser::to_value(instance)?;
    Ok(compiled.is_valid(&instance))
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
#[text_signature = "(schema, instance, draft=None, with_meta_schemas=False)"]
fn validate(
    schema: &PyAny,
    instance: &PyAny,
    draft: Option<u8>,
    with_meta_schemas: Option<bool>,
) -> PyResult<()> {
    let options = make_options(draft, with_meta_schemas)?;
    let schema = ser::to_value(schema)?;
    let compiled = options.compile(&schema).map_err(ValidationErrorWrapper)?;
    raise_on_error(&compiled, instance)
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
#[text_signature = "(schema, draft=None, with_meta_schemas=False)"]
struct JSONSchema {
    schema: jsonschema::JSONSchema<'static>,
    raw_schema: &'static Value,
}

#[pymethods]
impl JSONSchema {
    #[new]
    fn new(schema: &PyAny, draft: Option<u8>, with_meta_schemas: Option<bool>) -> PyResult<Self> {
        let options = make_options(draft, with_meta_schemas)?;
        // Currently, it is the simplest way to pass a reference to `JSONSchema`
        // It is cleaned up in the `Drop` implementation
        let raw_schema = ser::to_value(schema)?;
        let schema: &'static Value = Box::leak(Box::new(raw_schema));
        Ok(JSONSchema {
            schema: options.compile(schema).map_err(ValidationErrorWrapper)?,
            raw_schema: schema,
        })
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
    #[text_signature = "(instance)"]
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
    #[text_signature = "(instance)"]
    fn validate(&self, instance: &PyAny) -> PyResult<()> {
        raise_on_error(&self.schema, instance)
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
    let validation_error = py.get_type::<ValidationError>();
    validation_error.setattr("__doc__", VALIDATION_ERROR_DOCSTRING)?;
    module.add("ValidationError", validation_error)?;
    module.add("Draft4", DRAFT4)?;
    module.add("Draft6", DRAFT6)?;
    module.add("Draft7", DRAFT7)?;

    // Add build metadata to ease triaging incoming issues
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;

    Ok(())
}
