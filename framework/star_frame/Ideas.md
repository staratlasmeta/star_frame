program trait 


Account set 



Program<FactionEnlistment>;

```rust
trait ToSeedBytes {
    fn to_seed_bytes(&self) -> Vec<u8>;
}
```

```rust
[
    Seed::Constant(CERTIFICATE_MINT),
    Seed::AccountSetPath(["starbase_and_starbase_player", "starbase", "key"]),
    Seed::AccountSetPath(["cargo_mint"]),
    Seed::AccountSetPath(["starbase_and_starbase_player", "starbase", "game_id"])
]
```

```rust
#[account(
    mut,
    seeds = [
        CERTIFICATE_MINT,
        starbase_and_starbase_player.starbase.as_ref().key.as_ref(),
        cargo_mint.key().as_ref(),
        &starbase_and_starbase_player.starbase.load()?.seq_id.to_le_bytes(),
        starbase_and_starbase_player.starbase.load()?.game_id.as_ref(),
    ],
    bump,
)]
mod stuff {
    
}
```

```swagger codegen
template <typename T, typename T1> 
auto compose(T a, T1 b) -> decltype(a + b) {
   return a+b;
}
```

```rust
/// Redeems a certificate for a given cargo
#[derive(Accounts, Derivative)]
#[derivative(Debug)]
pub struct RedeemCertificate<'info> {
    /// The starbase to create a certificate mint for
    pub starbase_and_starbase_player: StarbaseAndStarbasePlayer<'info>,

    /// The game, game state and profile accounts.
    pub game_accounts_and_profile: GameAndGameStateAndProfile<'info>,

    /// The mint of the cargo in question
    /// CHECK: Checked in constraints
    pub cargo_mint: UncheckedAccount<'info>,

    /// The cargo certificate mint
    /// CHECK: Checked in constraints, seeds, and token program
    #[account(
        mut,
        seeds = [
            CERTIFICATE_MINT,
            Pubkey: starbase_and_starbase_player.starbase.as_ref().key,
            Pubkey: cargo_mint.key(),
            u8: &starbase_and_starbase_player.starbase.load()?.seq_id,
            Pubkey: starbase_and_starbase_player.starbase.load()?.game_id,
        ],
        bump,
    )]
    pub certificate_mint: Init<Writable<SeededAccount<DataAccount<'info, StuffStruct>, CeritifacteSeeds>>>,

    /// Owner of the certificates
    pub certificate_owner_authority: Signer<'info>,

    /// The source token account for the cargo certificate - owned by the `certificate_owner_authority`
    /// CHECK: Checked in token program
    #[account(mut)]
    pub certificate_token_from: UncheckedAccount<'info>,

    /// The source token account for the cargo - owned by the Starbase
    /// CHECK: Checked in cargo & token programs
    #[account(mut)]
    pub cargo_token_from: UncheckedAccount<'info>,

    /// The destination token account for the cargo - owned by the `cargo_pod`
    /// CHECK: Checked in constraints and cargo program
    #[account(
        mut,
        address = get_associated_token_address(&cargo_pod.key(), &cargo_mint.key()) @SageError::AddressMismatch
    )]
    pub cargo_token_to: UncheckedAccount<'info>,

    /// The cargo pod to send to
    /// CHECK: Checked by the cargo program
    #[account(mut)]
    pub cargo_pod: UncheckedAccount<'info>,

    /// The cargo type account
    /// CHECK: checked in cargo program
    pub cargo_type: UncheckedAccount<'info>,

    /// The cargo stats definition account
    /// CHECK: checked in cargo program
    #[account(
        address = game_accounts_and_profile.game_id.load()?.cargo.stats_definition @cargo::error::CargoError::StatsDefinitionMismatch,
    )]
    pub cargo_stats_definition: UncheckedAccount<'info>,

    /// The Cargo Program
    #[derivative(Debug = "ignore")]
    pub cargo_program: Program<'info, Cargo>,

    /// The token program
    #[derivative(Debug = "ignore")]
    pub token_program: Program<'info, Token>,
}
```