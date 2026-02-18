# Star Frame vs Anchor

A practical comparison for developers migrating from or choosing between Star Frame and Anchor.

## TL;DR

| Aspect | Star Frame | Anchor |
|--------|-----------|--------|
| Compute Units | **60-93% less** | Baseline |
| Binary Size | **~50% smaller** | Baseline |
| Data Model | Zero-copy (bytemuck) | Borsh deserialization |
| Validation | Trait-based, composable | Attribute macros |
| PDA Seeds | Typed `GetSeeds` structs | `seeds = [...]` in attributes |
| Account Types | Composable modifiers | Flat types (`Account`, `Signer`) |
| Discriminant | Anchor-compatible (default) | 8-byte sighash |
| Dynamic Data | First-class unsized type system | Borsh Vec/String |
| SPL Tokens | `star_frame_spl` crate | Built-in |
| IDL Format | Codama | Anchor IDL |
| Maturity | Newer, production-tested | Battle-tested, large ecosystem |

## Concept Mapping

### Program Definition

**Anchor:**
```rust
declare_id!("MyProgram...");

#[program]
pub mod my_program {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.counter.count = amount;
        Ok(())
    }
}
```

**Star Frame:**
```rust
#[derive(StarFrameProgram)]
#[program(instruction_set = MyInstructionSet, id = "MyProgram...")]
pub struct MyProgram;

#[derive(InstructionSet)]
pub enum MyInstructionSet {
    Initialize(Initialize),
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(run)]
    pub amount: u64,
}

#[star_frame_instruction]
fn Initialize(accounts: &mut InitializeAccounts, amount: u64) -> Result<()> {
    accounts.counter.data_mut()?.count = amount;
    Ok(())
}
```

**Key difference:** Star Frame separates the instruction set enum from instruction implementations. More boilerplate, but the type system catches more errors.

### Account Definitions

**Anchor:**
```rust
#[account]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}
```

**Star Frame:**
```rust
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}
```

**Key difference:** Star Frame uses bytemuck zero-copy by default. No deserialization — data is accessed directly from the account buffer. This is the main source of the compute unit savings.

### Account Constraints

**Anchor:**
```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Counter::INIT_SPACE,
        seeds = [b"counter", payer.key().as_ref()],
        bump,
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}
```

**Star Frame:**
```rust
#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub payer: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(CounterSeeds { authority: *self.payer.pubkey() }),
    ))]
    pub counter: Init<Seeded<Account<Counter>>>,
    pub system_program: Program<System>,
}
```

**Key differences:**
- No lifetimes (`'info`) — Star Frame handles lifetimes internally
- No manual `space` calculation — derived from the type automatically
- Seeds are typed structs instead of inline byte arrays
- Modifiers compose: `Init<Seeded<Account<T>>>` vs flat `#[account(init, seeds, bump)]`
- The `payer` is tagged with `#[validate(funder)]` instead of `payer = payer` on init

### Account Access

**Anchor:**
```rust
pub fn increment(ctx: Context<Increment>) -> Result<()> {
    ctx.accounts.counter.count += 1;  // Direct field access after deserialization
    Ok(())
}
```

**Star Frame:**
```rust
#[star_frame_instruction]
fn Increment(accounts: &mut IncrementAccounts) -> Result<()> {
    let mut counter = accounts.counter.data_mut()?;  // Returns a wrapper
    counter.count += 1;
    Ok(())
}
```

**Key difference:** Star Frame requires explicit `data()` / `data_mut()` calls. This is because data is zero-copy — you're borrowing directly from the account buffer. Anchor deserializes into an owned struct.

### PDA Seeds

**Anchor:**
```rust
seeds = [b"counter", authority.key().as_ref()]
```

**Star Frame:**
```rust
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"counter")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}
```

**Advantage:** Star Frame seeds are typed structs. You can reuse them, pass them around, and the compiler ensures seed types are correct. No string/byte literal mistakes.

### Error Handling

**Anchor:**
```rust
#[error_code]
pub enum MyError {
    #[msg("Invalid amount")]
    InvalidAmount,
}
// Usage: err!(MyError::InvalidAmount)
// Or: require!(amount > 0, MyError::InvalidAmount)
```

**Star Frame:**
```rust
#[star_frame_error]
pub enum MyError {
    #[msg("Invalid amount")]
    InvalidAmount,
}
// Usage: bail!(MyError::InvalidAmount);
// Or: ensure!(amount > 0, MyError::InvalidAmount);
```

Essentially identical. Star Frame uses `ensure!`/`bail!` instead of `require!`/`err!`.

