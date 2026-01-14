# Changelog
All notable changes to this project will be documented in this file.

## v0.6.1 - 2026-01-14
### Added
- `FixTaggedValue` trait for tag-bound values, implemented by FIX enums (including by-reference forms).
- `field_tagged`/`field_tagged_ref` and `try_field_tagged`/`try_field_tagged_ref` builder methods.
- `build_fix!` macro support for `tag => value` pairs and `@value` tagged fields (legacy `tag, value` remains supported).
- `tags` module with common FIX tag constants (not exhaustive).

### Documentation
- Updated README with tagged-value and new macro syntax examples.

## v0.6.0 - 2026-01-07
### Added
- `fixlite::tag::fix_tag_registry` re-export for registry macro access under the new module.
- `fixlite::decode` helper for parsing without calling the trait method.
- `fixlite::enums` module for FIX enum types.
- `fix_enum!` macro (renamed from `pub_fix_enum!`).

### Changed
- Tag/registry API moved to `fixlite::tag` (including `Registry`, `AllowedType`, `DefaultRegistry`).
- FIX enums moved to `fixlite::enums`.
- FixBuilder and builder traits moved to `fixlite::builder` (re-exported at crate root).
- Error types moved to `fixlite::error` (re-exported at crate root).

### Removed
- Root-level `fixlite::TagCursor` re-export (internal type is now hidden under `fixlite::__private`).
- `fixlite::fix::tag` module path (use `fixlite::tag` instead).
- `pub_fix_enum!` macro (use `fix_enum!`).
- `fixlite::fix::FixBuilder` path (use `fixlite::FixBuilder` or `fixlite::builder::FixBuilder`).

## v0.5.2 - 2026-01-07
### Added
- Derive macro now auto-detects the `fixlite` crate name, so renamed dependencies work without manual aliases.

## v0.5.1 - 2026-01-07
### Fixed
- Release workflow publishes `fixlite_derive` before `fixlite` and retries to allow crates.io index propagation.

## v0.5.0 - 2026-01-07
### Added
- `derive` feature to re-export `FixDeserialize` from `fixlite`, so users can depend on a single crate.

### Changed
- Internalized derive type-check helpers in `fixlite_derive` to avoid a crate cycle.

### Removed
- Public `fixlite::type_check` module (internal helper; no longer part of the API).

### Documentation
- Updated README usage to show the `derive` feature and single-dependency setup.

## v0.4.1 - 2025-12-26
### Changed
- FixBuilder `field`/`try_field` now take owned values; use `field_ref`/`try_field_ref` for borrowed values.
- f64 encoding now uses the shared checked helper from `TryFixValue` to keep behavior aligned.

### Documentation
- Updated README builder examples for owned/borrowed field APIs and fallible builder usage.

## v0.4.0 - 2025-12-25
### Added
- Fallible FixBuilder field APIs (`try_field`, `try_field_ref`, `try_fields`) plus `TryFixValue` for `f64` validation (NaN/inf now error).

### Documentation
- Documented public FixBuilder APIs and updated README with fallible builder usage.

## v0.3.27 - 2025-12-25
### Changed
- FixBuilder now returns a chainable message builder from `begin_with`, with an optional `fields` helper.
- Error handling distinguishes malformed FIX frames from semantic invalid values, and includes value context.
- `pub_fix_enum!` now takes numeric tag values (for example, `MsgType(35)`).
- Refined FixBuilder f64 encoding rounding/carry behavior.
- Reworked numeric encoding in FixBuilder, including fixed-precision f64 formatting and faster integer output.

### Removed
- ryu dependency (replaced by custom float encoding).

## v0.3.26 - 2025-12-24
### Added
- Optional single-pass BodyLength/CheckSum validation in `from_fix` behind the `checksum` feature flag.

## v0.3.25 - 2025-12-23
### Added
- FixBuilder API with FixValue/AsFixStr and the build_fix! macro for message construction.
- FixValue implementations for FIX enums, DateTime<Utc>, FixedPrice/Price, DayOfMonth, integers, bool, and f64.
- Builder benchmark in fixlite_example for a NewOrderSingle shape (including Parties group) and a builder/parser round-trip test.
- ryu dependency for allocation-free float encoding in the builder.
- CHANGELOG with tagged release history.

