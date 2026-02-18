# Star Frame Pitfalls & Gotchas

Things that will bite you. Read this before going to production.

---

## 1. Edition 2024 / SBF Toolchain Conflicts

**Problem:** Rust Edition 2024 crates (like the `az` crate and its dependents) don't compile with the Solana SBF toolchain.

**Symptoms:**
- Mysterious compilation errors when building with `cargo build-sbf`
- Errors mentioning `unsafe_op_in_unsafe_fn` or other edition-related lints
- Dependencies that work fine in tests but fail for SBF target

**Solution:** Pin dependencies to versions that use Edition 2021 or earlier. Check your `Cargo.lock` for any crate using `edition = "2024"`. Use `cargo tree` to find what's pulling them in.

```toml
# Example: pin a problematic transitive dependency
[patch.crates-io]
az = "=1.2.0"  # Last version using edition 2021
```

---

## 2. Account Mutability: Signer vs Mut<Signer<SystemAccount>>

**Problem:** If you need to modify an account's lamports (e.g., as a funder), it must be both `Signer` AND `Mut`. Just `Signer` isn't enough.

**Wrong:**
```rust
#[validate(funder)]
pub authority: Signer<SystemAccount>,  // ❌ Can't debit lamports — not writable!
```

**Right:**
```rust
#[validate(funder)]
pub authority: Signer<Mut<SystemAccount>>,  // ✅ Signed + writable
```

**Rule of thumb:** If an account is paying for anything (rent, transfers), it needs `Mut`. Always use `Signer<Mut<SystemAccount>>` for funders.

---

## 3. The CanInitSeeds Pattern

**Problem:** `Init<Seeded<Account<T>>>` requires seeds to be set before the account can be created. The validation argument must provide both `Create`/`CreateIfNeeded` AND `Seeds`.

**Wrong:**
```rust
#[validate(arg = Create(()))]  // ❌ Missing seeds!
pub account: Init<Seeded<Account<MyAccount>>>,
```

**Right:**
```rust
#[validate(arg = (
    Create(()),
    Seeds(MySeeds { key: *self.authority.pubkey() }),
))]
pub account: Init<Seeded<Account<MyAccount>>>,
```

The tuple `(Create(()), Seeds(...))` distributes: `Create` goes to `Init`, `Seeds` goes to `Seeded`.

---

## 4. Missing System Program for Init

**Problem:** `Init` performs a CPI to the System Program to create accounts. If you forget to include it, you get a runtime error.

**Wrong:**
```rust
#[derive(AccountSet)]
pub struct MyAccounts {
    #[validate(funder)]
    pub payer: Signer<Mut<SystemAccount>>,
    #[validate(arg = Create(()))]
    pub new_account: Init<Signer<Account<MyData>>>,
    // ❌ Missing system_program!
}
```

**Right:**
```rust
#[derive(AccountSet)]
pub struct MyAccounts {
    #[validate(funder)]
    pub payer: Signer<Mut<SystemAccount>>,
    #[validate(arg = Create(()))]
    pub new_account: Init<Signer<Account<MyData>>>,
    pub system_program: Program<System>,  // ✅
}
```

---

## 5. Bool in Zero-Copy Types

**Problem:** `bytemuck::Pod` doesn't allow `bool` because bit patterns other than 0/1 would be undefined behavior.

**Wrong:**
```rust
#[zero_copy(pod)]
pub struct MyAccount {
    pub is_active: bool,  // ❌ Won't compile
}
```

**Right:**
```rust
#[zero_copy(pod)]
pub struct MyAccount {
    pub is_active: u8,  // ✅ Use 0/1
}
```

---

## 6. Account Field Order = Transaction Account Order

**Problem:** Fields in your `AccountSet` struct are decoded in order from the transaction's accounts array. If the client passes accounts in the wrong order, you get wrong accounts or decode errors.

```rust
#[derive(AccountSet)]
pub struct MyAccounts {
    pub authority: Signer,        // accounts[0]
    pub counter: Mut<Account<T>>, // accounts[1]
    pub system_program: Program<System>, // accounts[2]
}
```

Your client must pass accounts in this exact order. The generated `ClientAccounts` struct handles this automatically, but if building instructions manually, watch the order.

---

## 7. data_mut() on Non-Writable Accounts

**Problem:** Calling `data_mut()` on an account that isn't wrapped in `Mut<...>` will panic or return an error at runtime.

```rust
pub counter: Account<CounterAccount>,  // Read-only

// In process():
accounts.counter.data_mut()?;  // ❌ Runtime error: "not writable"
```

