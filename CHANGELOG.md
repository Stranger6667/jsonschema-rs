# Changelog

## [Unreleased]

### Fixed

- Re-export `referencing::Error` as `ReferencingError`. [#614](https://github.com/Stranger6667/jsonschema-rs/issues/614)

## [0.25.0] - 2024-10-24

**Important:** This release removes deprecated old APIs. See the [Migration Guide](MIGRATION.md) for details on transitioning to the new API.

### Changed

- **BREAKING**: Default to Draft 2020-12.

### Removed

- Deprecated `draft201909`, `draft202012`, and `cli` features.
- Deprecated `CompilationOptions`, `JSONSchema`, `PathChunkRef`, `JsonPointerNode`, and `SchemaResolverError` aliases.
- Deprecated `jsonschema::compile`, `Validator::compile`, `ValidationOptions::compile`, `ValidationOptions::with_resolver`, `ValidationOptions::with_meta_schemas`, `ValidationOptions::with_document` functions.
- Deprecated `SchemaResolver` trait.

## [0.24.3] - 2024-10-24

### Fixed

- Infinite recursion when using mutually recursive `$ref` in `unevaluatedProperties`.

## [0.24.2] - 2024-10-24

### Fixed

- Infinite recursion in some cases. [#146](https://github.com/Stranger6667/jsonschema-rs/issues/146)
- `$ref` interaction with `$recursiveAnchor` in Draft 2019-09.
- `unevaluatedProperties` with `$recursiveRef` & `$dynamicRef`.

## [0.24.1] - 2024-10-21

### Fixed

- Incomplete external reference resolution.

## [0.24.0] - 2024-10-20

### Added

- Support `$ref`, `$recursiveRef`, and `$dynamicRef` in `unevaluatedItems`. [#287](https://github.com/Stranger6667/jsonschema-rs/issues/287)
- Support for `$vocabulary`. [#263](https://github.com/Stranger6667/jsonschema-rs/issues/263)

### Changed

- Ignore `prefixItems` under Draft 2019-09 as it was introduced in Draft 2020-12.

### Fixed

- Numbers with zero fraction incorrectly handled in `uniqueItems`.

### Performance

- Speedup `apply`.

## [0.23.0] - 2024-10-12

### Added

- Partial support for `unevaluatedItems`, excluding references.

### Changed

- Improve error messages on WASM. [#568](https://github.com/Stranger6667/jsonschema-rs/issues/568)
- Improve error messages on URI resolving and parsing.
- **BREAKING**: Replace `JsonPointer` in favor of `Location`.

### Deprecated

- `PathChunkRef` in favor of `LocationSegment`.
- `JsonPointerNode` in favor of `LazyLocation`.

### Fixed

- Resolving file references on Windows. [#441](https://github.com/Stranger6667/jsonschema-rs/issues/441)
- Missing annotations from by-reference applicators. [#403](https://github.com/Stranger6667/jsonschema-rs/issues/403)
- Relative keyword locations missing by-reference applicators (such as `$ref` or `$dynamicRef`).

### Performance

- Faster building of a validator.
- Speedup `hostname` & `idn-hostname` formats validation.
- Speedup `apply`.

### Removed

- `JsonPointerNode::to_vec` without a replacement.

## [0.22.3] - 2024-10-05

### Performance

- Speedup resolving.

## [0.22.2] - 2024-10-04

### Fixed

- ECMAScript 262 regex support.

### Performance

- Speedup `json-pointer` and `relative-json-pointer` formats validation.

## [0.22.1] - 2024-10-03

### Fixed

- Removed `dbg!` macro.

## [0.22.0] - 2024-10-03

### Changed

- Extend email validation. [#471](https://github.com/Stranger6667/jsonschema-rs/issues/471)
- **BREAKING**: Custom retrievers now receive `&Uri<&str>` instead of `&UriRef<&str>`
- Bump `once_cell` to `1.20`.
- Bump `regex` to `1.11`.

### Fixed

- `time` format validation (leap seconds and second fractions).
- `duration` format validation.
- Panic on root `$id` without base. [#547](https://github.com/Stranger6667/jsonschema-rs/issues/547)
- `hostname` format validation (double dot).
- `idn-hostname` format validation. [#101](https://github.com/Stranger6667/jsonschema-rs/issues/101)

### Performance

- Faster building of a validator.
- Speedup `hostname`, `date`, `time`, `date-time`, and `duration` formats validation.
- Cache regular expressions for `pattern`. [#417](https://github.com/Stranger6667/jsonschema-rs/issues/417)

## [0.21.0] - 2024-09-29

**Important:** This release brings a complete rework of reference resolving which deprecates some older APIs.
While backward compatibility is maintained for now, users are encouraged to update their code. See the [Migration Guide](MIGRATION.md) for details on transitioning to the new API.

### Added

- `$anchor` support.
- `$recursiveRef` & `$recursiveAnchor` support in Draft 2019-09.
- `$dynamicRef` & `$dynamicAnchor` support in Draft 2020-12.

### Changed

- **BREAKING**: Treat `$ref` as URI, not URL, and additionally normalize them. [#454](https://github.com/Stranger6667/jsonschema-rs/issues/454)
- **BREAKING**: Resolve all non-recursive references eagerly.
- **BREAKING**: Disallow use of fragments in `$id`. [#264](https://github.com/Stranger6667/jsonschema-rs/issues/264)

### Deprecated

- `SchemaResolver` trait and `SchemaResolverError` in favor of a simpler `Retrieve` that works with `Box<dyn std::error::Error>`. 
  In turn, it also deprecates `ValidationOptions::with_resolver` in favor of `ValidationOptions::with_retriever`
- `ValidationOptions::with_document` in favor of `ValidationOptions::with_resource`.

### Fixed

- Infinite recursion in `unevaluatedProperties`. [#420](https://github.com/Stranger6667/jsonschema-rs/issues/420)
- Cross-draft validation from newer to older ones.
- Changing base URI in folder.
- Location-independent identifier in remote resource.
- Missing some format validation for Draft 2020-12.
- Incomplete `iri` & `iri-reference` validation.

### Performance

- Faster validation for `uri`, `iri`, `uri-reference`, and `iri-reference` formats.

## [0.20.0] - 2024-09-18

**Important:** This release includes several deprecations and renames. While backward compatibility is maintained for now, users are encouraged to update their code. See the [Migration Guide](MIGRATION.md) for details on transitioning to the new API.

### Added

- New draft-specific modules for easier version-targeted validation:
  - `jsonschema::draft4`
  - `jsonschema::draft6`
  - `jsonschema::draft7`
  - `jsonschema::draft201909`
  - `jsonschema::draft202012`
  Each module provides `new()`, `is_valid()`, and `options()` functions.
- `jsonschema::options()` function as a shortcut for `jsonschema::Validator::options()`, that allows for customization of the validation process.

### Changed

- Make `Debug` implementation for `SchemaNode` opaque.
- Make `jsonschema::validator_for` and related functions return `ValidationError<'static>` in their `Err` variant.
  This change makes possible to use the `?` operator to return errors from functions where the input schema is defined.

### Deprecated

- Rename `CompilationOptions` to `ValidationOptions` for clarity.
- Rename `JSONSchema` to `Validator` for clarity. [#424](https://github.com/Stranger6667/jsonschema-rs/issues/424)
- Rename `JSONPointer` to `JsonPointer` for consistency with naming conventions. [#424](https://github.com/Stranger6667/jsonschema-rs/issues/424)
- Rename `jsonschema::compile` to `jsonschema::validator_for`.
- Rename `CompilationOptions::compile` to `ValidationOptions::build`.

Old names are retained for backward compatibility but will be removed in a future release.

### Fixed

- Location-independent references in remote schemas on drafts 4, 6, and 7.

## [0.19.1] - 2024-09-15

### Fixed

- `ipv4` format validation. [#512](https://github.com/Stranger6667/jsonschema-rs/issues/512)

## [0.19.0] - 2024-09-14

### Added

- `jsonschema::compile` shortcut.

### Changed

- Bump MSRV to `1.70`.

### Fixed

- `uuid` format validation.
- Combination of `unevaluatedProperties` with `allOf` and `oneOf`. [#496](https://github.com/Stranger6667/jsonschema-rs/issues/496)

### Deprecated

- `cli` feature in favor of a separate `jsonschema-cli` crate.
- `draft201909` and `draft202012` features. The relevant functionality is now enabled by default.

### Performance

- `uuid` validation via `uuid-simd`.

## [0.18.3] - 2024-09-12

### Fixed

- Changing base URI when `$ref` is present in drafts 7 and earlier.
- Removed `dbg!` macro.

## [0.18.2] - 2024-09-11

### Fixed

- Ignoring `$schema` in resolved references.
- Support integer-valued numbers for `maxItems`, `maxLength`, `maxProperties`, `maxContains`, `minItems`, `minLength`, `minProperties`, `minContains`.

### Deprecated

- `with_meta_schemas()` method. Meta schemas are included by default.

## [0.18.1] - 2024-08-24

### Added

- `ErrorDescription::into_inner` to retrieve the inner `String` value.

## [0.18.0] - 2024-05-07

### Added

- Custom keywords support. [#379](https://github.com/Stranger6667/jsonschema-rs/issues/379)
- Expose `JsonPointerNode` that can be converted into `JSONPointer`.
  This is needed for the upcoming custom validators support.

### Changed

- Bump `base64` to `0.22`.
- Bump `clap` to `4.5`.
- Bump `fancy-regex` to `0.13`.
- Bump `fraction` to `0.15`.
- Bump `memchr` to `2.7`.
- Bump `once_cell` to `1.19`.
- Bump `percent-encoding` to `2.3`.
- Bump `regex` to `1.10`.
- Bump `url` to `2.5`.
- Build CLI only if the `cli` feature is enabled.
- **BREAKING**: Extend `CompilationOptions` to support more ways to define custom format checkers (for example in Python bindings).
  In turn it changes `ValidationErrorKind::Format` to contain a `String` instead of a `&'static str`.

### Fixed

- Incorrect `schema_path` when multiple errors coming from the `$ref` keyword [#426](https://github.com/Stranger6667/jsonschema-rs/issues/426)

### Performance

- Optimize building `JSONPointer` for validation errors by allocating the exact amount of memory needed.
- Avoid cloning path segments during validation.

## [0.17.1] - 2023-07-05

### Changed

- Improved error messages for `oneOf` / `anyOf` keywords. [#429](https://github.com/Stranger6667/jsonschema-rs/issues/429)

### Fixed

- Improper handling of subschema validation in `unevaluatedProperties`. [#421](https://github.com/Stranger6667/jsonschema-rs/issues/421)

## [0.17.0] - 2023-03-16

### Changed

- Bump `base64` to `0.21`.
- Bump `fancy-regex` to `0.11`.
- Bump `fraction` to `0.13`.
- Bump `iso8601` to `0.6`.
- Replace `lazy_static` with `once_cell`.
- Add support for `unevaluatedProperties`. (gated by the `draft201909`/`draft202012` feature flags). [#288](https://github.com/Stranger6667/jsonschema-rs/issues/288)
- When using the draft 2019-09 or draft 2020-12 specification, `$ref` is now evaluated alongside
  other keywords. [#378](https://github.com/Stranger6667/jsonschema-rs/issues/378)

## [0.16.1] - 2022-10-20

### Added

- Add a compilation option (`should_ignore_unknown_formats()`) that allows treating unknown formats as compilation errors.

## [0.16.0] - 2022-04-21

### Fixed

- Library compilation with no default features. [#356](https://github.com/Stranger6667/jsonschema-rs/issues/356)
- Compilation with `resolve-file` only. [#358](https://github.com/Stranger6667/jsonschema-rs/issues/358)

### Changed

- **BREAKING**: Revert changes from [#353](https://github.com/Stranger6667/jsonschema-rs/issues/353) and [#343](https://github.com/Stranger6667/jsonschema-rs/issues/343), as they caused compilation issues.

## [0.15.2] - 2022-04-10

### Fixed

- Allow HTTP(S) schema resolving with `rustls`. [#353](https://github.com/Stranger6667/jsonschema-rs/issues/353)

## [0.15.1] - 2022-04-02

### Fixed

- Enable `reqwest/native-tls` by default to avoid validation errors caused by `reqwest` missing a TLS backend. [#343](https://github.com/Stranger6667/jsonschema-rs/issues/343)

## [0.15.0] - 2022-01-31

### Added

- The `SchemaResolver` trait to support resolving external schema references. [#246](https://github.com/Stranger6667/jsonschema-rs/issues/246)
- `resolve-file` feature to resolve external schema files via `std::fs`. [#76](https://github.com/Stranger6667/jsonschema-rs/issues/76)

### Changed

- The `reqwest` feature was changed to `resolve-http`. [#341](https://github.com/Stranger6667/jsonschema-rs/pull/341)

### Performance

- CLI: Use `serde::from_reader` instead of `serde::from_str`.

## [0.14.0] - 2022-01-23

### Changed

- Bump `itoa` to `1.0`. [#337](https://github.com/Stranger6667/jsonschema-rs/issues/337)

### Performance

- Optimize the loop implementation used for uniqueness check on short arrays.
- Simplify `equal_arrays` helper.
- Shortcut for `false` schemas.
- Reduce the number of generated LLVM lines.
- Do less work when resolving fragments.
- Avoid cloning the value when resolving empty fragments.
- Optimize searching by pointer in JSON documents.

## [0.13.3] - 2021-12-08

### Changed

- Make `BasicOutput.is_valid` public.

### Fixed

- False positives in some cases when calling `JSONSchema.apply` on schemas with `additionalProperties`, `patternProperties`, and `properties` combined.
- False negatives in some cases when calling `JSONSchema.apply` on schemas with `if` and `then` (without `else`) keywords. [#318](https://github.com/Stranger6667/jsonschema-rs/pull/318)
- Panic in `JSONSchema.apply` on some schemas with `prefixItems` and `items`. It panicked if `items` is an object and the length of `prefixItems` is greater than the length of the input array.

### Performance

- Remove unused private field in `JSONSchema`, that lead to improvement in the compilation performance.
- Optimize the `multipleOf` implementation, which now can short-circuit in some cases.
- Add special cases for arrays with 2 and 3 items in the `uniqueItems` keyword implementation.
- Remove the `schema` argument from all methods of the `Validate` trait.

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

[Unreleased]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.25.0...HEAD
[0.25.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.24.3...rust-v0.25.0
[0.24.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.24.2...rust-v0.24.3
[0.24.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.24.1...rust-v0.24.2
[0.24.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.24.0...rust-v0.24.1
[0.24.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.23.0...rust-v0.24.0
[0.23.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.22.3...rust-v0.23.0
[0.22.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.22.2...rust-v0.22.3
[0.22.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.22.1...rust-v0.22.2
[0.22.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.22.0...rust-v0.22.1
[0.22.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.21.0...rust-v0.22.0
[0.21.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.20.0...rust-v0.21.0
[0.20.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.19.1...rust-v0.20.0
[0.19.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.19.0...rust-v0.19.1
[0.19.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.18.3...rust-v0.19.0
[0.18.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.18.2...rust-v0.18.3
[0.18.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.18.1...rust-v0.18.2
[0.18.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.18.0...rust-v0.18.1
[0.18.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.17.1...rust-v0.18.0
[0.17.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.17.0...rust-v0.17.1
[0.17.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.16.1...rust-v0.17.0
[0.16.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.16.0...rust-v0.16.1
[0.16.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.15.2...rust-v0.16.0
[0.15.2]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.15.1...rust-v0.15.2
[0.15.1]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.15.0...rust-v0.15.1
[0.15.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.14.0...rust-v0.15.0
[0.14.0]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.13.3...rust-v0.14.0
[0.13.3]: https://github.com/Stranger6667/jsonschema-rs/compare/rust-v0.13.2...rust-v0.13.3
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
[0.4.0]: https://github.com/Stranger6667/jsonschema-rs/compare/2bf8ccb78070cc10d1006789923a102e07da499d...rust-v0.4.0
[0.3.1]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.3.0...2bf8ccb78070cc10d1006789923a102e07da499d
[0.3.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Stranger6667/jsonschema-rs/compare/v0.1.0...v0.2.0
