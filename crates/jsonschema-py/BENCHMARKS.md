# Benchmark Suite

A benchmarking suite for comparing different Python JSON Schema implementations.

## Implementations

- `jsonschema-rs` (latest version in this repo)
- [jsonschema](https://pypi.org/project/jsonschema/) (v4.23.0)
- [fastjsonschema](https://pypi.org/project/fastjsonschema/) (v2.20.0)

## Usage

Install the dependencies:

```console
$ pip install -e ".[bench]"
```

Run the benchmarks:

```console
$ pytest benches/bench.py
```

## Overview

| Benchmark     | Description                                    | Schema Size | Instance Size |
|----------|------------------------------------------------|-------------|---------------|
| OpenAPI  | Zuora API validated against OpenAPI 3.0 schema | 18 KB       | 4.5 MB        |
| Swagger  | Kubernetes API (v1.10.0) with Swagger schema   | 25 KB       | 3.0 MB        |
| GeoJSON  | Canadian border in GeoJSON format              | 4.8 KB      | 2.1 MB        |
| CITM     | Concert data catalog with inferred schema      | 2.3 KB      | 501 KB        |
| Fast     | From fastjsonschema benchmarks (valid/invalid) | 595 B       | 55 B / 60 B   |

Sources:
- OpenAPI: [Zuora](https://github.com/APIs-guru/openapi-directory/blob/master/APIs/zuora.com/2021-04-23/openapi.yaml), [Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v3.0/schema.json)
- Swagger: [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml), [Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json)
- GeoJSON: [Schema](https://geojson.org/schema/FeatureCollection.json)
- CITM: Schema inferred via [infers-jsonschema](https://github.com/Stranger6667/infers-jsonschema)
- Fast: [fastjsonschema benchmarks](https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15)

## Results

### Comparison with Other Libraries

| Benchmark     | fastjsonschema | jsonschema    | jsonschema-rs |
|---------------|----------------|---------------|----------------|
| OpenAPI       | - (1)          | 1516.65 ms (**x80.51**) | 18.84 ms     |
| Swagger       | - (1)          | 2627.74 ms (**x162.59**)| 16.16 ms     |
| Canada (GeoJSON) | 23.21 ms (**x5.15**)  | 1771.93 ms (**x393.52**) | 4.50 ms |
| CITM Catalog  | 9.90 ms (**x1.95**)   | 176.04 ms (**x34.61**) | 5.09 ms  |
| Fast (Valid)  | 3.79 µs (**x3.07**)   | 84.67 µs (**x68.56**) | 1.23 µs  |
| Fast (Invalid)| 4.25 µs (**x2.51**)   | 84.49 µs (**x50.02**) | 1.69 µs  |

### jsonschema-rs Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 18.84 ms   | 17.31 ms   | 1.09x   |
| Swagger       | 16.16 ms   | 14.18 ms   | 1.14x   |
| Canada (GeoJSON) | 4.50 ms | 4.51 ms    | 1.00x   |
| CITM Catalog  | 5.09 ms    | 3.03 ms    | 1.68x   |
| Fast (Valid)  | 1.23 µs    | 714.00 ns  | 1.73x   |
| Fast (Invalid)| 1.69 µs    | 1.14 µs    | 1.48x   |

Notes:

1. `fastjsonschema` fails to compile the Open API spec due to the presence of the `uri-reference` format (that is not defined in Draft 4). However, unknown formats are explicitly supported by the spec.

You can find benchmark code in [benches/](benches/), Python version `3.12.5`, Rust version `1.81`.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.
