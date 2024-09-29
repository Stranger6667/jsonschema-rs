# Benchmark Suite

A benchmarking suite for comparing different Rust JSON Schema implementations.

## Implementations

- `jsonschema` (latest version in this repo)
- [valico](https://crates.io/crates/valico) (v4.0.0)
- [jsonschema-valid](https://crates.io/crates/jsonschema-valid) (v0.5.2)
- [boon](https://crates.io/crates/boon) (v0.6.0)

## Usage

To run the benchmarks:

```console
$ cargo bench
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

| Benchmark     | jsonschema_valid | valico        | boon          | jsonschema (validate) |
|---------------|------------------|---------------|---------------|------------------------|
| OpenAPI       | -                | -             | 12.23 ms (**x2.51**) | 4.88 ms              |
| Swagger       | -                | 201.98 ms  (**x30.60**)   | 18.24 ms (**x2.76**)     | 6.60 ms              |
| GeoJSON       | 35.75 ms   (**x29.79**)      | 559.52 ms  (**x466.27**)   | 29.01 ms (**x24.18**)  | 1.20 ms              |
| CITM Catalog  | 5.51 ms  (**x2.16**)        | 46.31 ms  (**x18.16**)    | 2.07 ms  (**x0.81**)     | 2.55 ms              |
| Fast (Valid)  | 2.10 µs     (**x3.94**)     | 6.61 µs  (**x12.40**)     | 597.00 ns  (**x1.12**)   | 533.21 ns            |
| Fast (Invalid)| 368.53 ns     (**x0.51**)   | 6.77 µs  (**x9.41**)     | 748.22 ns (**x1.04**)    | 719.71 ns            |

### jsonschema Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 4.88 ms  | 4.43 ms  | **1.10x**   |
| Swagger       | 6.60 ms  | 4.87 ms  | **1.36x**   |
| GeoJSON       | 1.20 ms  | 1.19 ms  | **1.01x**   |
| CITM Catalog  | 2.55 ms  | 657.51 µs  | **3.88x**   |
| Fast (Valid)  | 533.21 ns  | 99.35 ns  | **5.37x**   |
| Fast (Invalid)| 719.71 ns  | 5.7769 ns  | **124.58x**  |

Notes:

1. `jsonschema_valid` and `valico` do not handle valid path instances matching the `^\\/` regex.

2. `jsonschema_valid` fails to resolve local references (e.g. `#/definitions/definitions`).

You can find benchmark code in [benches/](benches/), Rust version is `1.81`.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.

