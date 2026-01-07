# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-   Added `Init::needed_init` to check if the initialization ran (#311)

### Fixed

-   Updated CLI template `increment_rs`: Changed authority validation to use `Authority` type wrapper instead of raw `Pubkey` in `IncrementAccounts` (#297)
-   Updated CLI template `states_rs`: Changed `AccountValidate` implementation for `CounterAccount` to use `Authority` type parameter instead of `&Pubkey`, and updated reference type from `Self::Ref<'_>` to `Self::Ptr` (#297)
-   Updated CLI template `initialize_rs`: Changed `Seeds` argument in `idl` macro to use `FindCounterAccountSeeds` instead of `FindCounterSeeds` (#297)
-   Check the account discriminants during client AccountDeserialize calls (#302)
-   Removed redundant account owner check during account creation flow (#303)

### Changed

-   Used `Option<&[&[u8]]>` instead of `Option<Vec<&[u8]>>` for `CanInitAccount::init_account` (#300)
-   CLI template: ensure crate name uses underscores in `cargo_toml`. Added `{name_lowercase_underscore}` placeholder and generator replacement to convert hyphens to underscores in generated project names. (#296)

## [0.27.0] - 2025-11-07

### Changed

-   Improved ergonomics of indexing into UnsizedList by allowing direct use of RangeBounds instead of wrapping in a tuple. (#289)
-   Unified `UnsizedType::Ref` and `UnsizedType::Mut` into a single `UnsizedType::Ptr` type (#291)
-   Updated #[unsized_impl] to implicitly work for only exclusive functions and relaxed the signature restriction to allow moved self parameters (#291)

### Fixed

-   Ensure our `Error` type implements `Send` and `Sync` traits. (#290)
-   Added additional pointer checks to unsized type to detect mem::swaps to prevent UB (#291)

## [0.26.3] - 2025-10-28

### Fixed

-   Fixed errors in cli template caused due to breaking changes made in star_frame (#283)
-   Fixed errors in cli test template caused due to breaking changes made in star_frame (#284)

## [0.26.2] - 2025-10-15

### Fixed

-   Reduced unsized map CU cost by 50% when key already exists (#281)
-   Use full result path in unsized_type proc macro (#281)

## [0.26.1] - 2025-10-02

### Fixed

-   Fixed error macros and add tests (#277)

### Removed

-   Removed `err` macro (#277)

## [0.26.0] - 2025-09-25

### Added

-   New unit system functions including checked math, overflow, and fixed support (#255).
-   `ClockExt` for unit system timestamps (#255).
-   `normalize_rent` alias for `AccountSet` `cleanup` (#263)
-   Changed from `anyhow` to `eyre` (#265)
-   Added `borsh` to `UnitVal` (#266)
-   Removed default init requirement on unsized list default init. (#267)
-   Added more `Align1` impls for tuples. (#268)
-   Added `star_frame_error` macro and custom error system (#271)

### Changed

-   Replaced `eyre` with custom error system (#271)

### Updated

-   Updated solana dependencies (#270)

## [0.25.1] - 2025-09-11

### Fixed

-   Improved performance of ProgramAccount discriminant validation (#260)

## [0.25.0] - 2025-09-11

### Fixed

-   Significantly improved performance across the board (#247)

### Changed

-   Updated `CpiAccountSet` to use native pinocchio instruction types (#247)
-   Updated `CpiBuilder` to take in InstructionData by value and a program argument instead of ctx,
    and `MakeCpi::cpi` is now infallible (#247)
-   Return data must be NoUninit instead of `BorshSerialize` (#247)

### Removed

-   Program cache from the `Context` struct (#247)

## [0.24.2] - 2025-09-06

### Fixed

-   Updated star_frame version in template (#252)

## [0.24.1] - 2025-09-04

### Fixed

-   Remove unnecessary `'static` bound on `InitFn` (#244)

## [0.24.0] - 2025-09-04

### Added

-   `BorshAccount` AccountSet (#225)
-   `UnsizedString` UnsizedType (#235)
-   `star_frame_instruction` macro (#199)
-   `zero_copy` attribute macro (#237)
-   CLI with project scaffolding (#227)
-   insert_all to map and set (#231)
-   Changelog tracking (#241)

### Changed

-   Bump workspace Rust version to 1.89.0 (#234)
-   Updated complex `CanInitAccount` implementations to use closures for init arg (#239)

## [0.23.1] - 2025-08-28

### Added

-   Additional documentation improvements (#223).

[unreleased]: https://github.com/staratlasmeta/star_frame/compare/v0.27.0...HEAD
[0.27.0]: https://github.com/staratlasmeta/star_frame/compare/v0.26.3...v0.27.0
[0.26.3]: https://github.com/staratlasmeta/star_frame/compare/v0.26.2...v0.26.3
[0.26.2]: https://github.com/staratlasmeta/star_frame/compare/v0.26.1...v0.26.2
[0.26.1]: https://github.com/staratlasmeta/star_frame/compare/v0.26.0...v0.26.1
[0.26.0]: https://github.com/staratlasmeta/star_frame/compare/v0.25.1...v0.26.0
[0.25.1]: https://github.com/staratlasmeta/star_frame/compare/v0.25.0...v0.25.1
[0.25.0]: https://github.com/staratlasmeta/star_frame/compare/v0.24.2...v0.25.0
[0.24.2]: https://github.com/staratlasmeta/star_frame/compare/v0.24.1...v0.24.2
[0.24.1]: https://github.com/staratlasmeta/star_frame/compare/v0.24.0...v0.24.1
[0.24.0]: https://github.com/staratlasmeta/star_frame/compare/v0.23.1...v0.24.0
[0.23.1]: https://github.com/staratlasmeta/star_frame/compare/v0.23.0...v0.23.1
