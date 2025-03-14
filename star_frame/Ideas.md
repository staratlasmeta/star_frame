program trait 


Account set

```json
{
  "instruction": {
    "accounts": [
      {
        "ty": 
      }
    ]
  }
}

```


Program<FactionEnlistment>;

```rust
trait ToSeedBytes {
    fn to_seed_bytes(&self) -> Vec<u8>;
}
```

Types of seeds we want to handle
1. Constants
2. Account keys from the instruction
3. Instruction arguments
4. Fields from account data in the instruction




```rust
fn cook() {
    let x = [
        Seed::Constant(CERTIFICATE_MINT),
        Seed::AccountSetPath(["starbase_and_starbase_player", "starbase", "key"]),
        Seed::AccountSetPath(["cargo_mint"]),
        Seed::Argument("seq_id", )
        Seed::AccountSetPath(["starbase_and_starbase_player", "starbase", "game_id"])
    ];
}
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

```c
template <typename T, typename T1> 
auto compose(T a, T1 b) -> decltype(a + b) {
   return a+b;
}
```

```rust
#[derive(Seeds)]
#[end_constant(END)]
struct CeritifacteSeeds{
    #[constant(CERTIFICATE_MINT)]
    #[constant(ANOTHER_CONSTANT)]
    starbase: Pubkey,
    cargo_mint: Pubkey,
    seq_id: u8,
    #[constant(GAME_FOLLOWS)]
    game_id: Pubkey,
}
// Generated
impl Seeds for CeritifacteSeeds {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![
            path::to::Seed::seed(&CERTIFICATE_MINT),
            self.starbase.seed(),
            self.cargo_mint.seed(),
            self.seq_id.seed(),
            path::to::Seed::seed(&GAME_FOLLOWS),
            self.game_id.seed(),
            path::to::Seed::seed(&END),
        ]
    }
}
impl<T> Seeds for T where T: Seed {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seed()]
    }
}

pub trait Seed {
    fn seed(&self) -> &[u8];
}
impl<T> Seed for T where T: Pod {
    fn seed(&self) -> &[u8] {
        to_bytes(self)
    }
}
impl<'a> Seed for &'a [u8] {
    fn seed(&self) -> &[u8] {
        self
    }
}

struct SeedsWithBump<T: Seeds>{
    seeds: T,
    bump: u8,
}

pub trait SeededAccountData: AccountData {
    type Seeds: Seeds;
}

pub struct SeededAccount<T, S> {
    account: T,
    seeds: Option<SeedsWithBump<S>>,
}

pub struct SeededDataAccount<'info, T>(SeededAccount<DataAccount<'info, T>, T::Seeds>) where T: SeededAccountData;
impl<T, S> AccountSetValidate<S> for SeededAccount<T, S> where S: Seeds {}
impl<T, S> AccountSetValidate<SeedsWithBump<S>> for SeededAccount<T, S> where S: Seeds {}

struct SubInitArgs {
    seeds: Option<Vec<Vec<u8>>>,
    size: usize,
}
trait InitAccountSet<A>: AccountSetValidate<A> + SingleAccountSet {
    fn init_account(&mut self, args: &mut SubInitArgs, validate_arg: &A) -> Result<()>;
    fn post_init(&mut self) -> Result<()>;
}

pub struct Init<T>{t: T}
pub struct InitArgs<'a, 'info> {
    pub system_program: &'a Program<'info, SystemProgram>,
}

impl<'a, 'info, T, A> AccountSetValidate<'info, (InitArgs<'a, 'info>, A)> for Init<T> where T: InitAccountSet<A> {
    fn account_set_validate(&mut self, arg: (InitArgs<'a, 'info>, A)) -> Result<()> {
        let mut args: SubInitArgs = Default::default();
        self.t.init_account(&mut args, &arg.1)?;
        self.init_with_args(arg.0, args)?;
        T::account_set_validate(arg.1);
    }
}

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
    #[validate(
        arg = (InitArgs { system_program: &self.system_program }, CeritifacteSeeds {
            starbase: starbase_and_starbase_player.starbase.as_ref().key,
            cargo_mint: cargo_mint.key(),
            seq_id: starbase_and_starbase_player.starbase.load()?.seq_id,
            game_id: starbase_and_starbase_player.starbase.load()?.game_id,
        },)
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


```rs

#[extra = extra_a]
struct A {
 a_inner: AccountInfo,
}

#[extra = do_extra()]
struct Outer {
    a: A,
    b: Seeds<Init<Data>,
    c: C,
}
```

Seeds<Init>
before_seeds -> set seeds if possible
 before_init
 extra_init -> try to CPI w/ seeds, error if seeds couldnt be set
extra_seeds -> set seeds if not yet set

Seeds<Data>
before_seeds
 before_data
 extra_data
extra_seeds



A -> AccountInfo, extra_a,
B -> before_seeds
     Seeds ->
        before_init
        Init -> 
            before_data
            Data -> 
                data_account_validations,
        init_extra,
    seeds_extra
C -> extra_c,
extra_outer

Seeds<Data>
   


```