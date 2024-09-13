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
|---------------|----------------|---------------|--------------------------|
| OpenAPI       | - (1)          | 1477.92 ms (**x92.70**) | 15.94 ms     |
| Swagger       | - (1)          | 2586.88 ms (**x177.61**)| 14.56 ms     |
| Canada (GeoJSON) | 22.64 ms (**x5.03**)  | 1775.93 ms (**x394.76**) | 4.50 ms |
| CITM Catalog  | 10.16 ms (**x1.92**)  | 178.60 ms (**x33.73**) | 5.29 ms  |
| Fast (Valid)  | 3.73 µs (**x3.38**)   | 83.84 µs (**x75.94**) | 1.10 µs  |
| Fast (Invalid)| 4.24 µs (**x2.77**)   | 83.11 µs (**x54.18**) | 1.53 µs  |

### jsonschema-rs Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 15.94 ms   | 15.49 ms   | 1.03x   |
| Swagger       | 14.56 ms   | 14.42 ms   | 1.01x   |
| Canada (GeoJSON) | 4.50 ms | 4.46 ms    | 1.01x   |
| CITM Catalog  | 5.29 ms    | 3.01 ms    | 1.76x   |
| Fast (Valid)  | 1.10 µs    | 696.00 ns  | 1.59x   |
| Fast (Invalid)| 1.53 µs    | 1.08 µs    | 1.42x   |

Notes:

1. `fastjsonschema` fails to compile the Open API spec due to the presence of the `uri-reference` format (that is not defined in Draft 4). However, unknown formats are explicitly supported by the spec.

You can find benchmark code in [benches/](benches/), Python version `3.12.5`, Rust version `1.81`.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.
