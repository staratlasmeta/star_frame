# Production Learnings with Star Frame

Practical lessons from building and deploying a production Solana program with Star Frame. These are patterns, anti-patterns, and insights that only emerge from real usage.

---

## 1. Start with the Counter Example, Then Graduate

The `simple_counter` example is the right starting point. But real programs quickly need:
- Multiple instructions sharing account types
- Custom validation (not just owner/signer checks)
- PDA-derived accounts with complex seed structures
- Unsized types for dynamic data

Move to the `counter` (full) or `marketplace` examples once you understand the basics.

## 2. Design Your Seeds First

PDA seed design is the most important architectural decision. Get it wrong and you'll need to migrate accounts.

**Good pattern:** Derive seeds from the relationships between entities.
```rust
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"user_config")]
pub struct UserConfigSeeds {
    pub user: Pubkey,
    pub config_type: u8,  // Discriminate between config variants
}
```

**Anti-pattern:** Overloading a single PDA with too many seeds.
```rust
// Too many seeds — hard to query, brittle
pub struct OverEngineeredSeeds {
    pub user: Pubkey,
    pub market: Pubkey,
    pub epoch: u64,
    pub variant: u8,
}
```

**Lesson:** Every seed field is something the client needs to provide. Keep seeds minimal — just enough to guarantee uniqueness.

## 3. Store the Bump

Always store the PDA bump in the account data:

```rust
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
pub struct MyAccount {
    pub bump: u8,
    // ... other fields
}
```

Then on subsequent instructions, pass `SeedsWithBump` instead of `Seeds` to skip the expensive `find_program_address` call:

```rust
#[validate(arg = SeedsWithBump { seeds: MySeeds { ... }, bump: stored_bump })]
pub account: Seeded<Account<MyAccount>>,
```

This saves ~1,500 CU per PDA validation.

## 4. Use ValidatedAccount for Business Logic

Don't put all your validation in the instruction body. Use `AccountValidate` to declare invariants:

```rust
impl AccountValidate<CheckMarketActive> for Market {
    fn validate_account(self_ref: &Self::Ptr, _arg: CheckMarketActive) -> Result<()> {
        ensure!(self_ref.is_active == 1, MyError::MarketClosed);
        Ok(())
    }
}
```

**Why?** Validation runs *before* your process function. If validation fails, you haven't done any work yet. This is both safer and cheaper than checking mid-instruction.

## 5. Unsized Types Are Powerful but Complex

The unsized type system (`List<T>`, `Map<K, V>`) handles dynamic data brilliantly but requires careful thinking:

- **Always use `NormalizeRent` cleanup** for accounts with unsized types
- **Binary search is your friend** — `List` supports `binary_search_by_key`
- **Insertions/removals resize the account** — rent changes need a funder
- **Think about iteration order** — Lists maintain insertion order; use sorted insertion for efficient lookups

```rust
#[unsized_type(program_account)]
pub struct OrderBook {
    pub version: u8,
    #[unsized_start]
    pub orders: List<Order>,  // Sorted by price for binary search
}

#[unsized_impl]
impl OrderBook {
    pub fn insert_order(&mut self, order: Order) -> Result<()> {
        let index = self.orders
            .binary_search_by_key(&order.price, |o| o.price)
            .unwrap_or_else(|i| i);
        self.orders().insert(index, order)?;
        Ok(())
    }
}
```

## 6. The Funder/Recipient Cache Pattern

Understanding the `Context` cache is essential:

```rust
#[derive(AccountSet)]
pub struct MyAccounts {
    #[validate(funder)]      // Cached: used by Init for rent funding
    pub payer: Signer<Mut<SystemAccount>>,
    
    #[validate(recipient)]   // Cached: used by CloseAccount for receiving lamports
    pub refund_to: Mut<SystemAccount>,
    
    // These use the cached funder/recipient automatically:
    #[validate(arg = (Create(()), Seeds(...)))]
    pub new_account: Init<Seeded<Account<Data>>>,
    
    #[cleanup(arg = CloseAccount(()))]      // Uses cached recipient
    pub old_account: Mut<Account<OldData>>,
    
    #[cleanup(arg = NormalizeRent(()))]      // Uses cached funder
    pub dynamic_account: Mut<Account<Dynamic>>,
}
```

**Lesson:** You can have one funder and one recipient cached at a time. For instructions that need multiple different funders/recipients, pass them explicitly:
```rust
#[cleanup(arg = CloseAccount(&self.specific_recipient))]
```

