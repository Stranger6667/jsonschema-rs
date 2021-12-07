# Changelog

## [Unreleased]

### Performance

- Remove unused private field in `JSONSchema`, that lead to improvement in the compilation performance.
- Optimize the `multipleOf` implementation, which now can short-circuit in some cases.
- Add special cases for arrays with 2 and 3 items in the `uniqueItems` keyword implementation.
- Remove the `schema` argument from all methods of the `Validate` trait.
- Skip creating validators for always valid schemas such as `true` and `{}`.

## [0.13.2] - 2021-11-04

### Added

- Support for `prefixItems` keyword. [#303](https://github.com/Stranger6667/jsonschema-rs/pull/303)
- Expose methods to examine `OutputUnit`.

## [0.13.1] - 2021-10-28

### Fixed

- Missing `derive` from `serde`.

## [0.13.0] - 2021-10-28

### Added

- `uuid` format validator. [#266](https://github.com/Stranger6667/jsonschema-rs/issues/266)
- `duration` format validator. [#265](https://github.com/Stranger6667/jsonschema-rs/issues/265)
- Collect annotations whilst evaluating schemas. [#262](https://github.com/Stranger6667/jsonschema-rs/issues/262)
- Option to turn off processing of the `format` keyword. [#261](https://github.com/Stranger6667/jsonschema-rs/issues/261)
- `basic` & `flag` output formatting styles. [#100](https://github.com/Stranger6667/jsonschema-rs/issues/100)
- Support for `dependentRequired` & `dependentSchemas` keywords. [#286](https://github.com/Stranger6667/jsonschema-rs/issues/286)
- Forward `reqwest` features.

### Changed

- **INTERNAL**. A new `Draft201909` variant for the `Draft` enum that is available only under the `draft201909` feature. This feature is considered private and should not be used outside of the testing context.
  It allows us to add features from the 2019-09 Draft without exposing them in the public API. Therefore, support for this draft can be added incrementally.
- The `Draft` enum is now marked as `non_exhaustive`.
- `ValidationError::schema` was removed and the calls replaced by proper errors.

### Performance

- Reduce the size of `PrimitiveTypesBitMapIterator` from 3 to 2 bytes. [#282](https://github.com/Stranger6667/jsonschema-rs/issues/282)
- Use the `bytecount` crate for `maxLength` & `minLength` keywords, and for the `hostname` format.

## [0.12.2] - 2021-10-21

### Fixed

- Display the original value in errors from `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`. [#215](https://github.com/Stranger6667/jsonschema-rs/issues/215)
- Switch from `chrono` to `time==0.3.3` due to [RUSTSEC-2020-0159](https://rustsec.org/advisories/RUSTSEC-2020-0159.html) in older `time` versions that `chrono` depends on.

## [0.12.1] - 2021-07-29

### Fixed

- Allow using empty arrays or arrays with non-unique elements for the `enum` keyword in schemas. [#258](https://github.com/Stranger6667/jsonschema-rs/issues/258)
- Panic on incomplete escape sequences in regex patterns. [#253](https://github.com/Stranger6667/jsonschema-rs/issues/253)

## [0.12.0] - 2021-07-24

### Added

- Support for custom `format` validators. [#158](https://github.com/Stranger6667/jsonschema-rs/issues/158)

### Changed

- Validators now implement `Display` instead of `ToString`.
- `JSONSchema` now owns its data. [#145](https://github.com/Stranger6667/jsonschema-rs/issues/145)

## [0.11.0] - 2021-06-19

### Added

- Report schema paths in validation errors - `ValidationError.schema_path`. [#199](https://github.com/Stranger6667/jsonschema-rs/issues/199)

### Fixed

- Incorrect encoding of `/` and `~` characters in `fmt::Display` implementation for `JSONPointer`. [#233](https://github.com/Stranger6667/jsonschema-rs/issues/233)

## [0.10.0] - 2021-06-17

### Added

- **BREAKING**: Meta-schema validation for input schemas. By default, all input schemas are validated with their respective meta-schemas
  and instead of `CompilationError` there will be the usual `ValidationError`. [#198](https://github.com/Stranger6667/jsonschema-rs/issues/198)

### Removed

- `CompilationError`. Use `ValidationError` instead.

## [0.9.1] - 2021-06-17

### Fixed

- The `format` validator incorrectly rejecting supported regex patterns. [#230](https://github.com/Stranger6667/jsonschema-rs/issues/230)

## [0.9.0] - 2021-05-07

### Added

- Support for look-around patterns. [#183](https://github.com/Stranger6667/jsonschema-rs/issues/183)

### Fixed

- Extend the `email` format validation. Relevant test case from the JSONSchema test suite - `email.json`.

## [0.8.3] - 2021-05-05

### Added

- `paths::JSONPointer` implements `IntoIterator` over `paths::PathChunk`.

### Fixed

- Skipped validation on an unsupported regular expression in `patternProperties`. [#213](https://github.com/Stranger6667/jsonschema-rs/issues/213)
- Missing `array` type in error messages for `type` validators containing multiple values. [#216](https://github.com/Stranger6667/jsonschema-rs/issues/216)

## [0.8.2] - 2021-05-03

### Performance

- Avoid some repetitive `String` allocations during validation.
- Reduce the number of `RwLock.read()` calls in `$ref` validators.
- Shortcut in the `uniqueItems` validator for short arrays.
- `additionalProperties`. Use vectors instead of `AHashMap` if the number of properties is small.
- Special handling for single-item `required` validators.
- Special handling for single-item `enum` validators.
- Special handling for single-item `allOf` validators.
- Special handling for single-item `patternProperties` validators without defined `additionalProperties`.

### Fixed

- Floating point overflow in the `multipleOf` validator. Relevant test case from the JSONSchema test suite - `float_overflow.json`.

## [0.8.1] - 2021-04-30

### Performance

- Avoid `String` allocation in `JSONPointer.into_vec`.
- Replace heap-allocated `InstancePath` with stack-only linked list.

## [0.8.0] - 2021-04-27

### Changed

- The `propertyNames` validator now contains the parent object in its `instance` attribute instead of individual properties as strings.
- Improved error message for the `additionalProperties` validator. After - `Additional properties are not allowed ('faz' was unexpected)`, before - `False schema does not allow '"faz"'`.
- The `additionalProperties` validator emits a single error for all unexpected properties instead of separate errors for each unexpected property.
- **Breaking**: `ValidationError.instance_path` is now a separate struct, that can be transformed to `Vec<String>` or JSON Pointer of type `String`.

### Fixed

- All `instance_path` attributes are pointing to the proper location.

## [0.7.0] - 2021-04-27

### Added

- `ValidationError.instance_path` that shows the path to the erroneous part of the input instance.
  It has the `Vec<String>` type and contains components of the relevant JSON pointer.

### Changed

- Make fields of `ValidationError` public. It allows the end-user to customize errors formatting.

### Fixed

- Reject IPv4 addresses with leading zeroes. As per the new test case in the JSONSchema test suite. [More info](https://sick.codes/universal-netmask-npm-package-used-by-270000-projects-vulnerable-to-octal-input-data-server-side-request-forgery-remote-file-inclusion-local-file-inclusion-and-more-cve-2021-28918/)
- Do not look for sub-schemas inside `const` and `enum` keywords. Fixes an issue checked by [these tests](https://github.com/json-schema-org/JSON-Schema-Test-Suite/pull/471) 
- Check all properties in the `required` keyword implementation. [#190](https://github.com/Stranger6667/jsonschema-rs/issues/190)

### Removed

- Not used `ValidationErrorKind::Unexpected`.

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

[Unreleased]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.13.2...HEAD
[0.13.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.13.1...rust-v0.13.2
[0.13.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.13.0...rust-v0.13.1
[0.13.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.12.2...rust-v0.13.0
[0.12.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.12.1...rust-v0.12.2
[0.12.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.12.0...rust-v0.12.1
[0.12.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.11.0...rust-v0.12.0
[0.11.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.10.0...rust-v0.11.0
[0.10.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.9.1...rust-v0.10.0
[0.9.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.9.0...rust-v0.9.1
[0.9.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.8.3...rust-v0.9.0
[0.8.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.8.2...rust-v0.8.3
[0.8.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.8.1...rust-v0.8.2
[0.8.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.8.0...rust-v0.8.1
[0.8.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.7.0...rust-v0.8.0
[0.7.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.6.1...rust-v0.7.0
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
