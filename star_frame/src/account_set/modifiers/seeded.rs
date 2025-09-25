//! Account modifier for Program Derived Address (PDA) validation and management.
//!
//! The `Seeded<T>` modifier wraps accounts that are derived from seeds using Solana's PDA system.
//! It automatically validates that the provided account matches the expected PDA derived from the
//! given seeds and program, and can generate the correct PDA addresses for account creation.

use crate::{
    account_set::{
        modifiers::{CanInitAccount, CanInitSeeds, HasSeeds, SignedAccount},
        AccountSetValidate,
    },
    prelude::*,
    ErrorCode,
};
use bytemuck::bytes_of;
use derive_more::{Deref, DerefMut};
use std::marker::PhantomData;

pub use star_frame_proc::GetSeeds;
/// A trait for getting the seed bytes of an account. The last element of the returned vector should be an empty slice, in order to replace it with a bump later on without
/// having to push to the vector.
///
/// ## Derivable
///
/// This trait can be derived for structs with named fields using the [`GetSeeds`](star_frame_proc::GetSeeds) derive macro.
///
/// ## Manually Implementing `GetSeeds`
///
/// `GetSeeds` can be manually implemented by defining a `seeds` method that returns a `Vec<&[u8]>`.
/// The `seeds` method should optionally include a constant seed at the beginning of the vector,
/// followed by calling the `seed` method on each field of the struct, with an empty slice at the end.
///
/// ```
/// # use star_frame::prelude::*;
/// #[derive(Debug, Clone)]
/// pub struct Cool {
///     key: Pubkey,
///     number: u64,
/// }
///
/// impl GetSeeds for Cool {
///    fn seeds(&self) -> Vec<&[u8]> {
///       vec![b"TEST_CONST", self.key.seed(), self.number.seed(), &[]]
///     }
/// }
///
/// ```
pub trait GetSeeds: Debug {
    fn seeds(&self) -> Vec<&[u8]>;
}
impl<T> GetSeeds for T
where
    T: Seed + Debug,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seed(), &[]]
    }
}

pub trait Seed {
    fn seed(&self) -> &[u8];
}
impl<T> Seed for T
where
    T: NoUninit,
{
    fn seed(&self) -> &[u8] {
        bytes_of(self)
    }
}

/// A combination of seeds and bump value for deterministic PDA generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct SeedsWithBump<T: GetSeeds> {
    pub seeds: T,
    pub bump: u8,
}

impl<T> SeedsWithBump<T>
where
    T: GetSeeds,
{
    pub fn seeds_with_bump(&self) -> Vec<&[u8]> {
        let mut seeds = self.seeds.seeds();
        // TODO: Replace with let chains once stable
        if let Some(last) = seeds.last_mut() {
            if last.is_empty() {
                *last = bytes_of(&self.bump);
                return seeds;
            }
        }
        seeds.push(bytes_of(&self.bump));
        seeds
    }
}

/// Wrapper type for seed validation arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Seeds<T>(pub T);

/// Allows generic [`crate::account_set`]s to be used in multiple programs by defaulting the [`SeedProgram`] to the current
/// executing program. This is the default [`SeedProgram`] for [`Seeded`], and the only [`SeedProgram`] that can be used with
/// the [`Init`] account set.
#[derive(Debug, Clone, Copy)]
pub struct CurrentProgram;
/// Trait for types that can provide a program ID for PDA derivation.
pub trait SeedProgram {
    fn id(ctx: &Context) -> Result<Pubkey>;
    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    fn idl_program() -> Option<Pubkey>;
}

impl SeedProgram for CurrentProgram {
    fn id(ctx: &Context) -> Result<Pubkey> {
        Ok(*ctx.current_program_id())
    }
    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    fn idl_program() -> Option<Pubkey> {
        None
    }
}

impl<P> SeedProgram for P
where
    P: StarFrameProgram,
{
    fn id(_ctx: &Context) -> Result<Pubkey> {
        Ok(P::ID)
    }

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    fn idl_program() -> Option<Pubkey> {
        Some(P::ID)
    }
}

