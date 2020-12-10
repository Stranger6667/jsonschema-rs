# Changelog

## [Unreleased]

### Fixed

- Number comparison for `enum` and `const` keywords. [#149](https://github.com/Stranger6667/jsonschema-rs/issues/149)

## [0.4.1] - 2020-12-09

### Fixed

- Integers not recognized as numbers when the `type` keyword is a list of multiple values. [#147](https://github.com/Stranger6667/jsonschema-rs/issues/147)

## [0.4.0] - 2020-11-09

### Added

- Python 3.9 support.

### Changed

- Remove not needed `__init__.py` file. It improves performance for compiled schemas. [#121](https://github.com/Stranger6667/jsonschema-rs/issues/121)
- Update `PyO3` to `0.12`. [#125](https://github.com/Stranger6667/jsonschema-rs/issues/125)
- Use stable Rust.
- Set module documentation only once.

### Fixed

- ECMAScript regex support
- Formats should be associated to Draft versions (ie. `idn-hostname` is not defined on draft 4 and draft 6)
- Handle errors during conversion to `Value` instead of using `unwrap` in `JSONSchema::is_valid` and `JSONSchema::validate`. [#127](https://github.com/Stranger6667/jsonschema-rs/issues/127)

### Removed

- Python 3.5 support.

## [0.3.3] - 2020-06-22

### Fixed

- `items` allows the presence of boolean schemas. [#115](https://github.com/Stranger6667/jsonschema-rs/pull/115)

## [0.3.2] - 2020-06-13

### Fixed

- Packaging issue.

## [0.3.1] - 2020-06-12

### Added

- Added `jsonschema_rs.__build__` which contains useful build information. [#111](https://github.com/Stranger6667/jsonschema-rs/pulls/111)
- Wheels for Mac OS and Windows. [#110](https://github.com/Stranger6667/jsonschema-rs/issues/110)

### Changed

- Linux wheels are `manylinux2014` compatible. Previously they were `manylinux2010` compatible. [#111](https://github.com/Stranger6667/jsonschema-rs/pulls/111)

## [0.3.0] - 2020-06-11

### Fixed

- Copying not needed compiled files to the wheel distribution files. [#109](https://github.com/Stranger6667/jsonschema-rs/issues/109)

## [0.2.0] - 2020-06-11

### Added

- `JSONSchema.validate` method that raises `ValidationError` for invalid input. [#105](https://github.com/Stranger6667/jsonschema-rs/issues/105)

### Changed

- Public functions docstrings to support PyCharm skeletons generation. Functions signatures now have proper signatures (but untyped) in PyCharm. [#107](https://github.com/Stranger6667/jsonschema-rs/issues/107)
- Enable Link-Time Optimizations and set `codegen-units` to 1. [#104](https://github.com/Stranger6667/jsonschema-rs/issues/104)

## 0.1.0 - 2020-06-09

- Initial public release

[Unreleased]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.4.1...HEAD
[0.4.1]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.4.0...python-v0.4.1
[0.4.0]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.3.3...python-v0.4.0
[0.3.3]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.3.2...python-v0.3.3
[0.3.2]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.3.1...python-v0.3.2
[0.3.1]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.3.0...python-v0.3.1
[0.3.0]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.2.0...python-v0.3.0
[0.2.0]: https://github.com/Stranger6667/jsonschema-rs/compare/python-v0.1.0...python-v0.2.0
