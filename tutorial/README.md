
# Star Frame Tutorial Series

This tutorial series teaches Star Frame concepts progressively, mirroring the learning path of Anchor's tutorials while showcasing Star Frame's unique features.

## Tutorial Progression

### basic-0: Minimal Program
**Concepts:** Program structure, instruction set, entrypoint
- Simplest possible Star Frame program
- No instructions, just the basic structure
- Demonstrates the framework's minimal boilerplate

### basic-1: Data Storage & Updates
**Concepts:** Account initialization, state management, instruction processing
- Store and update a simple counter
- Initialize accounts with PDA seeds
- Basic instruction implementation

### basic-2: Access Control & Authority
**Concepts:** Account validation, authority patterns, modifiers
- Authority-based access control
- `AccountValidate` trait implementation
- `ValidatedAccount` wrapper usage

### basic-3: Cross-Program Invocation (CPI)
**Concepts:** CPI calls, program composition, account passing
- Two programs: puppet and puppet-master
- Demonstrate Star Frame's CPI patterns
- Cross-program account validation

### basic-4: Advanced PDAs & Error Handling
**Concepts:** Complex seeds, custom errors, validation patterns
- Multi-seed PDA derivation
- Custom error types with Star Frame
- Advanced account validation scenarios

### basic-5: Complex State Management
**Concepts:** State machines, user-specific accounts, multiple instructions
- Robot state machine example
- User-specific PDAs
- Multiple state transitions

## Running the Tutorials

Each tutorial contains:
- `Cargo.toml` - Dependencies and build configuration
- `src/lib.rs` - The Star Frame program
- `tests/` - Test files demonstrating usage (when applicable)

### Quick Start with Just

The tutorials include a `justfile` for convenient build and test commands:

```bash
# Check that all tutorials compile
just check

# Run comprehensive checks (compile, tests, clippy, idl)
just check-all

# Build all tutorials in release mode
just build

# Clean all build artifacts
just clean

# Format all tutorial code
just fmt

# Generate IDL files for all tutorials
just idl

# List all available commands
just --list
```

### Building Individual Tutorials

```bash
cd tutorial/basic-X
cargo build --release
```

### Testing

```bash
# Test a single tutorial
cd tutorial/basic-X
cargo test --features idl

# Test all tutorials
just test
```

### Checking Code Quality

```bash
# Run clippy on all tutorials
just clippy

# Check formatting
just fmt-check

# Run all CI checks
just ci
```

## Key Differences from Anchor

### 1. Explicit Type System
Star Frame uses explicit traits and types for compile-time safety:
- `StarFrameProgram` instead of `#[program]`
- `InstructionSet` for instruction routing
- `AccountSet` for account validation

### 2. Performance First
- Zero-copy deserialization with `Pod` and `Zeroable`
- Direct memory access with `data_mut()`
- Minimal compute unit usage

### 3. Validation as Types
- `AccountValidate` trait for custom validation logic
- `ValidatedAccount<T>` ensures validation before use
- Compile-time guarantees through type wrappers

### 4. Seed Management
- Separate seed structs with `GetSeeds`
- Reusable seed definitions
- Type-safe PDA derivation

## Learning Path

1. **Start with basic-0**: Understand the minimal structure
2. **Progress through basic-1 to basic-2**: Learn core concepts
3. **Study basic-3**: Master program composition
4. **Explore basic-4**: Handle complex scenarios
5. **Complete basic-5**: Build real-world patterns

Each tutorial builds on the previous ones, introducing new concepts gradually while reinforcing earlier lessons.

## Comparison with Anchor

| Tutorial | Anchor Concept | Star Frame Equivalent |
|----------|---------------|----------------------|
| basic-0 | `#[program]` module | `StarFrameProgram` derive |
| basic-1 | `#[account]` struct | `ProgramAccount` with traits |
| basic-2 | `has_one` constraint | `AccountValidate` trait |
| basic-3 | CPI with context | Star Frame CPI patterns |
| basic-4 | Seeds & bumps | `GetSeeds` trait |
| basic-5 | Complex state | State machines with validation |

## Resources

- [Star Frame Documentation](https://docs.rs/star_frame)
- [Star Frame vs Anchor Guide](../docs/star-frame-vs-anchor.md)
- [Tutorial Comparison](../docs/tutorial-comparison.md)
- [Example Programs](../example_programs/)
