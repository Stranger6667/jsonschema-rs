# jsonschema

[![ci](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![codecov](https://codecov.io/gh/Stranger6667/jsonschema-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Stranger6667/jsonschema-rs)
[![Crates.io](https://img.shields.io/crates/v/jsonschema.svg)](https://crates.io/crates/jsonschema)
[![docs.rs](https://docs.rs/jsonschema/badge.svg?version=0.2.0)](https://docs.rs/jsonschema/0.2.0/jsonschema/)

A JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

Supported drafts:
- Draft 7
- Draft 6
- Draft 4 (except optional `bignum.json` test case)

```toml
# Cargo.toml
jsonschema = "0.2"
```

To validate documents against some schema and get validation errors (if any):

```rust
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    let compiled = JSONSchema::compile(&schema, Some(Draft::Draft7));
    let result = compiled.validate(&instance);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error)
        }   
    }
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
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    // Draft is detected automatically with fallback to Draft7
    let compiled = JSONSchema::compile(&schema, None);
    assert!(compiled.is_valid(&instance));
}
```

## Performance

There is a comparison with other JSON Schema validators written in Rust - `jsonschema_valid` and `valico`.

Test machine i8700K (12 cores), 32GB RAM.

Performance of `jsonschema::JSONSchema.is_valid`. Ratios are given against compiled jsonschema:

- Big valid input (`canada_schema.json` and `canada.json`)
- Small valid input (`small_schema.json` and `small_valid.json`)
- Small invalid input (`small_schema.json` and `small_invalid.json`)

| Case          | jsonschema_valid       | valico                  | jsonschema (not compiled) | jsonschema (compiled) |
| ------------- | ---------------------- | ----------------------- | ------------------------- | --------------------- |
| Big valid     | 56.746 ms (**x187.2**) | 149.49 ms (**x493.17**) | 317.14 us (**x1.04**)     | 303.12 us             |
| Small valid   | 2.23 us   (**x14.92**) | 3.87 us   (**x25.9**)   | 3.76 us   (**x25.17**)    | 149.38 ns             |
| Small invalid | 515.22 ns (**x85.58**) | 4.08 us   (**x677.74**) | 3.63 us   (**x602.99**)   | 6.02 ns               |

As you can see the compiled version is faster, especially for large inputs. However, not-compiled version is slower
on smaller inputs than `jsonschema_valid`.

You can find benchmark code in `benches/jsonschema.rs`

**NOTE**. This library is in early development.