/// A modifier that validates accounts are derived from the expected seeds using Solana's PDA system.
///
/// This wrapper ensures that the provided account matches the Program Derived Address (PDA) that
/// would be generated from the given seeds and program. It supports validation with both automatic
/// bump discovery and explicit bump values, making it suitable for both account lookup and creation scenarios.
#[derive(AccountSet, Deref, DerefMut, derive_where::DeriveWhere)]
#[derive_where(Debug, Clone; T, SeedsWithBump<S>)]
#[account_set(skip_default_idl, skip_default_validate)]
#[validate(
    id = "seeds",
    generics = [where T: AccountSetValidate<()> + SingleAccountSet],
    arg = Seeds<S>,
    before_validation = self.validate_and_set_seeds(&arg, ctx)
)]
#[validate(
    id = "seeds_generic",
    arg = (Seeds<S>, A),
    before_validation = self.validate_and_set_seeds(&arg.0, ctx)
)]
#[validate(
    id = "seeds_with_bump",
    generics = [where T: AccountSetValidate<()> + SingleAccountSet],
    arg = SeedsWithBump<S>,
    before_validation = self.validate_and_set_seeds_with_bump(&arg, ctx)
)]
#[validate(
    id = "seeds_with_bump_generic",
    arg = (SeedsWithBump<S>, A),
    before_validation = self.validate_and_set_seeds_with_bump(&arg.0, ctx)
)]
pub struct Seeded<T, S = <T as HasSeeds>::Seeds, P = CurrentProgram>
where
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    #[single_account_set(
        skip_signed_account,
        skip_has_seeds,
        skip_can_init_seeds,
        skip_can_init_account
    )]
    #[validate(id = "seeds_generic", arg = arg.1)]
    #[validate(id = "seeds_with_bump_generic", arg = arg.1)]
    #[deref]
    #[deref_mut]
    pub(crate) account: T,
    /// Seeds of the account. Starts as `None`, and are set to `Some` after validation, during `AccountSetValidate`.
    #[account_set(skip = None)]
    pub(crate) seeds: Option<SeedsWithBump<S>>,
    #[account_set(skip = PhantomData)]
    phantom_p: PhantomData<P>,
}

impl<T, S, P, A> CanInitSeeds<(Seeds<S>, A)> for Seeded<T, S, P>
where
    T: SingleAccountSet + AccountSetValidate<A>,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    fn init_seeds(&mut self, arg: &(Seeds<S>, A), ctx: &Context) -> Result<()> {
        self.validate_and_set_seeds(&arg.0, ctx)
    }
}

impl<T, S, P> CanInitSeeds<Seeds<S>> for Seeded<T, S, P>
where
    T: SingleAccountSet + AccountSetValidate<()>,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    fn init_seeds(&mut self, arg: &Seeds<S>, ctx: &Context) -> Result<()> {
        self.validate_and_set_seeds(arg, ctx)
    }
}

impl<T, S, P, A> CanInitSeeds<(SeedsWithBump<S>, A)> for Seeded<T, S, P>
where
    T: SingleAccountSet + AccountSetValidate<A>,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    fn init_seeds(&mut self, arg: &(SeedsWithBump<S>, A), ctx: &Context) -> Result<()> {
        self.validate_and_set_seeds_with_bump(&arg.0, ctx)
    }
}

impl<T, S, P> CanInitSeeds<SeedsWithBump<S>> for Seeded<T, S, P>
where
    T: SingleAccountSet + AccountSetValidate<()>,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    fn init_seeds(&mut self, arg: &SeedsWithBump<S>, ctx: &Context) -> Result<()> {
        self.validate_and_set_seeds_with_bump(arg, ctx)
    }
}

