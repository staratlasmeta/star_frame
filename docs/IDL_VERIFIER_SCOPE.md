# IDL Verifier Scope

This document defines the scope and guarantees of `star_frame_idl` verifier checks.

## Purpose

`verify_idl_definitions(...)` validates that an IDL definition graph is structurally sound.
It is intentionally fail-closed: unknown/unimplemented paths must return `Err`.

## Verification Modes

- `verify_idl_definitions(...)` uses `VerificationMode::Compatibility` (default):
  - Namespaced type references are resolved with local `get_type` lookup first (`types` + `external_types`), then fallback to provided namespace definitions.
  - Namespaced account references are resolved against local `accounts` first, then fallback to provided namespace definitions.
- `verify_idl_definitions_strict(...)` uses `VerificationMode::StrictGraph`:
  - Namespaced references must resolve through the provided definition set.
  - Embedded local copies alone do not satisfy a missing namespace definition.

## In Scope (Structural Checks)

- Namespace validity:
  - Namespace must be non-empty.
  - Namespaces must be unique across provided definitions.
- Cross-reference resolution:
  - Type references resolve (local/external namespaces).
  - Account set references resolve.
  - Account references in single account sets resolve.
  - Instruction and account type references resolve.
- Shape constraints:
  - `Many` bounds are valid (`max >= min` when `max` is present).
  - `Or` account sets are non-empty.
- Generic arity checks:
  - Defined type references provide the expected number of generics.
  - Defined account set references provide expected type/account generic arity.

## Out of Scope

- Codama semantic validation or internal Codama node behavior.
- Program/business logic correctness.
- Audit-grade guarantees beyond structural graph validity.

## Feature-Gate Behavior

- `feature = "verifier"` enabled:
  - Verifier API is available.
  - Structural checks run and fail closed.
- `feature = "verifier"` disabled:
  - Verifier module API is not exposed.
  - Compile-gate coverage enforces this (`tests/verifier_feature_gate.rs`).

## Rule IDs

The verifier emits stable rule identifiers in errors:

- `SFIDL001` empty namespace
- `SFIDL002` duplicate namespace
- `SFIDL003` missing namespace reference
- `SFIDL004` missing type
- `SFIDL005` type generic arity mismatch
- `SFIDL006` missing account set
- `SFIDL007` account-set type generic arity mismatch
- `SFIDL008` account-set account generic arity mismatch
- `SFIDL009` missing account
- `SFIDL010` invalid `Many` bounds
- `SFIDL011` empty `Or`