**Fix:** Use `Mut<Account<CounterAccount>>` if you need to write.

---

## 8. Forgetting #[validate(funder)] or #[validate(recipient)]

**Problem:** `Init` and `CloseAccount` look for cached funder/recipient in the `Context`. If you forget the tag, you get runtime errors like "Missing `funder` in cache".

```rust
// For Init:
#[validate(funder)]  // Don't forget this!
pub payer: Signer<Mut<SystemAccount>>,

// For CloseAccount:
#[validate(recipient)]  // Don't forget this!
pub funds_to: Mut<SystemAccount>,
```

**Alternative:** Pass the funder/recipient explicitly:
```rust
#[cleanup(arg = CloseAccount(&self.funds_to))]  // Explicit reference
pub account: Mut<Account<MyAccount>>,
```

---

## 9. Unsized Type Rent Management

**Problem:** When using unsized types (dynamic lists, maps), account size changes during execution. If you don't normalize rent, the account may become rent-exempt-ineligible.

**Solution:** Always use `NormalizeRent` cleanup for unsized accounts:

```rust
#[cleanup(arg = NormalizeRent(()))]
pub dynamic_account: Mut<Account<DynamicData>>,
```

This adjusts lamports up or down to match the current data size.

---

## 10. Discriminant Collisions

**Problem:** If you rename an instruction variant, the discriminant changes (it's derived from the name). Existing transactions with the old discriminant will fail.

```rust
// Before:
pub enum MyInstructions {
    DoThing(DoThing),  // discriminant = sighash("global:do_thing")
}

// After rename:
pub enum MyInstructions {
    PerformAction(DoThing),  // discriminant = sighash("global:perform_action") — DIFFERENT!
}
```

**Solution:** Once deployed, never rename instruction variants. Or use explicit discriminants:
```rust
#[derive(InstructionSet)]
pub enum MyInstructions {
    #[discriminant(sighash("global:do_thing"))]  // Pin the discriminant
    PerformAction(DoThing),
}
```

---

## 11. Validation Order Dependencies

**Problem:** `#[validate(arg = ...)]` expressions can reference `self.other_field`, but only fields that appear *before* the current field are guaranteed to be validated.

```rust
#[derive(AccountSet)]
pub struct MyAccounts {
    pub authority: Signer,
    // ✅ Can reference authority (declared above)
    #[validate(arg = Seeds(MySeeds { key: *self.authority.pubkey() }))]
    pub account: Seeded<Account<MyData>>,
}
```

All accounts are decoded first (in order), then validated (in order). You can reference any field in validation expressions since they're all decoded by that point. But understand that validation of earlier fields runs first.

---

## 12. CreateIfNeeded vs Create

**Problem:** Using `Create(())` when the account might already exist causes the instruction to fail. Use `CreateIfNeeded(())` for idempotent initialization.

```rust
// Will fail if account exists:
#[validate(arg = (Create(()), Seeds(...)))]
pub account: Init<Seeded<Account<T>>>,

// Safe for re-runs:
#[validate(arg = (CreateIfNeeded(()), Seeds(...)))]
pub account: Init<Seeded<Account<T>>>,
```

Check `accounts.my_account.needed_init()` to know if creation actually happened.

---

## 13. Packed Struct Alignment

**Problem:** All zero-copy account structs must be `#[repr(C, packed)]` (1-byte aligned) because Solana accounts don't guarantee alignment. The `#[zero_copy(pod)]` macro handles this, but if you're doing it manually, don't forget `Align1`.

**Wrong:**
```rust
#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]  // ❌ May have padding, won't work on-chain
pub struct MyData { ... }
```

**Right:**
```rust
#[derive(Align1, Pod, Zeroable, Copy, Clone)]
#[repr(C, packed)]  // ✅ No padding, 1-byte aligned
pub struct MyData { ... }
```

Or just use `#[zero_copy(pod)]` which does this automatically.

---

## 14. test_helpers Feature Required for Tests

**Problem:** Running `cargo test` without the `test_helpers` feature gives a compile error.

```bash
cargo test  # ❌ compile_error!("You must enable the `test_helpers` feature")
cargo test --features test_helpers  # ✅
```

---

## 15. Closing Accounts Writes 0xFF to Discriminant

When you close an account with `CloseAccount`, Star Frame writes `0xFF` bytes to the discriminant area and zeroes the lamports. This means:
- The account data isn't fully zeroed (just the discriminant is invalidated)
- Any code checking `data == [0; N]` to detect closed accounts won't work
- Check the discriminant or lamports instead