impl<T, S, P> Seeded<T, S, P>
where
    T: SingleAccountSet,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    fn validate_and_set_seeds(&mut self, seeds: &Seeds<S>, ctx: &Context) -> Result<()> {
        if self.seeds.is_some() {
            return Ok(());
        }
        let seeds = seeds.clone().0;
        let (address, bump) = Pubkey::find_program_address(&seeds.seeds(), &P::id(ctx)?);
        let expected = self.account.account_info().pubkey();
        ensure!(
            address.fast_eq(expected),
            ProgramError::InvalidSeeds,
            "Seeds: {seeds:?} result in address `{address}` and bump `{bump}`, expected `{expected}`"
        );
        self.seeds = Some(SeedsWithBump { seeds, bump });
        Ok(())
    }

    fn validate_and_set_seeds_with_bump(
        &mut self,
        seeds: &SeedsWithBump<S>,
        ctx: &Context,
    ) -> Result<()> {
        if self.seeds.is_some() {
            return Ok(());
        }
        let arg_seeds = seeds.seeds_with_bump();
        let address = Pubkey::create_program_address(&arg_seeds, &P::id(ctx)?)?;
        let expected = self.account.account_info().pubkey();
        ensure!(
            address.fast_eq(expected),
            ErrorCode::AddressMismatch,
            "Seeds `{seeds:?}` result in address `{address}`, expected `{expected}`"
        );
        self.seeds = Some(seeds.clone());
        Ok(())
    }
}

impl<T, S, P> Seeded<T, S, P>
where
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    pub fn access_seeds(&self) -> &SeedsWithBump<S> {
        self.seeds.as_ref().expect("Seeds not set!")
    }
}

/// [`Seeded`] can only sign when the seed program is [`CurrentProgram`].
impl<T, S> SignedAccount for Seeded<T, S, CurrentProgram>
where
    T: SingleAccountSet,
    S: GetSeeds + Clone,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        Some(self.access_seeds().seeds_with_bump())
    }
}

impl<T, S, P> HasSeeds for Seeded<T, S, P>
where
    T: SingleAccountSet,
    S: GetSeeds + Clone,
    P: SeedProgram,
{
    type Seeds = S;
}

/// [`Seeded`] can only be initialized with [`CurrentProgram`] as the seed program.
impl<T, S, A> CanInitAccount<A> for Seeded<T, S, CurrentProgram>
where
    T: CanInitAccount<A>,
    S: GetSeeds + Clone,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: A,
        account_seeds: Option<Vec<&[u8]>>,
        ctx: &Context,
    ) -> Result<()> {
        // override seeds. Init should be called after seeds are set
        if account_seeds.is_some() {
            bail!(
                crate::ErrorCode::ConflictingAccountSeeds,
                "Conflicting account seeds during init."
            );
        }
        let seeds = self
            .seeds
            .as_ref()
            .map(|s| s.seeds_with_bump())
            .ok_or_else(|| {
                error!(
                    crate::ErrorCode::SeedsNotSet,
                    "Seeds not set for `Seeded` during init."
                )
            })?;
        self.account
            .init_account::<IF_NEEDED>(arg, Some(seeds), ctx)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::FindIdlSeeds;

    use super::*;
    use star_frame_idl::{account_set::IdlAccountSetDef, seeds::IdlFindSeeds, IdlDefinition};

    impl<T, A, S, P, F> AccountSetToIdl<(Seeds<F>, A)> for Seeded<T, S, P>
    where
        T: AccountSetToIdl<A> + SingleAccountSet,
        S: GetSeeds + Clone,
        P: SeedProgram,
        F: FindIdlSeeds,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (Seeds<F>, A),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let mut set = T::account_set_to_idl(idl_definition, arg.1)?;
            let single = set.single()?;
            if single.seeds.is_some() {
                return Err(star_frame_idl::Error::Custom(format!(
                    "Seeds already set for `Seeded`. Got: {single:?}"
                )));
            }
            if single.is_init {
                return Err(star_frame_idl::Error::Custom(format!(
                    "`Seeded` should not wrap an init account. Wrap `Seeded` with `Init` instead. Got: {single:?}"
                )));
            }
            let seeds = IdlFindSeeds {
                seeds: F::find_seeds(&arg.0 .0)?,
                program: P::idl_program(),
            };
            single.seeds = Some(seeds);

            Ok(set)
        }
    }

    impl<T, S, P, F> AccountSetToIdl<Seeds<F>> for Seeded<T, S, P>
    where
        T: AccountSetToIdl<()> + SingleAccountSet,
        S: GetSeeds + Clone,
        P: SeedProgram,
        F: FindIdlSeeds,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Seeds<F>,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            Self::account_set_to_idl(idl_definition, (arg, ()))
        }
    }

    impl<T, S, P> AccountSetToIdl<()> for Seeded<T, S, P>
    where
        T: AccountSetToIdl<()> + SingleAccountSet,
        S: GetSeeds + Clone,
        P: SeedProgram,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)?.assert_single()
        }
    }
}

