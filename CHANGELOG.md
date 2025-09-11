# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.25.1] - 2025-09-11

### Fixed

- Improved performance of ProgramAccount discriminant validation (#260)

## [0.25.0] - 2025-09-11

### Fixed

- Significantly improved performance across the board (#247)

### Changed

- Updated `CpiAccountSet` to use native pinocchio instruction types (#247)
- Updated `CpiBuilder` to take in InstructionData by value and a program argument instead of ctx,
  and `MakeCpi::cpi` is now infallible (#247)
- Return data must be NoUninit instead of `BorshSerialize` (#247)

### Removed

- Program cache from the `Context` struct (#247)

## [0.24.2] - 2025-09-06

### Fixed

- Updated star_frame version in template (#252)

## [0.24.1] - 2025-09-04

### Fixed

- Remove unnecessary `'static` bound on `InitFn` (#244)

## [0.24.0] - 2025-09-04

### Added

- `BorshAccount` AccountSet (#225)
- `UnsizedString` UnsizedType (#235)
- `star_frame_instruction` macro (#199)
- `zero_copy` attribute macro (#237)
- CLI with project scaffolding (#227)
- insert_all to map and set (#231)
- Changelog tracking (#241)

### Changed

- Bump workspace Rust version to 1.89.0 (#234)
- Updated complex `CanInitAccount` implementations to use closures for init arg (#239)

## [0.23.1] - 2025-08-28

### Added

- Additional documentation improvements (#223).

[unreleased]: https://github.com/staratlasmeta/star_frame/compare/v0.25.1...HEAD
[0.25.1]: https://github.com/staratlasmeta/star_frame/compare/v0.25.0...v0.25.1
[0.25.0]: https://github.com/staratlasmeta/star_frame/compare/v0.24.2...v0.25.0
[0.24.2]: https://github.com/staratlasmeta/star_frame/compare/v0.24.1...v0.24.2
[0.24.1]: https://github.com/staratlasmeta/star_frame/compare/v0.24.0...v0.24.1
[0.24.0]: https://github.com/staratlasmeta/star_frame/compare/v0.23.1...v0.24.0
[0.23.1]: https://github.com/staratlasmeta/star_frame/compare/v0.23.0...v0.23.1
