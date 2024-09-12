# JSON Schema Test Suite

A Rust proc macro for generating test suites from the official JSON Schema Test Suite.

## Features

- Automatically generates tests for multiple JSON Schema drafts
- Supports selective draft inclusion
- Allows marking specific tests as expected failures

## Usage

```rust
use jsonschema_testsuite::{suite, Test};

#[suite(
    path = "path/to/test/suite",
    drafts = ["draft7", "draft2020-12"],
    xfail = ["draft7::some::failing::test"]
)]
fn test_suite(test: Test) {
    // Your test implementation here
}
```

Where `Test` is the following struct:

```rust
#[derive(Debug)]
pub struct Test {
    pub draft: &'static str,
    pub schema: Value,
    /// Test case description.
    pub case: &'static str,
    /// The test description, briefly explaining which behavior it exercises.
    pub description: &'static str,
    /// The instance which should be validated against the schema in schema.
    pub data: Value,
    /// Whether the validation process of this instance should consider the instance valid or not.
    pub valid: bool,
}
```

## License

This project is licensed under the MIT License.