### CPI

**Anchor:**
```rust
let cpi_accounts = Transfer {
    from: ctx.accounts.source.to_account_info(),
    to: ctx.accounts.dest.to_account_info(),
    authority: ctx.accounts.authority.to_account_info(),
};
let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
token::transfer(cpi_ctx, amount)?;
```

**Star Frame:**
```rust
Token::cpi(
    Transfer { amount },
    TransferCpiAccounts { source, destination, owner },
    None,
).invoke()?;
```

**Advantage:** Star Frame's CPI is more concise. The CPI accounts struct is auto-generated and type-safe.

## Performance Deep Dive

### Why Star Frame is Faster

1. **Zero-copy accounts:** Anchor deserializes every account (Borsh) on entry and serializes on exit. Star Frame reads/writes directly to account buffers via bytemuck.

2. **Minimal validation overhead:** Star Frame's trait-based validation compiles down to very few instructions. Anchor's macro-generated validation has more overhead.

3. **No serialization on exit:** Anchor re-serializes modified accounts back to Borsh after instruction execution. Star Frame's writes are already in-place.

4. **Pinocchio runtime:** Star Frame uses Pinocchio instead of `solana-program`, which is significantly lighter.

### Benchmark Highlights

For 8 typed accounts (sized, read-only):
- **Star Frame:** 480 CU
- **Anchor:** 3,387 CU
- **Savings:** 85.8%

For 8 token accounts:
- **Star Frame:** 662 CU  
- **Anchor:** 8,550 CU
- **Savings:** 92.3%

Account initialization (8 accounts):
- **Star Frame:** 13,178 CU
- **Anchor:** 34,723 CU
- **Savings:** 62.1%

## Migration Guide (Anchor → Star Frame)

### Step 1: Replace dependencies

```diff
- anchor-lang = "0.31"
- anchor-spl = "0.31"
+ star_frame = "0.29"
+ star_frame_spl = { version = "0.29", features = ["token"] }
+ bytemuck = { version = "1.22", features = ["derive"] }
```

### Step 2: Convert program definition

```diff
- declare_id!("MyProg...");
- #[program]
- pub mod my_program { ... }
+ #[derive(StarFrameProgram)]
+ #[program(instruction_set = MyInstructionSet, id = "MyProg...")]
+ pub struct MyProgram;
+
+ #[derive(InstructionSet)]
+ pub enum MyInstructionSet { ... }
```

### Step 3: Convert account structs

```diff
- #[account]
+ #[zero_copy(pod)]
+ #[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
  pub struct MyAccount {
      pub field: u64,
-     pub name: String,      // Must become fixed-size
+     pub name: [u8; 32],    // Fixed-size byte array
+     pub name_len: u8,
  }
```

### Step 4: Convert instruction handlers

```diff
- pub fn my_instruction(ctx: Context<MyAccounts>, arg: u64) -> Result<()> {
-     ctx.accounts.data.field = arg;
-     Ok(())
- }
+ #[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
+ pub struct MyInstruction {
+     #[ix_args(run)]
+     pub arg: u64,
+ }
+
+ #[star_frame_instruction]
+ fn MyInstruction(accounts: &mut MyAccounts, arg: u64) -> Result<()> {
+     accounts.data.data_mut()?.field = arg;
+     Ok(())
+ }
```

### Step 5: Convert account sets

```diff
- #[derive(Accounts)]
- pub struct MyAccounts<'info> {
-     #[account(mut)]
-     pub signer: Signer<'info>,
-     #[account(mut, has_one = authority)]
-     pub data: Account<'info, MyData>,
- }
+ #[derive(AccountSet)]
+ pub struct MyAccounts {
+     pub signer: Signer,
+     #[validate(arg = CheckAuthority(*self.signer.pubkey()))]
+     pub data: Mut<ValidatedAccount<MyData>>,
+ }
```

### Step 6: Update client code

The generated `ClientAccounts` structs work similarly to Anchor's, but the IDL format is different (Codama vs Anchor IDL). You'll need to regenerate clients using Codama tooling.

## When to Choose Star Frame

**Choose Star Frame when:**
- Compute units matter (complex programs, many accounts)
- Binary size is a concern
- You want composable, type-safe validation
- You need dynamic account data (unsized types)
- You're starting a new project

**Choose Anchor when:**
- You need the largest ecosystem of examples/libraries
- Your team already knows Anchor well
- Compute units aren't a bottleneck
- You want the most battle-tested option

**Both are production-ready.** Star Frame is newer but has been used in production programs with complex state management. The choice depends on your priorities.
