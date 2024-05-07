# jsonschema

[![ci](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![codecov](https://codecov.io/gh/Stranger6667/jsonschema-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Stranger6667/jsonschema-rs)
[![Crates.io](https://img.shields.io/crates/v/jsonschema.svg)](https://crates.io/crates/jsonschema)
[![docs.rs](https://docs.rs/jsonschema/badge.svg)](https://docs.rs/jsonschema/)
[![gitter](https://img.shields.io/gitter/room/Stranger6667/jsonschema-rs.svg)](https://gitter.im/Stranger6667/jsonschema-rs)

A JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

Supported drafts:

- Draft 7 (except optional `idn-hostname.json` test case)
- Draft 6
- Draft 4 (except optional `bignum.json` test case)

Partially supported drafts (some keywords are not implemented):
- Draft 2019-09 (requires the `draft201909` feature enabled)
- Draft 2020-12 (requires the `draft202012` feature enabled)

```toml
# Cargo.toml
jsonschema = "0.18"
```

To validate documents against some schema and get validation errors (if any):

```rust
use jsonschema::JSONSchema;
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    let compiled = JSONSchema::compile(&schema)
        .expect("A valid schema");
    let result = compiled.validate(&instance);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
            println!(
                "Instance path: {}", error.instance_path
            );
        }
    }
}
```

Each error has an `instance_path` attribute that indicates the path to the erroneous part within the validated instance.
It could be transformed to JSON Pointer via `.to_string()` or to `Vec<String>` via `.into_vec()`.

If you only need to know whether document is valid or not (which is faster):

```rust
use jsonschema::is_valid;
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    assert!(is_valid(&schema, &instance));
}
```

Or use a compiled schema (preferred):

```rust
use jsonschema::JSONSchema;
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    // Draft is detected automatically
    // with fallback to Draft7
    let compiled = JSONSchema::compile(&schema)
        .expect("A valid schema");
    assert!(compiled.is_valid(&instance));
}
```

## Output styles

`jsonschema` supports `basic` & `flag` output styles from Draft 2019-09, so you can serialize the validation results with `serde`:

```rust
use jsonschema::{Output, BasicOutput, JSONSchema};
use serde_json::json;

fn main() {
    let schema_json = json!({
        "title": "string value",
        "type": "string"
    });
    let instance = json!("some string");
    let schema = JSONSchema::compile(&schema_json)
        .expect("A valid schema");
    
    let output: BasicOutput = schema.apply(&instance).basic();
    let output_json = serde_json::to_value(output)
        .expect("Failed to serialize output");
    
    assert_eq!(
        output_json, 
        json!({
            "valid": true,
            "annotations": [
                {
                    "keywordLocation": "",
                    "instanceLocation": "",
                    "annotations": {
                        "title": "string value"
                    }
                }
            ]
        })
    );
}
```

## Custom keywords

`jsonschema` allows you to implement custom validation logic by defining custom keywords.
To use your own keyword, you need to implement the `Keyword` trait and add it to the `JSONSchema` instance via the `with_keyword` method:

```rust
use jsonschema::{
    paths::{JSONPointer, JsonPointerNode},
    ErrorIterator, JSONSchema, Keyword, ValidationError,
};
use serde_json::{json, Map, Value};
use std::iter::once;

struct MyCustomValidator;

impl Keyword for MyCustomValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        // ... validate instance ...
        if !instance.is_object() {
            let error = ValidationError::custom(
                JSONPointer::default(),
                instance_path.into(),
                instance,
                "Boom!",
            );
            Box::new(once(error))
        } else {
            Box::new(None.into_iter())
        }
    }
    fn is_valid(&self, instance: &Value) -> bool {
        // ... determine if instance is valid ...
        true
    }
}

// You can create a factory function, or use a closure to create new validator instances.
fn custom_validator_factory<'a>(
    // Parent object where your keyword is defined
    parent: &'a Map<String, Value>,
    // Your keyword value
    value: &'a Value,
    // JSON Pointer to your keyword within the schema
    path: JSONPointer,
) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
    // You may return validation error if the keyword is misused for some reason
    Ok(Box::new(MyCustomValidator))
}

fn main() {
    let schema = json!({"my-type": "my-schema"});
    let instance = json!({"a": "b"});
    let compiled = JSONSchema::options()
        // Register your keyword via a factory function
        .with_keyword("my-type", custom_validator_factory)
        // Or use a closure
        .with_keyword("my-type-with-closure", |_, _, _| Ok(Box::new(MyCustomValidator)))
        .compile(&schema)
        .expect("A valid schema");
    assert!(compiled.is_valid(instance));
}
```

## Reference resolving and TLS

By default, `jsonschema` resolves HTTP references via `reqwest` without TLS support.
If you'd like to resolve HTTPS, you need to enable TLS support in `reqwest`:

```toml
reqwest = { version = "*", features = [ "rustls-tls" ] }
```

Otherwise, you might get validation errors like `invalid URL, scheme is not http`.

## Status

This library is functional and ready for use, but its API is still evolving to the 1.0 API.

## Bindings

- Python - See the `./bindings/python` directory
- Ruby - a [crate](https://github.com/driv3r/rusty_json_schema) by @driv3r
- NodeJS - a [package](https://github.com/ahungrynoob/jsonschema) by @ahungrynoob

## Running tests

The tests in [jsonschema/](jsonschema/) depend on the [JSON Schema Test Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite). Before calling `cargo test`, download the suite:

```bash
$ git submodule init
$ git submodule update
```
These commands clone the suite to [jsonschema/tests/suite/](jsonschema/tests/suite/).

Now, enter the `jsonschema` directory and run `cargo test`.

```bash
$ cd jsonschema
$ cargo test
```

## Performance

There is a comparison with other JSON Schema validators written in Rust - `jsonschema_valid==0.5.2` and `valico==4.0.0`.

Test machine i8700K (12 cores), 32GB RAM.

Input values and schemas:

- [Zuora](https://github.com/APIs-guru/openapi-directory/blob/master/APIs/zuora.com/2021-04-23/openapi.yaml) OpenAPI schema (`zuora.json`). Validated against [OpenAPI 3.0 JSON Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v3.0/schema.json) (`openapi.json`).
- [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml) Swagger schema (`kubernetes.json`). Validated against [Swagger JSON Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json) (`swagger.json`).
- Canadian border in GeoJSON format (`canada.json`). Schema is taken from the [GeoJSON website](https://geojson.org/schema/FeatureCollection.json) (`geojson.json`).
- Concert data catalog (`citm_catalog.json`). Schema is inferred via [infers-jsonschema](https://github.com/Stranger6667/infers-jsonschema) & manually adjusted (`citm_catalog_schema.json`).
- `Fast` is taken from [fastjsonschema benchmarks](https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15) (`fast_schema.json`, `fast_valid.json` and `fast_invalid.json`).

| Case           | Schema size | Instance size |
| -------------- | ----------- | ------------- |
| OpenAPI        | 18 KB       | 4.5 MB        |
| Swagger        | 25 KB       | 3.0 MB        |
| Canada         | 4.8 KB      | 2.1 MB        |
| CITM catalog   | 2.3 KB      | 501 KB        |
| Fast (valid)   | 595 B       | 55 B          |
| Fast (invalid) | 595 B       | 60 B          |

Here is the average time for each contender to validate. Ratios are given against compiled `JSONSchema` using its `validate` method. The `is_valid` method is faster, but gives only a boolean return value:

| Case           | jsonschema_valid        | valico                  | jsonschema (validate) | jsonschema (is_valid)  |
| -------------- | ----------------------- | ----------------------- | --------------------- | ---------------------- |
| OpenAPI        |                   - (1) |                   - (1) |              3.500 ms |   3.147 ms (**x0.89**) |
| Swagger        |                   - (2) |  180.65 ms (**x32.12**) |              5.623 ms |   3.634 ms (**x0.64**) |
| Canada         |  40.363 ms (**x33.13**) | 427.40 ms (**x350.90**) |              1.218 ms |   1.217 ms (**x0.99**) |
| CITM catalog   |    5.357 ms (**x2.51**) |  39.215 ms (**x18.44**) |              2.126 ms |  569.23 us (**x0.26**) |
| Fast (valid)   |     2.27 us (**x4.87**) |    6.55 us (**x14.05**) |             465.89 ns |  113.94 ns (**x0.24**) |
| Fast (invalid) |   412.21 ns (**x0.46**) |     6.69 us (**x7.61**) |             878.23 ns |    4.21ns (**x0.004**) |

Notes:

1. `jsonschema_valid` and `valico` do not handle valid path instances matching the `^\\/` regex.

2. `jsonschema_valid` fails to resolve local references (e.g. `#/definitions/definitions`).

You can find benchmark code in `benches/jsonschema.rs`, Rust version is `1.78`.

## Support

If you have anything to discuss regarding this library, please, join our [gitter](https://gitter.im/Stranger6667/jsonschema-rs)!
