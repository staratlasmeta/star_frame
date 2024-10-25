use crate::account_set::SignedAccount;
use crate::prelude::*;
use anyhow::bail;
use bytemuck::bytes_of;
use derive_more::{Deref, DerefMut};
pub use star_frame_proc::GetSeeds;
use std::marker::PhantomData;

/// A trait for getting the seed bytes of an account.
///
/// ## Derivable
///
/// This trait can be derived for structs with named fields using the [`GetSeeds`](star_frame_proc::GetSeeds) derive macro.
///
/// ## Manually Implementing `GetSeeds`
///
/// `GetSeeds` can be manually implemented by defining a `seeds` method that returns a `Vec<&[u8]>`.
/// The `seeds` method should optionally include a constant seed at the beginning of the vector,
/// followed by calling the `seed` method on each field of the struct.
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
///       vec![b"TEST_CONST", self.key.seed(), self.number.seed()]
///     }
/// }
///
/// ```
///
pub trait GetSeeds: Debug + Clone {
    fn seeds(&self) -> Vec<&[u8]>;
}
impl<T> GetSeeds for T
where
    T: Seed + Debug + Clone,
{
    fn seeds(&self) -> Vec<&[u8]> {
        vec![self.seed()]
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
        seeds.push(bytes_of(&self.bump));
        seeds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Seeds<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct CurrentProgram;
pub trait SeedProgram {
    fn id(sys_calls: &mut impl SyscallCore) -> Result<Pubkey>;
}
impl SeedProgram for CurrentProgram {
    fn id(sys_calls: &mut impl SyscallCore) -> Result<Pubkey> {
        Ok(*sys_calls.current_program_id())
    }
}
impl<P> SeedProgram for P
where
    P: StarFrameProgram,
{
    fn id(_syscalls: &mut impl SyscallCore) -> Result<Pubkey> {
        Ok(P::PROGRAM_ID)
    }
}

#[derive(Debug, AccountSet, Deref, DerefMut)]
#[account_set(
    skip_default_idl,
    skip_default_validate,
    generics = [where T: AccountSet < 'info >]
)]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    id = "seeds",
    generics = [where T: AccountSetValidate<'info, ()> + SingleAccountSet<'info>],
    arg = Seeds<S>,
    before_validation = self.set_seeds(&arg, syscalls)
)]
#[validate(
    id = "seeds_generic",
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>],
    arg = (Seeds<S>, A),
    before_validation = self.set_seeds(&arg, syscalls)
)]
#[validate(
    id = "seeds_with_bump",
    generics = [where T: AccountSetValidate<'info, ()> + SingleAccountSet<'info>],
    arg = SeedsWithBump<S>,
    before_validation = self.set_seeds(&arg, syscalls)
)]
#[validate(
    id = "seeds_with_bump_generic",
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>],
    arg = (SeedsWithBump<S>, A),
    before_validation = self.set_seeds(&arg, syscalls)
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup <'info, A>], arg = A)]
pub struct Seeded<
    T,
    S: GetSeeds + Clone = <T as HasSeeds>::Seeds,
    P: SeedProgram = <T as HasOwnerProgram>::OwnerProgram,
> {
    #[decode(arg = arg)]
    #[validate(id = "seeds_generic", arg = arg.1)]
    #[validate(id = "seeds_with_bump_generic", arg = arg.1)]
    #[cleanup(arg = arg)]
    #[deref]
    #[deref_mut]
    #[single_account_set(
        skip_can_set_seeds,
        skip_signed_account,
        skip_has_seeds,
        skip_can_init_account
    )]
    pub(crate) account: T,
    /// Seeds of the account. Starts as `None`, and are set to `Some` after validation, during `AccountSetValidate`.
    #[account_set(skip = None)]
    pub(crate) seeds: Option<SeedsWithBump<S>>,
    phantom_p: PhantomData<P>,
}

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds + Clone, P: SeedProgram, A>
    CanSetSeeds<'info, (Seeds<S>, A)> for Seeded<T, S, P>
{
    fn set_seeds(
        &mut self,
        arg: &(Seeds<S>, A),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_and_set_seeds(arg.0 .0.clone(), syscalls)
    }
}

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds + Clone, P: SeedProgram>
    CanSetSeeds<'info, Seeds<S>> for Seeded<T, S, P>
{
    fn set_seeds(
        &mut self,
        arg: &Seeds<S>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_and_set_seeds(arg.0.clone(), syscalls)
    }
}

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds + Clone, P: SeedProgram, A>
    CanSetSeeds<'info, (SeedsWithBump<S>, A)> for Seeded<T, S, P>
{
    fn set_seeds(
        &mut self,
        arg: &(SeedsWithBump<S>, A),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_and_set_seeds_with_bump(arg.0.clone(), syscalls)
    }
}

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds + Clone, P: SeedProgram>
    CanSetSeeds<'info, SeedsWithBump<S>> for Seeded<T, S, P>
{
    fn set_seeds(
        &mut self,
        arg: &SeedsWithBump<S>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        self.validate_and_set_seeds_with_bump(arg.clone(), syscalls)
    }
}

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds, P: SeedProgram> Seeded<T, S, P> {
    fn validate_and_set_seeds(
        &mut self,
        seeds: S,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if self.seeds.is_some() {
            return Ok(());
        }
        let (address, bump) = Pubkey::find_program_address(&seeds.seeds(), &P::id(sys_calls)?);
        if self.account.account_info().key != &address {
            bail!(
                "Seeds: {:?} result in address `{}` and bump `{}`, expected `{}`",
                seeds,
                address,
                bump,
                self.account.account_info().key
            );
        }
        self.seeds = Some(SeedsWithBump { seeds, bump });
        Ok(())
    }

    fn validate_and_set_seeds_with_bump(
        &mut self,
        seeds: SeedsWithBump<S>,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if self.seeds.is_some() {
            return Ok(());
        }
        let arg_seeds = seeds.seeds_with_bump();
        let address = Pubkey::create_program_address(&arg_seeds, &P::id(sys_calls)?)?;
        if self.account.account_info().key != &address {
            bail!(
                "Seeds `{:?}` result in address `{}`, expected `{}`",
                seeds,
                address,
                self.account.account_info().key
            );
        }
        self.seeds = Some(seeds);
        Ok(())
    }
}