## 7. Account Wrapper Newtype Pattern

For accounts used across multiple instructions with different validation needs, create a newtype:

```rust
#[derive(AccountSet, Deref, DerefMut, Debug)]
pub struct WrappedMarket(#[single_account_set] Account<Market>);
```

This lets different instructions apply different validation to the same underlying account type. The `marketplace` example uses this pattern extensively.

## 8. Testing with Mollusk

Mollusk-SVM is the recommended test framework. Key patterns:

```rust
let mollusk = Mollusk::new(&MyProgram::ID, "my_program");
let mollusk = mollusk.with_context(HashMap::from_iter([
    (payer, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
    (account, SolanaAccount::new(0, 0, &System::ID)),
    mollusk_svm::program::keyed_account_for_system_program(),
]));
```

**Tip:** Use `SerializeAccount`/`DeserializeAccount` for asserting account state:
```rust
use star_frame::client::{SerializeAccount, DeserializeAccount};

let expected = MyAccount { count: 5, .. };
Check::account(&account_key)
    .data(&MyAccount::serialize_account(expected)?)
    .owner(&MyProgram::ID)
    .build()
```

**Tip:** For unsized types, use `ModifyOwned` for mutation in tests:
```rust
Market::modify_owned(&mut market_data, |market| {
    market.process_order(args, maker)?;
    Ok(())
})?;
```

## 9. Error Context Is Critical for Debugging

Star Frame's `.ctx()` extension is invaluable for debugging:

```rust
self.init_account(arg, seeds, ctx)
    .ctx("Failed to init counter account")?;

account_set.validate_accounts(args, ctx)
    .ctx("Validation failed for PlaceOrder")?;
```

On-chain, errors are logged via Pinocchio's logger. Off-chain (in tests), the full error chain is available. Always add context to operations that might fail.

## 10. Be Careful with data() Re-validation

`Account<T>::data()` and `data_mut()` re-validate the discriminant if the account is writable. This is a safety feature (another instruction could have modified the account via CPI), but it has a small CU cost.

**Optimization:** If you're reading the same account multiple times in one instruction, call `data()` once and hold the reference:

```rust
// Good: one validation
let data = accounts.my_account.data()?;
let x = data.field_a;
let y = data.field_b;

// Less good: validates twice
let x = accounts.my_account.data()?.field_a;
let y = accounts.my_account.data()?.field_b;
```

## 11. IDL Generation Is a Test

Keep IDL generation as a test, not a build step:

```rust
#[cfg(feature = "idl")]
#[test]
fn generate_idl() -> Result<()> {
    let idl = StarFrameDeclaredProgram::program_to_idl()?;
    let codama_idl: ProgramNode = idl.try_into()?;
    std::fs::write("idl.json", &codama_idl.to_json()?)?;
    Ok(())
}
```

Run it manually when you change the interface. Don't make it part of CI unless you want to verify IDL consistency (which is actually a good idea — compare against a checked-in IDL).

## 12. CPI Signer Seeds from Seeded Accounts

When your PDA needs to sign a CPI, reconstruct seeds from the stored data:

```rust
let seeds_with_bump = SeedsWithBump {
    seeds: MarketSeeds {
        currency: *self.currency.key_for(),
        market_token: *self.market_token.key_for(),
    },
    bump: market_data.bump,
};
let signer_seeds = seeds_with_bump.seeds_with_bump();

Token::cpi(transfer, cpi_accounts, None)
    .invoke_signed(&[signer_seeds.as_slice()])?;
```

**Lesson:** Store all seed components and the bump in the account data. You'll need them for CPI signing.

## 13. Feature Flags Strategy

```toml
[features]
default = []
idl = ["star_frame/idl"]
test_helpers = ["star_frame/test_helpers"]
```

- Keep `idl` behind a feature flag — it pulls in heavy dependencies not needed on-chain
- `test_helpers` is only for testing — never enable in production builds
- Use `aggressive_inline` only if you've benchmarked and confirmed it helps your specific program

## 14. The Lifecycle Matters

Understanding the exact order: Decode → Validate → Process → Cleanup.

- **Decode:** Accounts are consumed from the slice. No validation yet.
- **Validate:** Runs in field order. Seeds resolved, Init creates accounts, discriminants checked.
- **Process:** Your logic. All accounts are valid at this point.
- **Cleanup:** Rent normalization, account closing. Runs after your logic succeeds.

**Implication:** If your instruction fails in Process, cleanup doesn't run. Account creation (from Init) has already happened but the transaction will revert.
