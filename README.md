# jsonschema

[![ci](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![codecov](https://codecov.io/gh/Stranger6667/jsonschema-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Stranger6667/jsonschema-rs)
[![Crates.io](https://img.shields.io/crates/v/jsonschema.svg)](https://crates.io/crates/jsonschema)
[![docs.rs](https://docs.rs/jsonschema/badge.svg?version=0.4.3)](https://docs.rs/jsonschema/0.4.3/jsonschema/)
[![gitter](https://img.shields.io/gitter/room/Stranger6667/jsonschema-rs.svg)](https://gitter.im/Stranger6667/jsonschema-rs)

A JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

Supported drafts:

- Draft 7 (except optional `idn-hostname.json` test cases)
- Draft 6
- Draft 4 (except optional `bignum.json` test cases)

```toml
# Cargo.toml
jsonschema = "0.4"
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

There is a comparison with other JSON Schema validators written in Rust - `jsonschema_valid` and `valico`.

Test machine i8700K (12 cores), 32GB RAM.

Performance of `jsonschema::JSONSchema.is_valid`. Ratios are given against compiled jsonschema:

- Big valid input (`canada_schema.json` and `canada.json`)
- Small valid input (`small_schema.json` and `small_valid.json`)
- Small invalid input (`small_schema.json` and `small_invalid.json`)

| Case          | jsonschema_valid        | valico                  | jsonschema   |
| ------------- | ----------------------- | ----------------------- | ------------ |
| Big valid     | 56.746 ms (**x185.65**) | 149.49 ms (**x489.07**) | 305.66 us    |
| Small valid   | 2.23 us   (**x17.15**)  | 3.87 us   (**x29.77**)  | 129.97 ns    |
| Small invalid | 515.22 ns (**x96.3**)   | 4.08 us   (**x762.61**) | 5.35 ns      |

All libraries were used in their "compiled" form, where a validator is prepared before usage. Here is comparison when
a validator is compiled every time.

| Case          | jsonschema_valid        | valico                  | jsonschema  |
| ------------- | ----------------------- | ----------------------- | ----------- |
| Big valid     | 56.714 ms (**x183.72**) | 146.82 ms (**x475.62**) | 308.69 us   |
| Small valid   | 3.02 us   (**x1.13**)   | 118.09 us (**x44.22**)  | 2.67 us     |
| Small invalid | 1.17 us   (**x0.46**)   | 81.95 us  (**x32.26**)  | 2.54 us     |

You can find benchmark code in `benches/jsonschema.rs`, Rust version is `1.44`

**NOTE**. This library is in early development.

## Support

If you have anything to discuss regarding this library, please, join our [gitter](https://gitter.im/Stranger6667/jsonschema-rs)!
