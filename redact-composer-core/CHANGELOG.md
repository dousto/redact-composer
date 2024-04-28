# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.2.4...redact-composer-core-v0.2.5) - 2024-04-28

### Added
- Add `synthesis` module to synthesize audio of `Composition`s

## [0.2.4](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.2.3...redact-composer-core-v0.2.4) - 2024-04-19

### Added
- Make `EndpointOffsets` trait public (used with `Timing::shift_by`)
- Allow specifying tuple for `Timing::shift_by`
- Update `CompositionContext::with_timing` signature for more convenient usage with most use cases

### Other
- Remove now unnecessary `.timing` accessor
- Remove now unnecessary `.timing` accessors
- Fix some typos in changelogs to make CI happy

## [0.2.3](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.2.2...redact-composer-core-v0.2.3) - 2024-04-19

### Added
- impl `From<SegmentRef<_>>` for `Timing`,`Range<i32>` to accomomdate frequent use cases.
- impl `From<&SegmentRef<_>>` for `Timing` and `Range<i32>` to accommodate frequent use cases.

### Other
- Update README with simple example changes

## [0.2.2](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.2.1...redact-composer-core-v0.2.2) - 2024-04-19

### Added
- Change composition render strategy to depth-first

## [0.2.1](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.2.0...redact-composer-core-v0.2.1) - 2024-03-09

### Other
- updated the following local packages: redact-composer-derive

## [0.2.0](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.1.2...redact-composer-core-v0.2.0) - 2024-03-05

### Added
- Allow naming / unnaming Segments as chained calls

### Other
- Update renamed trait (`IntoCompositionSegment` -> `IntoSegment`)

## [0.1.2](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.1.1...redact-composer-core-v0.1.2) - 2024-01-17

### Other
- Track workspace crate dependencies in individual Cargo.toml files

## [0.1.1](https://github.com/dousto/redact-composer/compare/redact-composer-core-v0.1.0...redact-composer-core-v0.1.1) - 2024-01-16

### Other
- release
