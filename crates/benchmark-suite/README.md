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
- OpenAPI: [Zuora](https://github.com/APIs-guru/openapi-directory/blob/1afd351ddf50e050acdb52937a819ef1927f417a/APIs/zuora.com/2021-04-23/openapi.yaml), [Schema](https://spec.openapis.org/oas/3.0/schema/2021-09-28)
- Swagger: [Kubernetes](https://raw.githubusercontent.com/APIs-guru/openapi-directory/master/APIs/kubernetes.io/v1.10.0/swagger.yaml), [Schema](https://github.com/OAI/OpenAPI-Specification/blob/main/schemas/v2.0/schema.json)
- GeoJSON: [Schema](https://geojson.org/schema/FeatureCollection.json)
- CITM: Schema inferred via [infers-jsonschema](https://github.com/Stranger6667/infers-jsonschema)
- Fast: [fastjsonschema benchmarks](https://github.com/horejsek/python-fastjsonschema/blob/master/performance.py#L15)

## Results

### Comparison with Other Libraries

| Benchmark     | jsonschema_valid | valico        | boon          | jsonschema (validate) |
|---------------|------------------|---------------|---------------|------------------------|
| OpenAPI       | -                | -             | 6.60 ms (**x2.97**) | 2.22 ms              |
| Swagger       | -                | 114.26 ms (**x38.93**)   | 10.06 ms (**x3.43**)     | 2.94 ms              |
| GeoJSON       | 19.56 ms (**x23.85**)      | 299.53 ms (**x365.22**)   | 16.59 ms (**x20.23**)  | 820.21 µs            |
| CITM Catalog  | 2.84 ms (**x2.22**)        | 28.30 ms (**x22.11**)    | 1.11 ms (**x0.87**)     | 1.28 ms              |
| Fast (Valid)  | 1.11 µs (**x4.37**)        | 3.78 µs (**x14.89**)     | 332.39 ns (**x1.31**)   | 253.77 ns            |
| Fast (Invalid)| 247.88 ns (**x0.64**)      | 3.82 µs (**x9.85**)      | 383.79 ns (**x0.99**)   | 387.78 ns            |

### jsonschema Performance: `validate` vs `is_valid`

| Benchmark     | validate   | is_valid   | Speedup |
|---------------|------------|------------|---------|
| OpenAPI       | 2.2104 ms  | 2.0860 ms  | **1.06x**   |
| Swagger       | 2.9327 ms  | 1.9649 ms  | **1.49x**   |
| GeoJSON       | 820.21 µs  | 803.42 µs  | **1.02x**   |
| CITM Catalog  | 1.2794 ms  | 309.66 µs  | **4.13x**   |
| Fast (Valid)  | 253.77 ns  | 54.162 ns  | **4.69x**   |
| Fast (Invalid)| 387.78 ns  | 3.4379 ns  | **112.80x**  |

Notes:

1. `jsonschema_valid` and `valico` do not handle valid path instances matching the `^\\/` regex.

2. `jsonschema_valid` fails to resolve local references (e.g. `#/definitions/definitions`).

You can find benchmark code in [benches/](benches/), Rust version is `1.81`.

## Contributing

Contributions to improve, expand, or optimize the benchmark suite are welcome. This includes adding new benchmarks, ensuring fair representation of real-world use cases, and optimizing the configuration and usage of benchmarked libraries. Such efforts are highly appreciated as they ensure accurate and meaningful performance comparisons.

