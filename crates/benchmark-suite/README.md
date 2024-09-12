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
| OpenAPI       | -                | -             | 12.23 ms (**x2.25**) | 5.43 ms              |
| Swagger       | -                | 201.98 ms  (**x28.98**)   | 18.24 ms (**x2.62**)     | 6.97 ms              |
| GeoJSON       | 35.75 ms   (**x30.56**)      | 559.52 ms  (**x478.22**)   | 29.01 ms (**x24.79**)  | 1.17 ms              |
| CITM Catalog  | 5.51 ms  (**x2.45**)        | 46.31 ms  (**x20.58**)    | 2.07 ms  (**x0.92**)     | 2.25 ms              |
| Fast (Valid)  | 2.10 µs     (**x4.27**)     | 6.61 µs  (**x13.44**)     | 597.00 ns  (**x1.21**)   | 491.82 ns            |
| Fast (Invalid)| 368.53 ns     (**x0.56**)   | 6.77 µs  (**x10.22**)     | 748.22 ns (**x1.13**)    | 662.52 ns            |

### jsonschema Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 5.43 ms  | 4.70 ms  | 1.16x   |
| Swagger       | 6.97 ms  | 5.35 ms  | 1.30x   |
| GeoJSON       | 1.17 ms  | 1.16 ms  | 1.00x   |
| CITM Catalog  | 2.2510 ms  | 630.82 µs  | 3.57x   |
| Fast (Valid)  | 491.82 ns  | 111.09 ns  | 4.43x   |
| Fast (Invalid)| 662.52 ns  | 7.1289 ns  | 92.93x  |

Notes:

1. `jsonschema_valid` and `valico` do not handle valid path instances matching the `^\\/` regex.

2. `jsonschema_valid` fails to resolve local references (e.g. `#/definitions/definitions`).

You can find benchmark code in [benches/](benches/), Rust version is `1.81`.

## Purpose

The `benchmark-suite` crate provides a standardized way to measure and compare the performance of various JSON Schema validation libraries in Rust. It helps in identifying performance bottlenecks and guiding optimization efforts for the `jsonschema` crate.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.