### Changed
- fixlite::fix now re-exports builder traits and types to make the builder public.
- README updated with builder usage and corrected MsgType paths.

## v0.3.24 - 2025-09-19

- No code changes recorded (version bump only).

## v0.3.23 - 2025-09-19

- furter optimized scanner + added bench
- removed unused dependency

## v0.3.21 - 2025-09-18

- fast fix scanner

## v0.3.20 - 2025-09-09

- fixed readme in Cargo.toml

## v0.3.19 - 2025-09-09

- fixing clippy errors

## v0.3.18 - 2025-09-09

- fixing clippy errors

## v0.3.17 - 2025-09-09

- adsded README.md to Cargo.toml

## v0.3.16 - 2025-09-09

- use edition 2024; formatted
- generate tag as u32
- added LGPL

## v0.3.15 - 2025-06-04

- GapFillFlag enum

## v0.3.14 - 2025-06-03

- renamed from_fix methods

## v0.3.13 - 2025-06-02

- added enum CxlRejResponseTo(434)

## v0.3.12 - 2025-06-01

- make enums Copy

## v0.3.11 - 2025-05-16

- updated DefaultRegistry

## v0.3.10 - 2025-05-16

- added ResetSeqNumFlag enum

## v0.3.9 - 2025-05-14

- Merge pull request #1 from dkumsh/release-0.3.9

## v0.3.8 - 2025-05-14

- No code changes recorded (version bump only).

## v0.3.7 - 2025-05-14

- README

## v0.3.6 - 2025-05-14

- cleanup

## v0.3.5 - 2025-05-14

- allow Option<T>

## v0.3.4 - 2025-05-14

- support for custom fix::tag::Registry

## v0.3.3 - 2025-05-12

- updatded FixedPrice

## v0.3.2 - 2025-05-12

- added FixedPrice

## v0.3.1 - 2025-05-12

- changed allowed types for tag 34

## v0.3.0 - 2025-05-09

- subcomponent fupport

## v0.2.19 - 2025-05-09

- added release command to justfile

## v0.2.18 - 2025-05-09

- fixed fix::DayOfMonth and allow using numeric literals with tag attributes

## v0.2.17 - 2025-05-09

- added fix::DayOfMonth and metadata for 205,314

## v0.2.16 - 2025-05-08

- added enum and metadata for tag 167: SecurityType

## v0.2.15 - 2025-05-08

- added enum and metadata for tag 373: SessionRejectReason

## v0.2.14 - 2025-05-08

- bumped version to 0.2.14: added metadata for  tag 21

## v0.2.13 - 2025-05-08

- bumped version to 0.2.13: added enum for tag 21

## v0.2.12 - 2025-05-08

- bumped version to 0.2.12: added metadata for tag 151

## v0.2.11 - 2025-05-08

- bumped version to 0.2.11: added more enums metadata (22,40)

## v0.2.10 - 2025-05-08

- bumped version to 0.2.10: fixed formatting

## v0.2.9 - 2025-05-08

- bumped version to 0.2.9: added more enums metadata

## v0.2.8 - 2025-05-08

- bumped version to 0.2.8: fix metadata for Price/Qty tags 14,31,32,38

## v0.2.7 - 2025-05-08

- bumped version to 0.2.7

## v0.2.6 - 2025-05-08

- updated enums
- bumped version to 0.2.6

## v0.2.5 - 2025-05-07

- bump version to v0.2.5

## v0.2.4 - 2025-05-07

- bump version to v0.2.4

## v0.2.3 - 2025-05-07

- fixed actions-rust-lang/setup-rust-toolchain@v1 parameter to toolchain
- add From<> and fmt::Display impls

## v0.2.2 - 2025-05-02

- releasing 0.2.2: bumped versions

## v0.2.1 - 2025-05-02

- fix deserialsier
- use known parent tags to detect end of repeating group
- v2
- syn version 2
- renamed to fixlite
- prepare to publish
- added more message type definitions
- Create release.yml
- Create ci.yml
- Update release.yml
- Update ci.yml
- Update ci.yml
- Update release.yml
- upgraded actions in release.yml
- Update ci.yml
- Update ci.yml
- Update release.yml
- bump version to v0.2.1
- bump version to v0.2.1
- Update release.yml
