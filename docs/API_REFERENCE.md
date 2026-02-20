# Star Frame API Reference

> **Note:** This reference is manually maintained. Always verify against the source code in
> `star_frame_proc/src/lib.rs` and `star_frame/src/` for the authoritative API. Attribute names
> and signatures may drift between releases.
>
> <!-- TODO: Add CI step to detect doc/source drift (e.g., grep-based attribute inventory check) -->

Quick reference for all key traits, types, macros, and their usage.

## Table of Contents

- [Program Definition](#program-definition)
- [Instruction System](#instruction-system)
- [Account Types](#account-types)
- [Account Modifiers](#account-modifiers)
- [Account Set Attributes](#account-set-attributes)
- [PDA / Seeds](#pda--seeds)
- [Data Types](#data-types)
- [Error Handling](#error-handling)
- [CPI (Cross-Program Invocation)](#cpi)
- [SPL Token Integration](#spl-token-integration)
- [Unsized Types](#unsized-types)
- [Context](#context)
- [IDL Generation](#idl-generation)

---

## Program Definition

### `#[derive(StarFrameProgram)]`

Defines the main program struct and generates the Solana entrypoint.

```rust
#[derive(StarFrameProgram)]
#[program(
    instruction_set = MyInstructionSet,   // Required: enum implementing InstructionSet
    id = "ProgramId...",                  // Required: base58 program ID
    errors = MyErrorEnum,                 // Optional: custom error enum
    no_entrypoint,                        // Optional: skip entrypoint generation (for libs)
)]
pub struct MyProgram;
```

**Generated items:**
- `StarFrameDeclaredProgram` — type alias for your program
- `ID: Pubkey` — const program ID
- `id() -> Pubkey` — returns program ID
- `check_id(&Pubkey) -> bool` — checks if pubkey matches

**Trait: `StarFrameProgram`**
```rust
pub trait StarFrameProgram {
    type InstructionSet: InstructionSet;
    type AccountDiscriminant: Pod + Eq;
    const ID: Pubkey;
    fn entrypoint(program_id, accounts, data) -> ProgramResult;
    fn handle_error(error: Error) -> ProgramError;
}
```

---

## Instruction System

### `#[derive(InstructionSet)]`

Defines the dispatch enum for all program instructions.

```rust
#[derive(InstructionSet)]
pub enum MyInstructionSet {
    DoSomething(DoSomething),
    DoAnother(DoAnother),
}
```

**Discriminants:** Default is Anchor-compatible sighash: `sha256("global:<snake_case_variant>")[..8]`.

### `#[derive(InstructionArgs)]`

Splits instruction data into lifecycle phases.

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct MyInstruction {
    #[ix_args(run)]          // Passed to process() by value
    pub amount: u64,
    #[ix_args(&run)]         // Passed to process() by reference
    pub config: Option<u8>,
    #[ix_args(&validate)]    // Passed to validate_accounts() by reference
    pub expected_owner: Pubkey,
    #[ix_args(decode)]       // Passed to decode_accounts()
    pub num_accounts: u8,
    #[ix_args(cleanup)]      // Passed to cleanup_accounts()
    pub should_close: bool,
}
```

**No-arg instructions:**
```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct SimpleInstruction;
```

### `#[star_frame_instruction]`

Macro that implements `StarFrameInstruction` from a function.

```rust
#[star_frame_instruction]
fn MyInstruction(accounts: &mut MyAccounts, amount: u64, config: &Option<u8>) -> Result<()> {
    // Logic here
    Ok(())
}
```

- Function name must match the instruction struct name
- First param is always `&mut AccountSet`
- Remaining params match the `run` args from `InstructionArgs`
- Return type must be `Result<T>` where `T: NoUninit` (or `Result<()>`)

**With return data:**
```rust
#[star_frame_instruction]
fn Count(accounts: &mut CountAccounts, amount: u64) -> Result<u64> {
    let new_count = accounts.counter.data()?.count + amount;
    accounts.counter.data_mut()?.count = new_count;
    Ok(new_count)  // Set as Solana return data
}
```

---

## Account Types

### `AccountInfo`

Raw unchecked account. No validation. Use when you need maximum flexibility.

```rust
pub account: AccountInfo,
```

### `SystemAccount`

Validates the account is owned by the System Program.

```rust
pub user: SystemAccount,
pub user_signer: Signer<SystemAccount>,
pub user_writable: Signer<Mut<SystemAccount>>,
```

### `Account<T>`

Typed program account. Validates owner matches `T::OwnerProgram` and discriminant is correct.

```rust
pub counter: Account<CounterAccount>,          // Read-only
pub counter: Mut<Account<CounterAccount>>,     // Writable

// Access data:
let data = accounts.counter.data()?;           // SharedWrapper<CounterAccount>
let data = accounts.counter.data_mut()?;       // ExclusiveWrapper<CounterAccount>
**accounts.counter.data_mut()? = CounterAccount { ... };  // Overwrite entirely
```

### `ValidatedAccount<T>`

Like `Account<T>` but also calls `AccountValidate::validate_account` during validation.

```rust
// Define validation:
pub struct CheckAuthority(pub Pubkey);
impl AccountValidate<CheckAuthority> for MyAccount {
    fn validate_account(self_ref: &Self::Ptr, arg: CheckAuthority) -> Result<()> {
        ensure!(self_ref.authority == arg.0, ProgramError::IncorrectAuthority);
        Ok(())
    }
}

// Use in account set:
#[validate(arg = CheckAuthority(*self.signer.pubkey()))]
pub account: ValidatedAccount<MyAccount>,

// Tuple validation (multiple checks):
#[validate(arg = (CheckAuthority(*self.signer.pubkey()), CheckActive))]
pub account: ValidatedAccount<MyAccount>,
```

### `Program<T>`

Validates account pubkey equals `T::ID`.

```rust
pub system_program: Program<System>,
pub token_program: Program<Token>,
```

In client accounts, `Program<T>` fields become `Option<Pubkey>` defaulting to `T::ID`.

### `MintAccount` / `TokenAccount` (star_frame_spl)

SPL token account types with built-in validation.

```rust
use star_frame_spl::token::state::{MintAccount, TokenAccount};

pub mint: MintAccount,
#[validate(arg = ValidateToken { mint: Some(*self.mint.key_for()), owner: Some(*self.user.pubkey()) })]
pub token_account: TokenAccount,
```

---

## Account Modifiers

### `Signer<T>`

Validates the account is a transaction signer.

```rust
pub authority: Signer,                    // Signer<AccountInfo>
pub authority: Signer<SystemAccount>,     // Signed system account
pub authority: Signer<Mut<SystemAccount>>,// Signed + writable
```

### `Mut<T>`

Validates the account is marked writable.

```rust
pub counter: Mut<Account<MyAccount>>,
```

### `Init<T>`

Creates the account during the validation phase.

```rust
// Basic creation (keypair-signed account):
#[validate(arg = Create(()))]
pub account: Init<Signer<Account<MyAccount>>>,

// PDA creation:
#[validate(arg = (Create(()), Seeds(MySeeds { ... })))]
pub account: Init<Seeded<Account<MyAccount>>>,

// Create only if doesn't exist:
#[validate(arg = (CreateIfNeeded(()), Seeds(MySeeds { ... })))]
pub account: Init<Seeded<Account<MyAccount>>>,
```

**`Init` requires:**
- A `funder` tagged account in the account set (for paying rent)
- `System` program in the account set (for account creation CPI)

**Check if init happened:**
```rust
if accounts.my_account.needed_init() {
    // First-time setup logic
}
```

### `Seeded<T>`

Validates account address matches the PDA derived from seeds.

```rust
// With Seeds (derives bump automatically):
#[validate(arg = Seeds(MySeeds { key: *self.authority.pubkey() }))]
pub account: Seeded<Account<MyAccount>>,

// With SeedsWithBump (known bump, skips find_program_address):
#[validate(arg = SeedsWithBump { seeds: MySeeds { ... }, bump: 254 })]
pub account: Seeded<Account<MyAccount>>,

// Access seeds after validation:
let seeds = accounts.account.access_seeds();  // &SeedsWithBump<MySeeds>
let bump = seeds.bump;
```

---

## Account Set Attributes

### `#[validate(...)]`

| Attribute | Description |
|-----------|-------------|
| `#[validate(funder)]` | Cache as rent funder in Context |
| `#[validate(recipient)]` | Cache as rent recipient in Context |
| `#[validate(arg = Expr)]` | Pass validation argument to account type |
| `#[validate(address = &expr)]` | Validate pubkey matches expression |
| `#[validate(extra_validation = expr)]` | Run custom validation expression |
| `#[validate(skip)]` | Skip validation for this field |

### `#[cleanup(...)]`

| Attribute | Description |
|-----------|-------------|
| `#[cleanup(arg = CloseAccount(()))]` | Close account: resizes to discriminant size, fills with `0xFF`, drains lamports to cached recipient |
| `#[cleanup(arg = CloseAccount(&recipient))]` | Close account: resizes to discriminant size, fills with `0xFF`, drains lamports to explicit recipient |
| `#[cleanup(arg = NormalizeRent(()))]` | Adjust rent to match current data size |
| `#[cleanup(arg = RefundRent(()))]` | Refund excess rent to cached recipient |

### `#[account_set(...)]`

| Attribute | Description |
|-----------|-------------|
| `#[account_set(skip = default_expr)]` | Skip field during account set processing; initialized with the provided default value |

### Struct-level validation

```rust
#[derive(AccountSet)]
#[validate(extra_validation = self.custom_check())]
pub struct MyAccounts { ... }

impl MyAccounts {
    fn custom_check(&self) -> Result<()> { ... }
}
```

---

## PDA / Seeds

### `#[derive(GetSeeds)]`

```rust
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"MY_SEED")]   // Optional constant prefix
pub struct MySeeds {
    pub key1: Pubkey,      // 32 bytes
    pub key2: u64,         // 8 bytes (little-endian)
}
// Produces: [b"MY_SEED", key1_bytes, key2_bytes, &[/*bump*/]]
```

**Link seeds to account type:**
```rust
#[derive(ProgramAccount)]
#[program_account(seeds = MySeeds)]
pub struct MyAccount { ... }
```

**Manual `GetSeeds` implementation:**
```rust
impl GetSeeds for CustomSeeds {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![b"PREFIX", self.key.seed(), &[]]
    }
}
```

### `Seed` trait

Auto-implemented for any `NoUninit` type (via `bytemuck::bytes_of`). Custom implementations:
```rust
// Pubkey: 32 bytes
// u64: 8 bytes (little-endian)
// [u8; N]: N bytes
```

---

## Data Types

### `#[zero_copy(pod)]`

Convenience macro that expands to: `#[derive(Align1, Pod, Zeroable, Copy, Clone)]` + `#[repr(C, packed)]`

```rust
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
pub struct MyAccount {
    pub field1: Pubkey,    // 32 bytes
    pub field2: u64,       // 8 bytes
    pub field3: u8,        // 1 byte
    // Total: 41 bytes + 8 byte discriminant = 49 bytes on-chain
}
```

**Allowed field types:** `u8`, `u16`, `u32`, `u64`, `u128`, `i8`-`i128`, `Pubkey`, `[T; N]`, other `Pod` types.

**Not allowed:** `bool` (use `u8`), `String`, `Vec`, `Option`, enums (unless manually implemented).

### `#[derive(ProgramAccount)]`

Generates owner validation and discriminant.

```rust
#[derive(ProgramAccount)]
#[program_account(seeds = MySeeds)]  // Optional: links seeds type
pub struct MyAccount { ... }
```

### `KeyFor<T>`

Type-safe pubkey reference to an account type:

```rust
pub struct MyAccount {
    pub mint: KeyFor<MintAccount>,  // Pubkey that should reference a MintAccount
}

// Usage:
let key: &Pubkey = my_account.mint.pubkey();
let key_for: &KeyFor<MintAccount> = &my_account.mint;
```

### `PackedValue<T>`

Wrapper for values in packed structs (handles alignment):

```rust
pub struct MyAccount {
    pub amount: PackedValue<u64>,
}
// Access: my_account.amount.0 (the inner value)
```

### `UnitVal<T, U>` / `create_unit_system!`

Type-safe units for domain values:

```rust
create_unit_system!(pub struct MyUnits<Currency>);
type Price = UnitVal<PackedValue<u64>, Currency>;
```

---

## Error Handling

### `#[star_frame_error]`

```rust
#[star_frame_error]
pub enum MyError {
    #[msg("Something went wrong")]
    BadThing,
    #[msg("Value {0} is invalid")]
    InvalidValue,
}
```

### Macros

```rust
ensure!(condition, MyError::BadThing);              // Return Err if false
ensure!(condition, ProgramError::Custom(0), "msg"); // With message
ensure_eq!(a, b, MyError::Mismatch);                // Equality check
ensure_ne!(a, b, MyError::Duplicate);               // Inequality check
bail!(MyError::BadThing);                            // Always return Err
bail!(MyError::BadThing, "with context: {}", val);   // With formatted message
error!(MyError::BadThing);                           // Create Error without returning
```

---

## CPI

### Cross-Program Invocation

```rust
use star_frame_spl::token::{Token, instructions::{Transfer, TransferCpiAccounts}};

// Unsigned CPI
Token::cpi(
    Transfer { amount: 100 },
    TransferCpiAccounts {
        source: *source_info,
        destination: *dest_info,
        owner: *owner_info,
    },
    None,
).invoke()?;

// Signed CPI (PDA signer)
Token::cpi(
    Transfer { amount: 100 },
    TransferCpiAccounts { source, destination, owner },
    None,
).invoke_signed(&[&signer_seeds])?;
```

---

## SPL Token Integration

Add `star_frame_spl` to your dependencies:

```toml
star_frame_spl = { version = "0.29", features = ["token"] }
```

### Key Types

```rust
use star_frame_spl::token::{Token, state::{MintAccount, TokenAccount}};
use star_frame_spl::associated_token::state::AssociatedTokenAccount;

// In account sets:
pub mint: MintAccount,
#[validate(arg = ValidateToken { mint: Some(*self.mint.key_for()), owner: Some(*self.user.pubkey()) })]
pub token_account: Mut<TokenAccount>,
pub ata: AssociatedTokenAccount,
```

### Token Instructions

```rust
use star_frame_spl::token::instructions::{Transfer, TransferCpiAccounts, MintTo, MintToCpiAccounts};

Token::cpi(Transfer { amount }, TransferCpiAccounts { source, destination, owner }, None).invoke()?;
Token::cpi(MintTo { amount }, MintToCpiAccounts { mint, account, owner }, None).invoke_signed(&[&seeds])?;
```

---

## Unsized Types

For variable-length account data:

```rust
#[unsized_type(program_account, seeds = MySeeds)]
pub struct DynamicAccount {
    pub fixed_field: Pubkey,
    #[unsized_start]             // Everything after this is dynamically-sized
    pub items: List<MyItem>,     // Dynamic-length list
    pub lookup: Map<Pubkey, u64>,// Dynamic-length map
}
```

**Available unsized collections:** `List<T>`, `Map<K, V>`, `UnsizedMap<K, V>`

**Mutable access with `#[unsized_impl]`:**

```rust
#[unsized_impl]
impl DynamicAccount {
    pub fn add_item(&mut self, item: MyItem) -> Result<()> {
        self.items().insert(self.items.len(), item)?;
        Ok(())
    }
}
```

**Cleanup for resizable accounts:**
```rust
#[cleanup(arg = NormalizeRent(()))]  // Auto-adjusts rent after resize
pub dynamic_account: Mut<Account<DynamicAccount>>,
```

---

## Context

The `Context` struct is passed through the instruction lifecycle:

```rust
pub struct Context {
    fn current_program_id(&self) -> &Pubkey;
    fn get_rent(&self) -> Result<Rent>;      // Cached sysvar
    fn get_clock(&self) -> Result<Clock>;    // Cached sysvar
    fn get_funder(&self) -> Option<&dyn CanFundRent>;
    fn get_recipient(&self) -> Option<&dyn CanAddLamports>;
    fn set_funder(&mut self, funder: Box<dyn CanFundRent>);
    fn set_recipient(&mut self, recipient: Box<dyn CanAddLamports>);
}
```

Available in `process()` via the `star_frame_instruction` macro (as `_ctx: &mut Context` — add it as a parameter if needed):

```rust
#[star_frame_instruction]
fn MyIx(accounts: &mut MyAccounts) -> Result<()> {
    // ctx is available as _ctx if you add it:
    // fn MyIx(accounts: &mut MyAccounts, _ctx: &mut Context) -> Result<()>
    Ok(())
}
```

---

## IDL Generation

```rust
// In your test file:
#[cfg(feature = "idl")]
#[test]
fn generate_idl() -> Result<()> {
    let idl = StarFrameDeclaredProgram::program_to_idl()?;
    let codama_idl: ProgramNode = idl.try_into()?;
    let idl_json = codama_idl.to_json()?;
    std::fs::write("idl.json", &idl_json)?;
    Ok(())
}
```

Run: `cargo test --features idl -- generate_idl`

### IDL Hints

```rust
// For seed paths in IDL generation:
#[idl(arg = Seeds(FindMySeeds { key: seed_path("authority") }))]
pub account: Seeded<Account<MyAccount>>,

// For ATA seeds:
#[idl(arg = Seeds(FindAtaSeeds { mint: seed_path("mint"), wallet: seed_path("market") }))]
pub ata: AssociatedTokenAccount,
```
