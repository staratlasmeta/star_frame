use crate::account_set::{SignedAccount, WritableAccount};
use crate::prelude::*;
use anyhow::bail;
use bytemuck::bytes_of;
use std::ops::{Deref, DerefMut};

pub trait GetSeeds {
    fn seeds(&self) -> Vec<&[u8]>;
}
impl<T> GetSeeds for T
where
    T: Seed,
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

#[derive(Debug)]
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

// Structs
#[derive(Debug, AccountSet)]
#[account_set(
    skip_default_idl,
    generics = [where T: AccountSet < 'info >]
)]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>],
    arg = (S, A),
    before_validation = Self::validate_seeds(self, arg.0)
)]
#[validate(
    id = "seeds",
    generics = [where T: AccountSetValidate<'info, ()> + SingleAccountSet<'info>],
    arg = Seeds<S>,
    before_validation = Self::validate_seeds(self, arg.0)
)]
#[validate(
    id = "seeds_generic",
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>],
    arg = (Seeds<S>, A),
    before_validation = Self::validate_seeds(self, arg.0.0)
)]
#[validate(
    id = "seeds_skip",
    generics = [<> where T: SingleAccountSet<'info>],
    arg = (Skip, Seeds<S>),
    before_validation = Self::validate_seeds(self, arg.1.0)
)]
#[validate(
    id = "seeds_with_bump",
    generics = [where T: AccountSetValidate<'info, ()> + SingleAccountSet<'info>],
    arg = SeedsWithBump<S>,
    before_validation = Self::validate_seeds_with_bump(self, arg)
)]
#[validate(
    id = "seeds_with_bump_generic",
    generics = [<A> where T: AccountSetValidate<'info, A> + SingleAccountSet<'info>],
    arg = (SeedsWithBump<S>, A),
    before_validation = Self::validate_seeds_with_bump(self, arg.0)
)]
#[validate(
    id = "seeds_with_bump_skip",
    generics = [<> where T: SingleAccountSet<'info>],
    arg = (Skip, SeedsWithBump<S>),
    before_validation = Self::validate_seeds_with_bump(self, arg.1)
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup <'info, A>], arg = A)]
pub struct SeededAccount<T, S: GetSeeds> {
    #[cleanup(arg = arg)]
    #[validate(arg = arg.1)]
    #[validate(id = "seeds", arg = ())]
    #[validate(id = "seeds_generic", arg = arg.1)]
    #[validate(id = "seeds_skip", skip)]
    #[validate(id = "seeds_with_bump", arg = ())]
    #[validate(id = "seeds_with_bump_generic", arg = arg.1)]
    #[validate(id = "seeds_with_bump_skip", skip)]
    #[decode(arg = arg)]
    pub(crate) account: T,
    #[account_set(skip = None)]
    pub(crate) seeds: Option<SeedsWithBump<S>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Skip;

impl<'info, T: SingleAccountSet<'info>, S: GetSeeds> SeededAccount<T, S> {
    fn validate_seeds(&mut self, seeds: S) -> Result<()> {
        let (address, bump) =
            Pubkey::find_program_address(&seeds.seeds(), self.account_info().owner);
        if self.account.account_info().key != &address {
            bail!(ProgramError::Custom(20));
        }
        self.seeds = Some(SeedsWithBump { seeds, bump });
        Ok(())
    }

    fn validate_seeds_with_bump(&mut self, seeds: SeedsWithBump<S>) -> Result<()> {
        let arg_seeds = seeds.seeds_with_bump();
        let address = Pubkey::create_program_address(&arg_seeds, self.account_info().owner)?;
        if self.account.account_info().key != &address {
            bail!(ProgramError::Custom(20));
        }
        self.seeds = Some(seeds);
        Ok(())
    }
}

impl<T, S: GetSeeds> SeededAccount<T, S> {
    pub fn access_seeds(&self) -> &SeedsWithBump<S> {
        self.seeds.as_ref().unwrap()
    }

    pub fn seeds_with_bump(&self) -> Vec<&[u8]> {
        self.seeds.as_ref().unwrap().seeds_with_bump()
    }
}

impl<'info, T, S> SingleAccountSet<'info> for SeededAccount<T, S>
where
    T: SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.account.account_info()
    }
}
impl<'info, T, S> SignedAccount<'info> for SeededAccount<T, S>
where
    T: SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        Some(self.seeds_with_bump())
    }
}
impl<'info, T, S> WritableAccount<'info> for SeededAccount<T, S>
where
    T: WritableAccount<'info>,
    S: GetSeeds,
{
}

impl<T, S: GetSeeds> Deref for SeededAccount<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<T, S: GetSeeds> DerefMut for SeededAccount<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A, S> AccountSetToIdl<'info, A> for SeededAccount<T, S>
    where
        T: AccountSetToIdl<'info, A> + SingleAccountSet<'info>,
        S: GetSeeds,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::SeededAccount)
        }
    }
}
