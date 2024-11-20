# Star Frame SPL Brainstorming

- Need some sort of account set type that supports weird serialization impls
- Need some sort of account set type that supports borsh stuff


- Token accounts and mints are their own account sets.

```rust
pub struct SomeAccounts<'info> {
    pub token_accounts: Mut<TokenAccount<'info>>,
    pub token_accounts: Mut<MintAccount<'info>>,
    pub some_other_account: DataAccount<'info, SomeThing>,
}
```

as opposed to

```rust
pub struct SomeAccounts<'info> {
    pub some_other_account: DataAccount<'info, TokenAccount>,
}
```

Unknowns:

- How to get ATAs to work nicely

```rust
#[derive(AccountSet)]
pub struct SomeAccounts<'info> {
    #[validate(arg = CreateIfNeeded(InitATA {
        mint: token_accounts.mint,
        owner: token_accounts.owner,
    }))]
    pub token_account: Init<TokenAccount<'info>>,
    #[account_set(program)]
    pub associated_token_program: Program<'info, AssociatedTokenProgram>,
    #[account_set(program)]
    pub token_program: Program<'info, TokenProgram>,
    #[account_set(program)]
    pub system_program: Program<'info, SystemProgram>,
}
```

```rust
#[derive(AccountSet)]
pub struct SomeAccounts<'info> {
    #[validate(arg = AssociatedTokenAccount {
        mint: token_accounts.mint,
        owner: token_accounts.owner,
    })]
    pub token_account: TokenAccount<'info>,
}
```

- How to get normal tokens to work nicely

```rust
#[derive(AccountSet)]
pub struct SomeAccounts<'info> {
    #[validate(arg = CreateIfNeeded(InitATA {
        mint: token_accounts.mint,
        owner: token_accounts.owner,
    }))]
    pub token_account: Init<Seeded<TokenAccount<'info>, SomeSeeds, SomeProgram>>,
    #[account_set(program)]
    pub associated_token_program: Program<'info, AssociatedTokenProgram>,
    #[account_set(program)]
    pub token_program: Program<'info, TokenProgram>,
    #[account_set(program)]
    pub system_program: Program<'info, SystemProgram>,
}
```

```rust
#[derive(AccountSet)]
pub struct SomeAccounts<'info> {
    #[validate(arg = AssociatedTokenAccount {
        mint: token_accounts.mint,
        owner: token_accounts.owner,
    })]
    pub token_account: TokenAccount<'info>,
}
```

