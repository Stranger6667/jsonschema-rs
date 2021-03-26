# Changelog

## [Unreleased]

## [0.6.1] - 2021-03-26

### Fixed

- Incorrect handling of `\w` and `\W` character groups in `pattern` keywords. [#180](https://github.com/Stranger6667/jsonschema-rs/issues/180)
- Incorrect handling of strings that contain escaped character groups (like `\\w`) in `pattern` keywords.

## [0.6.0] - 2021-02-03

### Fixed

- Missing validation errors after the 1st one in `additionalProperties` validators.

### Performance

- Do not use `rayon` in `items` keyword as it gives significant overhead for a general case.
- Avoid partially overlapping work in `additionalProperties` / `properties` / `patternProperties` validators. [#173](https://github.com/Stranger6667/jsonschema-rs/issues/173)

## [0.5.0] - 2021-01-29

### Added

- Cache for documents loaded via the `$ref` keyword. [#75](https://github.com/Stranger6667/jsonschema-rs/issues/75)
- Meta schemas for JSON Schema drafts 4, 6, and 7. [#28](https://github.com/Stranger6667/jsonschema-rs/issues/28)

### Fixed

- Not necessary network requests for schemas with `$id` values with trailing `#` symbol. [#163](https://github.com/Stranger6667/jsonschema-rs/issues/163)

### Performance

- Enum validation for input values that have a type that is not present among the enum variants. [#80](https://github.com/Stranger6667/jsonschema-rs/issues/80)

### Removed

- `-V`/`--validator` options from the CLI. They were no-op and never worked.

## [0.4.3] - 2020-12-11

### Documentation

- Make examples in README.md runnable.

## [0.4.2] - 2020-12-11

### Changed

- Move `paste` to dev dependencies.

### Fixed

- Number comparison for `enum` and `const` keywords. [#149](https://github.com/Stranger6667/jsonschema-rs/issues/149)
- Do not accept `date` strings with single-digit month and day values. [#151](https://github.com/Stranger6667/jsonschema-rs/issues/151)

### Performance

- Some performance related changes were rolled back, due to increased complexity.

## [0.4.1] - 2020-12-09

### Fixed

- Integers not recognized as numbers when the `type` keyword is a list of multiple values. [#147](https://github.com/Stranger6667/jsonschema-rs/issues/147)

## [0.4.0] - 2020-11-09

### Added

- Command Line Interface. [#102](https://github.com/Stranger6667/jsonschema-rs/issues/102)
- `ToString` trait implementation for validators.
- Define `JSONSchema::options` to customise `JSONSchema` compilation [#131](https://github.com/Stranger6667/jsonschema-rs/issues/131)
- Allow user-defined `contentEncoding` and `contentMediaType` keywords

### Fixed

- ECMAScript regex support
- Formats should be associated to Draft versions (ie. `idn-hostname` is not defined on draft 4 and draft 6)

## [0.3.1] - 2020-06-21

### Changed

- Enable Link-Time Optimizations and set `codegen-units` to 1. [#104](https://github.com/Stranger6667/jsonschema-rs/issues/104)

### Fixed

- `items` allows the presence of boolean schemas. [#115](https://github.com/Stranger6667/jsonschema-rs/pull/115)

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

[Unreleased]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.6.1...HEAD
[0.6.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.6.0...rust-v0.6.1
[0.6.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.5.0...rust-v0.6.0
[0.5.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.4.3...rust-v0.5.0
[0.4.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.4.2...rust-v0.4.3
[0.4.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.4.1...rust-v0.4.2
[0.4.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.4.0...rust-v0.4.1
[0.4.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.3.1...rust-v0.4.0
[0.3.1]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.1.0...v0.2.0
