# jsonschema-cli

A fast and user-friendly command-line tool for JSON Schema validation, powered by the high-performance `jsonschema` Rust crate.

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