///
///```compile_fail
/// use star_frame_proc::GetSeeds;
/// #[derive(GetSeeds)]
/// struct Banana(i32, i32);
/// ```
fn _unnamed_seed_structs_fail() {}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use solana_pubkey::Pubkey;

    #[derive(Debug, GetSeeds, Clone)]
    pub struct UnitSeeds {}
    #[test]
    fn test_unit_struct() {
        let unit_seeds = UnitSeeds {};
        let seeds = <UnitSeeds as crate::prelude::GetSeeds>::seeds(&unit_seeds);
        assert_eq!(seeds, &[&[] as &[u8]]);
    }

    #[derive(Debug, GetSeeds, Clone)]
    pub struct SingleKey {
        key: Pubkey,
    }
    #[test]
    fn test_single_key() {
        let single_key = SingleKey {
            key: Pubkey::new_unique(),
        };
        let intended_seeds = vec![single_key.key.seed(), &[]];
        let seeds = single_key.seeds();
        assert_eq!(seeds, intended_seeds);
    }

    #[derive(Debug, GetSeeds, Clone)]
    pub struct TwoKeys {
        key1: Pubkey,
        key2: Pubkey,
    }
    #[test]
    fn test_two_keys() {
        let two_keys = TwoKeys {
            key1: Pubkey::new_unique(),
            key2: Pubkey::new_unique(),
        };
        let intended_seeds = vec![two_keys.key1.seed(), two_keys.key2.seed(), &[]];
        let seeds = two_keys.seeds();
        assert_eq!(seeds, intended_seeds);
    }

    #[derive(Debug, GetSeeds, Clone)]
    pub struct KeyAndNumber {
        key: Pubkey,
        number: u64,
    }
    #[test]
    fn test_key_and_number() {
        let key_and_number = KeyAndNumber {
            key: Pubkey::new_unique(),
            number: 42,
        };
        let intended_seeds = vec![key_and_number.key.seed(), key_and_number.number.seed(), &[]];
        let seeds = key_and_number.seeds();
        assert_eq!(seeds, intended_seeds);
    }

    #[derive(Debug, GetSeeds, Clone)]
    #[get_seeds(seed_const = b"TEST_CONST")]
    pub struct OnlyConstSeed {}
    #[test]
    fn test_unit_with_const_seed() {
        let only_const_seed = OnlyConstSeed {};
        let seeds = only_const_seed.seeds();
        let intended_seeds = vec![b"TEST_CONST".as_ref(), &[]];
        assert_eq!(seeds, intended_seeds);
    }

    #[derive(Debug, GetSeeds, Clone)]
    #[get_seeds(seed_const = b"TEST_CONST")]
    pub struct OneKeyConstSeed {
        key: Pubkey,
    }
    #[test]
    fn test_one_key_with_const_seed() {
        let account = OneKeyConstSeed {
            key: Pubkey::new_unique(),
        };
        let intended_seeds = vec![b"TEST_CONST".as_ref(), account.key.seed(), &[]];
        let seeds = account.seeds();
        assert_eq!(seeds, intended_seeds);
    }

    pub struct Cool {}
    impl Cool {
        const DISC: &'static [u8] = b"TEST_CONST";
    }
    #[derive(Debug, GetSeeds, Clone)]
    #[get_seeds(seed_const = Cool::DISC)]
    pub struct SeedPath {}

    #[test]
    fn test_path_seed() {
        let account = SeedPath {};
        let seeds = account.seeds();
        let intended_seeds = vec![b"TEST_CONST".as_ref(), &[]];
        assert_eq!(seeds, intended_seeds);
    }
}
