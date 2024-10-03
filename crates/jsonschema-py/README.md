# jsonschema-rs

[![Build](https://img.shields.io/github/actions/workflow/status/Stranger6667/jsonschema-rs/ci.yml?branch=master&style=flat-square)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![Version](https://img.shields.io/pypi/v/jsonschema-rs.svg?style=flat-square)](https://pypi.org/project/jsonschema-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/jsonschema-rs.svg?style=flat-square)](https://pypi.org/project/jsonschema-rs/)
[![License](https://img.shields.io/pypi/l/jsonschema-rs.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[<img alt="Supported Dialects" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fsupported_versions.json&style=flat-square">](https://bowtie.report/#/implementations/rust-jsonschema)

A high-performance JSON Schema validator for Python.

```python
import jsonschema_rs

schema = {"maxLength": 5}
instance = "foo"

# One-off validation
try:
    jsonschema_rs.validate(schema, "incorrect")
except jsonschema_rs.ValidationError as exc:
    assert str(exc) == '''"incorrect" is longer than 5 characters

Failed validating "maxLength" in schema

On instance:
    "incorrect"'''

# Build & reuse (faster)
validator = jsonschema_rs.validator_for(schema)

# Iterate over errors
for error in validator.iter_errors(instance):
    print(f"Error: {error}")
    print(f"Location: {error.instance_path}")

# Boolean result
assert validator.is_valid(instance)
```

> ⚠️ **Upgrading from pre-0.20.0?** Check our [Migration Guide](https://github.com/Stranger6667/jsonschema-rs/blob/master/crates/jsonschema-py/MIGRATION.md) for key changes.

## Highlights

- 📚 Support for popular JSON Schema drafts
- 🌐 Remote reference fetching (network/file)
- 🔧 Custom format validators

### Supported drafts

Compliance levels vary across drafts, with newer versions having some unimplemented keywords.

- [![Draft 2020-12](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2020-12.json)](https://bowtie.report/#/implementations/rust-jsonschema)
- [![Draft 2019-09](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2019-09.json)](https://bowtie.report/#/implementations/rust-jsonschema)
- [![Draft 7](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft7.json)](https://bowtie.report/#/implementations/rust-jsonschema)
- [![Draft 6](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft6.json)](https://bowtie.report/#/implementations/rust-jsonschema)
- [![Draft 4](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft4.json)](https://bowtie.report/#/implementations/rust-jsonschema)

You can check the current status on the [Bowtie Report](https://bowtie.report/#/implementations/rust-jsonschema).

## Limitations

- No support for arbitrary precision numbers

## Installation

To install `jsonschema-rs` via `pip` run the following command:

```bash
pip install jsonschema-rs
```

## Usage

If you have a schema as a JSON string, then you could pass it to `validator_for`
to avoid parsing on the Python side:

```python
validator = jsonschema_rs.validator_for('{"minimum": 42}')
...
```

You can use draft-specific validators for different JSON Schema versions:

```python
import jsonschema_rs

# Automatic draft detection
validator = jsonschema_rs.validator_for({"minimum": 42})

# Draft-specific validators
validator = jsonschema_rs.Draft7Validator({"minimum": 42})
validator = jsonschema_rs.Draft201909Validator({"minimum": 42})
validator = jsonschema_rs.Draft202012Validator({"minimum": 42})
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


validator = jsonschema_rs.validator_for(
    {"type": "string", "format": "currency"}, 
    formats={"currency": is_currency}
)
validator.is_valid("USD")  # True
validator.is_valid("invalid")  # False
```

## Performance

`jsonschema-rs` is designed for high performance, outperforming other Python JSON Schema validators in most scenarios:

- Up to **30-390x** faster than `jsonschema` for complex schemas and large instances
- Generally **2-5x** faster than `fastjsonschema` on CPython
- Comparable or slightly slower performance for very small schemas

For detailed benchmarks, see our [full performance comparison](https://github.com/Stranger6667/jsonschema-rs/blob/master/crates/jsonschema-py/BENCHMARKS.md).

## Python support

`jsonschema-rs` supports CPython 3.8, 3.9, 3.10, 3.11, and 3.12.

## Acknowledgements

This library draws API design inspiration from the Python [`jsonschema`](https://github.com/python-jsonschema/jsonschema) package. We're grateful to the Python `jsonschema` maintainers and contributors for their pioneering work in JSON Schema validation.

## Support

If you have questions, need help, or want to suggest improvements, please use [GitHub Discussions](https://github.com/Stranger6667/jsonschema-rs/discussions).

## Sponsorship

If you find `jsonschema-rs` useful, please consider [sponsoring its development](https://github.com/sponsors/Stranger6667).

## Contributing

We welcome contributions! Here's how you can help:

- Share your use cases
- Implement missing keywords
- Fix failing test cases from the [JSON Schema test suite](https://bowtie.report/#/implementations/rust-jsonschema)

See [CONTRIBUTING.md](https://github.com/Stranger6667/jsonschema-rs/blob/master/CONTRIBUTING.md) for more details.

## License

Licensed under [MIT License](https://github.com/Stranger6667/jsonschema-rs/blob/master/LICENSE).

