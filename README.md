# jsonschema

Yet another JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

```rust
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    let compiled = JSONSchema::compile(&schema, Some(Draft::Draft7));
    let result = compiled.validate(&instance);
}
``` 

**NOTE**. This library is in early development.