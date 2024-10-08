# jsonschema-cli

[<img alt="crates.io" src="https://img.shields.io/crates/v/jsonschema-cli.svg?style=flat-square&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/jsonschema-cli)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-jsonschema-cli?style=flat-square&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/jsonschema-cli)

A fast command-line tool for JSON Schema validation, powered by the `jsonschema` crate.

## Installation

```
cargo install jsonschema-cli
```

## Usage

```
jsonschema [OPTIONS] <SCHEMA>
```

**NOTE**: It only supports valid JSON as input.

### Options:

- `-i, --instance <FILE>`: JSON instance(s) to validate (can be used multiple times)
- `-v, --version`: Show version information
- `--help`: Display help information

### Examples:

Validate a single instance:
```
jsonschema schema.json -i instance.json
```

Validate multiple instances:
```
jsonschema schema.json -i instance1.json -i instance2.json
```

## Features

- Validate one or more JSON instances against a single schema
- Clear, concise output with detailed error reporting
- Fast validation using the `jsonschema` Rust crate

## Output

For each instance, the tool will output:

- `<filename> - VALID` if the instance is valid
- `<filename> - INVALID` followed by a list of errors if invalid

Example output:
```
instance1.json - VALID
instance2.json - INVALID. Errors:
1. "name" is a required property
2. "age" must be a number
```

## Exit Codes

- 0: All instances are valid (or no instances provided)
- 1: One or more instances are invalid, or there was an error

## License

This project is licensed under the MIT License.
