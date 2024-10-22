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
- OpenAPI: [Zuora](https://github.com/APIs-guru/openapi-directory/blob/1afd351ddf50e050acdb52937a819ef1927f417a/APIs/zuora.com/2021-04-23/openapi.yaml), [Schema](https://spec.openapis.org/oas/3.0/schema/2021-09-28)
- Swagger: [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml), [Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json)
- GeoJSON: [Schema](https://geojson.org/schema/FeatureCollection.json)
- CITM: Schema inferred via [infers-jsonschema](https://github.com/Stranger6667/infers-jsonschema)
- Fast: [fastjsonschema benchmarks](https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15)

## Results

### Comparison with Other Libraries

| Benchmark     | fastjsonschema | jsonschema    | jsonschema-rs |
|---------------|----------------|---------------|----------------|
| OpenAPI       | - (1)          | 644.21 ms (**x83.84**) | 7.68 ms     |
| Swagger       | - (1)          | 1141.58 ms (**x182.70**)| 6.25 ms     |
| Canada (GeoJSON) | 10.52 ms (**x4.07**)  | 780.44 ms (**x301.87**) | 2.59 ms |
| CITM Catalog  | 4.94 ms (**x2.04**)   | 82.37 ms (**x34.03**) | 2.42 ms  |
| Fast (Valid)  | 1.97 µs (**x3.39**)   | 37.66 µs (**x64.82**) | 581.00 ns  |
| Fast (Invalid)| 2.28 µs (**x2.78**)   | 36.45 µs (**x44.40**) | 821.00 ns  |

### jsonschema-rs Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 7.68 ms    | 6.91 ms    | **1.11x**   |
| Swagger       | 6.25 ms    | 5.24 ms    | **1.19x**   |
| Canada (GeoJSON) | 2.59 ms | 2.59 ms    | **1.00x**   |
| CITM Catalog  | 2.42 ms    | 1.36 ms    | **1.77x**   |
| Fast (Valid)  | 581.00 ns  | 350.00 ns  | **1.66x**   |
| Fast (Invalid)| 821.00 ns  | 591.00 ns  | **1.39x**   |

Notes:

1. `fastjsonschema` fails to compile the Open API spec due to the presence of the `uri-reference` format (that is not defined in Draft 4). However, unknown formats are explicitly supported by the spec.

You can find benchmark code in [benches/](benches/), Python version `3.13.0`, Rust version `1.81`.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.