impl<T, S: GetSeeds, P: SeedProgram> Seeded<T, S, P> {
    pub fn access_seeds(&self) -> &SeedsWithBump<S> {
        self.seeds.as_ref().unwrap()
    }
}

impl<'info, T, S: GetSeeds, P: SeedProgram> SignedAccount<'info> for Seeded<T, S, P>
where
    T: SingleAccountSet<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        Some(self.access_seeds().seeds_with_bump())
    }
}

impl<'info, T, S: GetSeeds, P: SeedProgram> HasSeeds for Seeded<T, S, P>
where
    T: SingleAccountSet<'info>,
{
    type Seeds = S;
}

impl<'info, T, S: GetSeeds, P: SeedProgram, A> CanInitAccount<'info, A> for Seeded<T, S, P>
where
    T: SingleAccountSet<'info> + CanInitAccount<'info, A>,
{
    fn init(
        &mut self,
        arg: A,
        syscalls: &mut impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        // override seeds. Init should be called after seeds are set
        if account_seeds.is_some() {
            bail!("Conflicting account seeds during init!");
        }
        let seeds = self.seeds.as_ref().map(|s| s.seeds_with_bump());
        self.account.init(arg, syscalls, seeds)
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A, S, P: SeedProgram> AccountSetToIdl<'info, A> for Seeded<T, S, P>
    where
        T: AccountSetToIdl<'info, A> + SingleAccountSet<'info>,
        S: GetSeeds,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            // TODO: Include program
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::SeededAccount)
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

    #[test]
    fn test_unit_struct() {
        #[derive(Debug, GetSeeds, Clone)]
        pub struct TestAccount {}

        let account = TestAccount {};
        let seeds = account.seeds();
        assert_eq!(seeds.len(), 0);
    }

    #[test]
    fn test_single_key() {
        #[derive(Debug, GetSeeds, Clone)]
        pub struct TestAccount {
            key: Pubkey,
        }

        let account = TestAccount {
            key: Pubkey::new_unique(),
        };
        let intended_seeds = vec![account.key.seed()];
        let seeds = account.seeds();
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 1);
    }

    #[test]
    fn test_two_keys() {
        #[derive(Debug, GetSeeds, Clone)]
        pub struct TestAccount {
            key1: Pubkey,
            key2: Pubkey,
        }

        let account = TestAccount {
            key1: Pubkey::new_unique(),
            key2: Pubkey::new_unique(),
        };
        let intended_seeds = vec![account.key1.seed(), account.key2.seed()];
        let seeds = account.seeds();
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 2);
    }

    #[test]
    fn test_key_and_number() {
        #[derive(Debug, GetSeeds, Clone)]
        pub struct TestAccount {
            key: Pubkey,
            number: u64,
        }

        let account = TestAccount {
            key: Pubkey::new_unique(),
            number: 42,
        };
        let intended_seeds = vec![account.key.seed(), account.number.seed()];
        let seeds = account.seeds();
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 2);
    }

    #[test]
    fn test_unit_with_const_seed() {
        #[derive(Debug, GetSeeds, Clone)]
        #[seed_const(b"TEST_CONST")]
        pub struct TestAccount {}

        let account = TestAccount {};
        let seeds = account.seeds();
        let intended_seeds = vec![b"TEST_CONST".as_ref()];
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 1);
    }

    #[test]
    fn test_one_key_with_const_seed() {
        #[derive(Debug, GetSeeds, Clone)]
        #[seed_const(b"TEST_CONST")]
        pub struct TestAccount {
            key: Pubkey,
        }

        let account = TestAccount {
            key: Pubkey::new_unique(),
        };
        let intended_seeds = vec![b"TEST_CONST".as_ref(), account.key.seed()];
        let seeds = account.seeds();
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 2);
    }

    #[test]
    fn test_path_seed() {
        pub struct Cool {}

        impl Cool {
            const DISC: &'static [u8] = b"TEST_CONST";
        }

        #[derive(Debug, GetSeeds, Clone)]
        #[seed_const(Cool::DISC)]
        pub struct TestAccount {}

        let account = TestAccount {};
        let seeds = account.seeds();
        let intended_seeds = vec![b"TEST_CONST".as_ref()];
        assert_eq!(seeds, intended_seeds);
        assert_eq!(seeds.len(), 1);
    }
}
