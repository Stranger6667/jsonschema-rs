# jsonschema

[![ci](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![codecov](https://codecov.io/gh/Stranger6667/jsonschema-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Stranger6667/jsonschema-rs)
[![Crates.io](https://img.shields.io/crates/v/jsonschema.svg)](https://crates.io/crates/jsonschema)
[![docs.rs](https://docs.rs/jsonschema/badge.svg?version=0.5.0)](https://docs.rs/jsonschema/0.5.0/jsonschema/)
[![gitter](https://img.shields.io/gitter/room/Stranger6667/jsonschema-rs.svg)](https://gitter.im/Stranger6667/jsonschema-rs)

A JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

Supported drafts:

- Draft 7 (except optional `idn-hostname.json`, `float_overflow.json` and `format_email.json` test cases)
- Draft 6 (except optional `float_overflow.json` and `format_email.json` test cases)
- Draft 4 (except optional `bignum.json`, `float_overflow.json` and `format_email.json` test cases)

```toml
# Cargo.toml
jsonschema = "0.5"
```

To validate documents against some schema and get validation errors (if any):

```rust
use jsonschema::{JSONSchema, Draft, CompilationError};
use serde_json::json;

fn main() -> Result<(), CompilationError> {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    let compiled = JSONSchema::compile(&schema)?;
    let result = compiled.validate(&instance);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error)
        }
    }
    Ok(())
}
```

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
use jsonschema::{JSONSchema, Draft, CompilationError};
use serde_json::json;

fn main() -> Result<(), CompilationError> {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    // Draft is detected automatically with fallback to Draft7
    let compiled = JSONSchema::compile(&schema)?;
    assert!(compiled.is_valid(&instance));
    Ok(())
}
```

## Bindings

- Python - See the `/python` directory
- Ruby - a [crate](https://github.com/driv3r/rusty_json_schema) by @driv3r

## Performance

There is a comparison with other JSON Schema validators written in Rust - `jsonschema_valid==0.4.0` and `valico==3.5.0`.

Test machine i8700K (12 cores), 32GB RAM.

Schemas & input values:

- Big valid input. It is an Open API 2.0 schema for [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml) which is ~3.15 MB (`kubernetes.json` and `swagger.json` files)
- Small valid input (`small_schema.json` and `small_valid.json`)
- Small invalid input (`small_schema.json` and `small_invalid.json`)

Ratios are given against compiled `JSONSchema` using its `validate`. The `is_valid` method is faster, but gives only a boolean return value:

| Case          | jsonschema_valid        | valico                  | jsonschema.validate   | jsonschema.is_valid    |
| ------------- | ----------------------- | ----------------------- | --------------------- | ---------------------- |
| Big valid     | -                       | 95.008 ms (**x13.07**)  | 7.264 ms              | 5.974 ms (**x0.82**)   |
| Small valid   | 2.04 us    (**x5.45**)  | 3.67 us   (**x9.81**)   | 373.91 ns             | 119.02 ns (**x0.31**)  |
| Small invalid | 397.52 ns  (**x0.82**)  | 3.73 us   (**x7.74**)   | 481.33 ns             | 5.26 ns  (**x0.01**)   |

Unfortunately, `jsonschema_valid` mistakenly considers the Kubernetes Open API schema as invalid and therefore can't be compared with other libraries in this case.

You can find benchmark code in `benches/jsonschema.rs`, Rust version is `1.49`.

**NOTE**. This library is in early development.

## Support

If you have anything to discuss regarding this library, please, join our [gitter](https://gitter.im/Stranger6667/jsonschema-rs)!
