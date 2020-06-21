# Changelog

## [Unreleased]

### Changed

- Enable Link-Time Optimizations and set `codegen-units` to 1. [#104](https://github.com/Stranger6667/jsonschema-rs/issues/104)
- fix: `items` allows the presence of boolean schemas. [#115](https://github.com/Stranger6667/jsonschema-rs/pull/115)

## [0.3.0] - 2020-06-08

### Added

- JSONSchema Draft 4 support (except one optional case). [#34](https://github.com/Stranger6667/jsonschema-rs/pull/34)
- CI builds. [#35](https://github.com/Stranger6667/jsonschema-rs/pull/35) and [#36](https://github.com/Stranger6667/jsonschema-rs/pull/36)
- Implement specialized `is_valid` methods for all keywords.
- Use `rayon` in `items` keyword validation.
- Various `clippy` lints. [#66](https://github.com/Stranger6667/jsonschema-rs/pull/66)
- `Debug` implementation for `JSONSchema` and  `Resolver`. [#97](https://github.com/Stranger6667/jsonschema-rs/pull/97)
- `Default` implementation for `Draft`.

### Changed

- Do not pin dependencies. [#90](https://github.com/Stranger6667/jsonschema-rs/pull/90)
- Use `to_string` instead of `format!`. [#85](https://github.com/Stranger6667/jsonschema-rs/pull/85)
- Cache compiled validators in `$ref` keyword. [#83](https://github.com/Stranger6667/jsonschema-rs/pull/83)
- Use bitmap for validation of multiple types in `type` keyword implementation. [#78](https://github.com/Stranger6667/jsonschema-rs/pull/78)
- Return errors instead of unwrap in various locations. [#73](https://github.com/Stranger6667/jsonschema-rs/pull/73)
- Improve debug representation of validators. [#70](https://github.com/Stranger6667/jsonschema-rs/pull/70)
- Reduce the number of `match` statements during compilation functions resolving.
- Use `expect` instead of `unwrap` for known cases when it is known that the code won't panic.
- Add specialized validators for all `format` cases.
- Reuse `DEFAULT_SCOPE` during reference resolving.
- Replace some `Value::as_*` calls with `if let`.
- Inline all `compile` functions.
- Optimize `format` keyword compilation by using static strings.
- Optimize compilation of `true`, `false` and `$ref` validators.
- Reuse parsed `DEFAULT_ROOT_URL` in `JSONSchema::compile`.
- Avoid string allocation during `scope` parsing in `JSONSchema::compile`.
- Refactor benchmark suite
- Use `BTreeSet` in `additionalProperties` keyword during compilation to reduce the amount of copied data. [#91](https://github.com/Stranger6667/jsonschema-rs/pull/91)

### Fixed

- Wrong implementation of `is_valid` for `additionalProperties: false` keyword case. [#61](https://github.com/Stranger6667/jsonschema-rs/pull/61)
- Possible panic due to type conversion in some numeric validators. [#72](https://github.com/Stranger6667/jsonschema-rs/pull/72)
- Precision loss in `minimum`, `maximum`, `exclusiveMinimum` and `exclusiveMaximum` validators. [#84](https://github.com/Stranger6667/jsonschema-rs/issues/84)

## [0.2.0] - 2020-03-30

### Added

- Implement `is_valid` for various validators.
- Implement `Error` and `Display` for `CompilationError`

### Changed

- Debug representation & error messages in various validators.
- Make `ErrorIterator` `Sync` and `Send`.

### Fixed

- Return `CompilationError` on invalid input schemas instead of panic.

## 0.1.0 - 2020-03-29

- Initial public release

[Unreleased]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.1.0...v0.2.0
