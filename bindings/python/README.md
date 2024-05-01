# jsonschema-rs

[![Build](https://github.com/Stranger6667/jsonschema-rs/workflows/ci/badge.svg)](https://github.com/Stranger6667/jsonschema-rs/actions)
[![Version](https://img.shields.io/pypi/v/jsonschema-rs.svg)](https://pypi.org/project/jsonschema-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/jsonschema-rs.svg)](https://pypi.org/project/jsonschema-rs/)
[![License](https://img.shields.io/pypi/l/jsonschema-rs.svg)](https://opensource.org/licenses/MIT)

Fast JSON Schema validation for Python implemented in Rust.

Supported drafts:

- Draft 7
- Draft 6
- Draft 4

There are some notable restrictions at the moment:

- The underlying Rust crate doesn't support arbitrary precision integers yet, which may lead to `SystemError` when such value is used;
- Unicode surrogates are not supported;

## Installation

To install `jsonschema-rs` via `pip` run the following command:

```bash
pip install jsonschema-rs
```

## Usage

To check if the input document is valid:

```python
import jsonschema_rs

validator = jsonschema_rs.JSONSchema({"minimum": 42})
validator.is_valid(45)  # True
```

or:

```python
import jsonschema_rs

validator = jsonschema_rs.JSONSchema({"minimum": 42})
validator.validate(41)  # raises ValidationError
```

If you have a schema as a JSON string, then you could use
`jsonschema_rs.JSONSchema.from_str` to avoid parsing on the
Python side:

```python
import jsonschema_rs

validator = jsonschema_rs.JSONSchema.from_str('{"minimum": 42}')
...
```

You can define custom format checkers:

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

According to our benchmarks, `jsonschema-rs` is usually faster than
existing alternatives in real-life scenarios.

However, for small schemas & inputs it might be slower than
`fastjsonschema` or `jsonschema` on PyPy.

### Input values and schemas

- [Zuora](https://github.com/APIs-guru/openapi-directory/blob/master/APIs/zuora.com/2021-04-23/openapi.yaml) OpenAPI schema (`zuora.json`). Validated against [OpenAPI 3.0 JSON Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v3.0/schema.json) (`openapi.json`).
- [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml) Swagger schema (`kubernetes.json`). Validated against [Swagger JSON Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json) (`swagger.json`).
- Canadian border in GeoJSON format (`canada.json`). Schema is taken from the [GeoJSON website](https://geojson.org/schema/FeatureCollection.json) (`geojson.json`).
- Concert data catalog (`citm_catalog.json`). Schema is inferred via [infers-jsonschema](https://github.com/Stranger6667/infers-jsonschema) & manually adjusted (`citm_catalog_schema.json`).
- `Fast` is taken from [fastjsonschema benchmarks](https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15) (`fast_schema.json`, `fast_valid.json` and `fast_invalid.json`).

| Case             | Schema size   | Instance size   |
| ---------------- | ------------- | --------------- |
| OpenAPI          |  18 KB        |  4.5 MB         |
| Swagger          |  25 KB        |  3.0 MB         |
| Canada           |  4.8 KB       |  2.1 MB         |
| CITM catalog     |  2.3 KB       |  501 KB         |
| Fast (valid)     |  595 B        |  55 B           |
| Fast (invalid)   |  595 B        |  60 B           |

Compiled validators (when the input schema is compiled once and reused
later). `jsonschema-rs` comes in three variants in the tables below:

- `validate`. This method raises `ValidationError` on errors or returns `None` on their absence.
- `is_valid`. A faster method that returns a boolean result whether the instance is valid.
- `overhead`. Only transforms data to underlying Rust types and do not perform any validation. Shows the Python -> Rust data conversion cost.

Ratios are given against the `validate` variant.

Small schemas:

| library                   | `true`                | `{"minimum": 10}`      | `Fast (valid)`         | `Fast (invalid)`       |
|---------------------------|-----------------------|------------------------|------------------------|------------------------|
| jsonschema-rs\[validate\] | 93.84 ns             | 94.83 ns               | 1.2 us                 | 1.84 us                |
| jsonschema-rs\[is_valid\] | 70.22 ns (**x0.74**) | 68.26 ns (**x0.71**)   | 688.70 ns (**x0.57**)  | 1.26 us (**x0.68**)    |
| jsonschema-rs\[overhead\] | 65.27 ns (**x0.69**) | 66.90 ns (**x0.70**)   | 461.53 ns (**x0.38**)  | 925.16 ns (**x0.50**)  |
| fastjsonschema\[CPython\] | 58.19 ns (**x0.62**) | 105.77 ns (**x1.11**)  | 3.98 us (**x3.31**)    | 4.57 us (**x2.48**)    |
| fastjsonschema\[PyPy\]    | 10.39 ns (**x0.11**) | 34.96 ns (**x0.36**)   | 866 ns (**x0.72**)     | 916 ns (**x0.49**)     |
| jsonschema\[CPython\]     | 235.06 ns (**x2.50**)| 1.86 us (**x19.6**)    | 56.26 us (**x46.88**)  | 59.39 us (**x32.27**)  |
| jsonschema\[PyPy\]        | 40.83 ns (**x0.43**) | 232.41 ns (**x2.45**)  | 21.82 us (**x18.18**)  | 22.23 us (**x12.08**)  |

Large schemas:

| library                   | `Zuora (OpenAPI)`      | `Kubernetes (Swagger)` | `Canada (GeoJSON)`     | `CITM catalog`         |
|---------------------------|------------------------|------------------------|------------------------|------------------------|
| jsonschema-rs\[validate\] | 17.311 ms              | 15.194 ms              | 5.018 ms               | 4.765 ms               |
| jsonschema-rs\[is_valid\] | 16.605 ms (**x0.95**)  | 12.610 ms (**x0.82**)  | 4.954 ms (**x0.98**)   | 2.792 ms (**x0.58**)   |
| jsonschema-rs\[overhead\] | 12.017 ms (**x0.69**)  | 8.005 ms (**x0.52**)   | 3.702 ms (**x0.73**)   | 2.303 ms (**x0.48**)   |
| fastjsonschema\[CPython\] | -- (1)                 | 90.305 ms (**x5.94**)  | 32.389 ms (**6.45**)   | 12.020 ms (**x2.52**)  |
| fastjsonschema\[PyPy\]    | -- (1)                 | 37.204 ms (**x2.44**)  | 8.450 ms (**x1.68**)   | 4.888 ms (**x1.02**)   |
| jsonschema\[CPython\]     | 764.172 ms (**x44.14**)| 1.063 s (**x69.96**)   | 1.301 s (**x259.26**)  | 115.362 ms (**x24.21**)|
| jsonschema\[PyPy\]        | 604.557 ms (**x34.92**)| 619.744 ms (**x40.78**)| 524.275 ms (**x104.47**)| 25.275 ms (**x5.30**) |

Notes:

1. `fastjsonschema` fails to compile the Open API spec due to the presence of the `uri-reference` format (that is not defined in Draft 4). However, unknown formats are [explicitly supported](https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-7.1) by the spec.

The bigger the input is the bigger is performance win. You can take a look at benchmarks in `benches/bench.py`.

Package versions:

- `jsonschema-rs` - latest version from the repository
- `jsonschema` - `3.2.0`
- `fastjsonschema` - `2.15.1`

Measured with stable Rust 1.56, CPython 3.9.7 / PyPy3 7.3.6 on Intel i8700K

## Python support

`jsonschema-rs` supports CPython 3.7, 3.8, 3.9, 3.10, 3.11, and 3.12.

## License

The code in this project is licensed under [MIT license](https://opensource.org/licenses/MIT). By contributing to `jsonschema-rs`, you agree that your contributions will be licensed under its MIT license.
