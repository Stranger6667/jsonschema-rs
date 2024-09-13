# jsonschema-rs

[![Build](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![Version](https://img.shields.io/pypi/v/jsonschema-rs.svg)](https://pypi.org/project/jsonschema-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/jsonschema-rs.svg)](https://pypi.org/project/jsonschema-rs/)
[![License](https://img.shields.io/pypi/l/jsonschema-rs.svg)](https://opensource.org/licenses/MIT)
![Supported Dialects](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fsupported_versions.json)

A high-performance JSON Schema validator for Python.

```python
import jsonschema_rs

validator = jsonschema_rs.JSONSchema({"minimum": 42})

# Boolean result
validator.is_valid(45)

# Raise a ValidationError
validator.validate(41)
# ValidationError: 41 is less than the minimum of 42
#
# Failed validating "minimum" in schema
#
# On instance:
#    41

# Iterate over all validation errors
for error in validator.iter_errors(40):
    print(f"Error: {error}")
```

## Highlights

- 📚 Support for popular JSON Schema drafts
- 🌐 Remote reference fetching (network/file)
- 🔧 Custom format validators

### Supported drafts

Compliance levels vary across drafts, with newer versions having some unimplemented keywords.

- ![Draft 2020-12](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2020-12.json)
- ![Draft 2019-09](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2019-09.json)
- ![Draft 7](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft7.json)
- ![Draft 6](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft6.json)
- ![Draft 4](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft4.json)

You can check the current status on the [Bowtie Report](https://bowtie.report/#/implementations/rust-jsonschema).

## Limitations

- No support for arbitrary precision numbers

## Installation

To install `jsonschema-rs` via `pip` run the following command:

```bash
pip install jsonschema-rs
```

## Usage

If you have a schema as a JSON string, then you could use
`jsonschema_rs.JSONSchema.from_str` to avoid parsing on the
Python side:

```python
validator = jsonschema_rs.JSONSchema.from_str('{"minimum": 42}')
...
```

You can specify a custom JSON Schema draft using the `draft` argument:

```python
import jsonschema_rs

validator = jsonschema_rs.JSONSchema(
    {"minimum": 42}, 
    draft=jsonschema_rs.Draft7
)
```

JSON Schema allows for format validation through the `format` keyword. While `jsonschema-rs`
provides built-in validators for standard formats, you can also define custom format validators
for domain-specific string formats.

To implement a custom format validator:

1. Define a function that takes a `str` and returns a `bool`.
2. Pass it with the `formats` argument.

```python
import jsonschema_rs

def is_currency(value):
    # The input value is always a string
    return len(value) == 3 and value.isascii()


validator = jsonschema_rs.JSONSchema(
    {"type": "string", "format": "currency"}, 
    formats={"currency": is_currency}
)
validator.is_valid("USD")  # True
validator.is_valid("invalid")  # False
```

## Performance

`jsonschema-rs` is designed for high performance, outperforming other Python JSON Schema validators in most scenarios:

- Up to **30-390x** faster than `jsonschema` for complex schemas and large instances
- Generally 2-5x faster than `fastjsonschema` on CPython
- Comparable or slightly slower performance for very small schemas

For detailed benchmarks, see our [full performance comparison](BENCHMARKS.md).

## Python support

`jsonschema-rs` supports CPython 3.8, 3.9, 3.10, 3.11, and 3.12.

## Support

If you have questions, need help, or want to suggest improvements, please use [GitHub Discussions](https://github.com/Stranger6667/jsonschema-rs/discussions).

## Sponsorship

If you find `jsonschema-rs` useful, please consider [sponsoring its development](https://github.com/sponsors/Stranger6667).

## Contributing

We welcome contributions! Here's how you can help:

- Share your use cases
- Implement missing keywords
- Fix failing test cases from the [JSON Schema test suite](https://bowtie.report/#/implementations/rust-jsonschema)

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for more details.

## License

Licensed under [MIT License](LICENSE).

