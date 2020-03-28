# jsonschema

Yet another JSON Schema validator implementation. It compiles schema into a validation tree to have validation as fast as possible.

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

If you only need to know whether document is valid or not:

```rust
use jsonschema::is_valid;
use serde_json::json;

fn main() {
    let schema = json!({"maxLength": 5});
    let instance = json!("foo");
    assert!(is_valid(&schema, &instance));
}
```  

**NOTE**. This library is in early development.