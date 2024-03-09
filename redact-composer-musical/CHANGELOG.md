# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.2.0...redact-composer-musical-v0.2.1) - 2024-03-09

### Fixed
- Fixed bug in note iteration w/ updated and reorganized tests ([#33](https://github.com/dousto/redact-composer/pull/33))

## [0.2.0](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.1.4...redact-composer-musical-v0.2.0) - 2024-03-05

### Added
- Added `PitchClass` concept
- Added `Interval` concept
- Added `Note` concept
- Added `NoteName` concept
- Reworked `Chord` to represent as a `PitchClass` + `ChordShape` instead of purely relative
- Replaced `Notes` w/ new `NoteIterator` trait
- Updated `Scale` w/ newly added concepts
- Updated `Key` w/ newly added concepts
- Add `derive(Element)` for `Rhythm`

### Other
- Re-organize crate structure as separate modules.

## [0.1.4](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.1.3...redact-composer-musical-v0.1.4) - 2024-01-19

### Other
- Update Element link to point to trait instead of macro

## [0.1.3](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.1.2...redact-composer-musical-v0.1.3) - 2024-01-18

### Other
- *(deps)* Update redact-composer-core to 0.1.2

## [0.1.2](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.1.1...redact-composer-musical-v0.1.2) - 2024-01-17

### Other
- Track workspace crate dependencies in individual Cargo.toml files

## [0.1.1](https://github.com/dousto/redact-composer/compare/redact-composer-musical-v0.1.0...redact-composer-musical-v0.1.1) - 2024-01-16

### Other
- release
