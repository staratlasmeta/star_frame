//! Strongly typed and statically verified instruction accounts.
pub mod account;
pub mod borsh_account;
mod impls; // Just impls, no need to re-export
pub mod modifiers;
pub mod program;
pub mod rest;
pub mod single_set;
pub mod system_account;
pub mod sysvar;
pub mod validated_account;

pub use star_frame_proc::{AccountSet, ProgramAccount};

use crate::{prelude::*, ErrorCode};
use bytemuck::bytes_of;
use modifiers::{HasOwnerProgram, OwnerProgramDiscriminant};
use std::{mem::MaybeUninit, slice};

/// An account that has a discriminant and is owned by a [`StarFrameProgram`].
///
/// Derivable via [`derive@ProgramAccount`].
pub trait ProgramAccount: HasOwnerProgram {
    /// The discriminant of the account. This should be unique for each account type in a program.
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;
    /// The discriminant of the account as bytes.
    #[must_use]
    #[inline]
    fn discriminant_bytes() -> Vec<u8> {
        bytes_of(&Self::DISCRIMINANT).into()
    }

    /// Validates the owner matches [`Self::OwnerProgram::ID`](`crate::program::StarFrameProgram::ID`) and the discriminant matches [`Self::DISCRIMINANT`].
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn validate_account_info(info: AccountInfo) -> Result<()> {
        validate_discriminant::<Self>(info)?;

        if !info.owner().fast_eq(&Self::OwnerProgram::ID) {
            bail!(
                ProgramError::InvalidAccountOwner,
                "Account {} owner {} does not match expected program ID {}",
                info.pubkey(),
                info.owner_pubkey(),
                Self::OwnerProgram::ID
            );
        }

        Ok(())
    }
}

/// Fast discriminant comparison, with fast path unaligned reads for small discriminants.
///
/// Adapted from [Typhoon](https://github.com/exotic-markets-labs/typhoon/blob/60c5197cc632f1bce07ba27876669e4ca8580421/crates/accounts/src/discriminator.rs#L8)
#[allow(clippy::inline_always)]
#[inline(always)]
fn validate_discriminant<T: ProgramAccount + ?Sized>(info: AccountInfo) -> Result<()> {
    // This check should be optimized out
    if size_of::<OwnerProgramDiscriminant<T>>() == 0 {
        return Ok(());
    }

    // Ensure account data is at least the size of the discriminant
    if info.data_len() < size_of::<OwnerProgramDiscriminant<T>>() {
        bail!(
            ProgramError::AccountDataTooSmall,
            "Account {} data length {} is less than expected discriminant size {}",
            info.pubkey(),
            info.data_len(),
            size_of::<OwnerProgramDiscriminant<T>>()
        );
    }

    info.can_borrow_data()?;
    let data_ptr = info.data_ptr();

    // SAFETY:
    // We've already verified that data.len() >= discriminant.len(),
    // so we know we have at least `len` bytes available for reading (so can cast to slice).
    // Unaligned reads are safe for primitive types on all supported architectures.
    // The pointer casts to smaller integer types (u16, u32, u64) are valid because we're
    // only reading the exact number of bytes specified by `len`.
    let matches = unsafe {
        // We are reading unaligned, so the casts are fine
        // Choose optimal comparison strategy based on discriminant length
        #[allow(clippy::cast_ptr_alignment)]
        #[allow(clippy::cast_ptr_alignment)]
        match size_of::<OwnerProgramDiscriminant<T>>() {
            1 => *data_ptr == bytemuck::cast::<_, u8>(T::DISCRIMINANT),
            2 => {
                let data_val = data_ptr.cast::<u16>().read_unaligned();
                let disc_val = bytemuck::cast::<_, u16>(T::DISCRIMINANT);
                data_val == disc_val
            }
            4 => {
                let data_val = data_ptr.cast::<u32>().read_unaligned();
                let disc_val = bytemuck::cast::<_, u32>(T::DISCRIMINANT);
                data_val == disc_val
            }
            8 => {
                let data_val = data_ptr.cast::<u64>().read_unaligned();
                let disc_val = bytemuck::cast::<_, u64>(T::DISCRIMINANT);
                data_val == disc_val
            }
            _ => {
                let data =
                    slice::from_raw_parts(data_ptr, size_of::<OwnerProgramDiscriminant<T>>());
                data == bytemuck::bytes_of(&T::DISCRIMINANT)
            }
        }
    };
    if !matches {
        bail!(
            ErrorCode::DiscriminantMismatch,
            "Account {} data does not match expected discriminant for program {}",
            info.pubkey(),
            T::OwnerProgram::ID
        );
    }

    Ok(())
}

/// Convenience methods for decoding and validating a list of [`AccountInfo`]s to an [`AccountSet`].
///
/// Performs [`AccountSetDecode::decode_accounts`] and [`AccountSetValidate::validate_accounts`] on the accounts.
///
/// See [`TryFromAccounts`] for a version of this trait that uses `()` for the decode and validate args.
pub trait TryFromAccountsWithArgs<'a, D, V>:
    AccountSetDecode<'a, D> + AccountSetValidate<V>
{
    fn try_from_accounts_with_args(
        accounts: &mut &'a [AccountInfo],
        decode: D,
        validate: V,
        ctx: &mut Context,
    ) -> Result<Self> {
        let mut set = Self::decode_accounts(accounts, decode, ctx)?;
        set.validate_accounts(validate, ctx)?;
        Ok(set)
    }

    fn try_from_account_with_args(
        account: &'a AccountInfo,
        decode: D,
        validate: V,
        ctx: &mut Context,
    ) -> Result<Self>
    where
        Self: SingleAccountSet,
    {
        let accounts = &mut slice::from_ref(account);
        Self::try_from_accounts_with_args(accounts, decode, validate, ctx)
    }
}

/// Additional convenience methods around [`TryFromAccountsWithArgs`] for when the [`AccountSetDecode`] and [`AccountSetValidate`] args are `()`.
pub trait TryFromAccounts<'a>: TryFromAccountsWithArgs<'a, (), ()> {
    fn try_from_accounts(accounts: &mut &'a [AccountInfo], ctx: &mut Context) -> Result<Self> {
        Self::try_from_accounts_with_args(accounts, (), (), ctx)
    }

    fn try_from_account(account: &'a AccountInfo, ctx: &mut Context) -> Result<Self>
    where
        Self: SingleAccountSet,
    {
        Self::try_from_account_with_args(account, (), (), ctx)
    }
}

impl<'a, T, D, V> TryFromAccountsWithArgs<'a, D, V> for T where
    T: AccountSetDecode<'a, D> + AccountSetValidate<V>
{
}

impl<'a, T> TryFromAccounts<'a> for T where T: TryFromAccountsWithArgs<'a, (), ()> {}

/// An [`AccountSet`] that can be decoded from a list of [`AccountInfo`]s using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetDecode<'a, A>: Sized {
    /// Decode the accounts from `accounts` using `decode_input`.
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: A,
        ctx: &mut Context,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
///
/// Evaluate wrapping as inner before outer.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetValidate<A> {
    /// Validate the accounts using `validate_input`.
    #[rust_analyzer::completions(ignore_flyimport)]
    fn validate_accounts(&mut self, validate_input: A, ctx: &mut Context) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetCleanup<A> {
    /// Clean up the accounts using `cleanup_input`.
    #[rust_analyzer::completions(ignore_flyimport)]
    fn cleanup_accounts(&mut self, cleanup_input: A, ctx: &mut Context) -> Result<()>;
}

/// Sentinel value for [`CpiAccountSet::AccountLen`] for a dynamic CPI account set.
pub type DynamicCpiAccountSetLen = typenum::U100;

/// An [`AccountSet`] that can be converted into a list of [`AccountInfo`]s and [`AccountMeta`]s for a CPI.
///
/// # Safety
/// With N >= 0, [`Self::write_account_infos`] and [`Self::write_account_metas`] must write to N elements of the array and increment the index by N.
/// Failure to do so will result in undefined behavior.
pub unsafe trait CpiAccountSet {
    /// Whether or not the CPI accounts contains an option (which requires passing in the program info)
    type ContainsOption: typenum::Bit;
    /// The minimum information needed to create a list of account infos and metas for a CPI for Self.
    type CpiAccounts: Debug;
    /// The number of accounts this CPI might use. Set to [`DynamicCpiAccountSetLen`] for dynamic
    type AccountLen: typenum::Unsigned;

    #[rust_analyzer::completions(ignore_flyimport)]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts;
    fn write_account_infos<'a>(
        program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()>;
    fn write_account_metas<'a>(
        program_id: &'a Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<pinocchio::instruction::AccountMeta<'a>>],
    );
}

/// A helper struct to create distict types to bind CpiAccountSet's associated types to when
/// a client struct has multiple identical fields
#[doc(hidden)]
#[derive(Debug)]
pub struct CpiConstWrapper<T, const N: usize>(T);
unsafe impl<T, const N: usize> CpiAccountSet for CpiConstWrapper<T, N>
where
    T: CpiAccountSet,
{
    type CpiAccounts = T::CpiAccounts;
    type ContainsOption = T::ContainsOption;
    type AccountLen = T::AccountLen;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        T::to_cpi_accounts(&self.0)
    }

    #[inline]
    fn write_account_infos<'a>(
        program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        T::write_account_infos(program, accounts, index, infos)
    }

    #[inline]
    fn write_account_metas<'a>(
        program_id: &'a Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
        T::write_account_metas(program_id, accounts, index, metas);
    }
}

/// Used to convert an `AccountSet`s [`Self::ClientAccounts`] into a list of [`AccountMeta`]s for an instruction.
#[rust_analyzer::completions(ignore_methods)]
pub trait ClientAccountSet {
    /// The minimum information needed to create a list of account metas for Self.
    type ClientAccounts: Clone + Debug;
    /// The minimum number of accounts the instructionmight use
    const MIN_LEN: usize;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    );
}

/// Used to check if the key matches the expected key.
pub trait CheckKey {
    /// Checks if the key matches the expected key.
    fn check_key(&self, key: &Pubkey) -> Result<()>;
}

static_assertions::assert_obj_safe!(CanAddLamports, CanFundRent);

/// Indicates that this can add lamports to another account.
#[rust_analyzer::completions(ignore_methods)]
pub trait CanAddLamports: Debug {
    #[rust_analyzer::completions(ignore_flyimport)]
    fn account_to_modify(&self) -> AccountInfo;
    #[inline]
    fn add_lamports(&self, lamports: u64) -> Result<()> {
        *self.account_to_modify().try_borrow_mut_lamports()? += lamports;
        Ok(())
    }
}
/// Indicates that this account can fund rent on another account, and potentially be used to create an account.
pub trait CanFundRent: CanAddLamports {
    /// Whether [`Self::account_to_modify`](`CanAddLamports::account_to_modify`) can be used as the funder for a [`crate::program::system::CreateAccount`] CPI.
    #[rust_analyzer::completions(ignore_flyimport)]
    fn can_create_account(&self) -> bool;
    /// Increases the rent of the recipient by `lamports`.
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        ctx: &Context,
    ) -> Result<()>;

    #[rust_analyzer::completions(ignore_flyimport)]
    #[inline]
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

pub trait CanCloseAccount {
    /// Closes the account by zeroing the lamports and replacing the discriminant with all `u8::MAX`,
    /// reallocating down to size.
    fn close_account(&self, recipient: &(impl CanAddLamports + ?Sized)) -> Result<()>
    where
        Self: HasOwnerProgram,
        Self: Sized;

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    ///
    /// It also happens to be unsound because [`AccountInfo::assign`] is unsound.
    fn close_account_full(&self, recipient: &dyn CanAddLamports) -> Result<()>;
}

pub trait CanModifyRent {
    /// Normalizes the rent of an account if data size is changed.
    /// Assumes `Self` is mutable and owned by this program.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn normalize_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()>;

    /// Refunds rent to the funder so long as the account has more than the minimum rent.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn refund_rent(&self, recipient: &(impl CanAddLamports + ?Sized), ctx: &Context) -> Result<()>;

    /// Receive rent to self to be at least the minimum rent. This will not normalize down excess lamports.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn receive_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()>;

    /// Emits a warning message if the account has more lamports than required by rent.
    #[rust_analyzer::completions(ignore_flyimport)]
    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    fn check_cleanup(&self, ctx: &Context) -> Result<()>;
}

pub trait CanSystemCreateAccount {
    /// Creates an account using the system program
    /// Assumes `Self` is owned by the System program and funder is a System account
    #[rust_analyzer::completions(ignore_flyimport)]
    fn system_create_account(
        &self,
        funder: &(impl CanFundRent + ?Sized),
        owner: Pubkey,
        space: usize,
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<()>;
}

#[doc(hidden)]
pub(crate) mod internal_reverse {
    use super::*;

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn _account_set_validate_reverse<T, A>(
        validate_input: A,
        this: &mut T,
        ctx: &mut Context,
    ) -> Result<()>
    where
        T: AccountSetValidate<A>,
    {
        this.validate_accounts(validate_input, ctx)
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn _account_set_cleanup_reverse<T, A>(
        cleanup_input: A,
        this: &mut T,
        ctx: &mut Context,
    ) -> Result<()>
    where
        T: AccountSetCleanup<A>,
    {
        this.cleanup_accounts(cleanup_input, ctx)
    }
}

pub(crate) mod prelude {
    use super::*;
    pub use super::{
        AccountSet, CanCloseAccount as _, CanModifyRent as _, CheckKey as _, ProgramAccount,
        TryFromAccounts, TryFromAccountsWithArgs,
    };
    pub use account::{
        discriminant, Account, CloseAccount, NormalizeRent, ReceiveRent, RefundRent,
    };
    pub use borsh_account::BorshAccount;
    pub use modifiers::{
        init::{Create, CreateIfNeeded, Init},
        mutable::Mut,
        seeded::{GetSeeds, Seed, Seeded, Seeds, SeedsWithBump},
        signer::Signer,
    };
    pub use program::Program;
    pub use rest::Rest;
    pub use single_set::SingleAccountSet;
    pub use system_account::SystemAccount;
    pub use sysvar::Sysvar;
    pub use validated_account::{AccountValidate, ValidatedAccount};
}

#[cfg(test)]
mod test {
    use super::{CpiAccountSet, CpiConstWrapper};
    use crate::{
        account_set::AccountSetValidate,
        prelude::{Context, PinocchioAccountMeta, ProgramError, Pubkey},
    };
    use star_frame_proc::AccountSet;
    use std::mem::MaybeUninit;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>, extra_validation = { arg.push(N); Ok(()) })]
    struct InnerAccount<const N: usize>;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet123 {
        #[validate(arg = &mut *arg)]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet213 {
        #[validate(arg = &mut *arg, requires = [b])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet312 {
        #[validate(arg = &mut *arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg, requires = [c])]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet231 {
        #[validate(arg = &mut *arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[test]
    fn cpi_const_wrapper_to_cpi_accounts_delegates_to_inner() {
        let inner_none = None::<()>;
        let wrapper_none = CpiConstWrapper::<Option<()>, 0>(inner_none);
        assert_eq!(wrapper_none.to_cpi_accounts(), inner_none.to_cpi_accounts());

        let inner_some = Some(());
        let wrapper_some = CpiConstWrapper::<Option<()>, 1>(inner_some);
        assert_eq!(wrapper_some.to_cpi_accounts(), inner_some.to_cpi_accounts());
    }

    #[test]
    fn cpi_const_wrapper_write_account_infos_delegates_and_propagates_errors() {
        let inner = None::<()>;
        let wrapper = CpiConstWrapper::<Option<()>, 2>(inner);
        let wrapper_accounts = wrapper.to_cpi_accounts();
        let inner_accounts = inner.to_cpi_accounts();
        let mut wrapper_index = 3;
        let mut inner_index = 3;
        let mut wrapper_infos = [MaybeUninit::uninit(); 4];
        let mut inner_infos = [MaybeUninit::uninit(); 4];

        let wrapper_result = CpiConstWrapper::<Option<()>, 2>::write_account_infos(
            None,
            &wrapper_accounts,
            &mut wrapper_index,
            &mut wrapper_infos,
        );
        let inner_result = <Option<()> as CpiAccountSet>::write_account_infos(
            None,
            &inner_accounts,
            &mut inner_index,
            &mut inner_infos,
        );

        assert!(wrapper_result.is_err());
        assert!(inner_result.is_err());
        assert_eq!(
            ProgramError::from(wrapper_result.unwrap_err()),
            ProgramError::from(inner_result.unwrap_err())
        );
        assert_eq!(wrapper_index, inner_index);
        assert_eq!(wrapper_index, 3);
    }

    #[test]
    fn cpi_const_wrapper_write_account_metas_delegates_and_preserves_index_progression() {
        let program_id = Pubkey::new_unique();

        let inner_none = None::<()>;
        let wrapper_none = CpiConstWrapper::<Option<()>, 3>(inner_none);
        let wrapper_none_accounts = wrapper_none.to_cpi_accounts();
        let inner_none_accounts = inner_none.to_cpi_accounts();
        let mut wrapper_none_index = 5;
        let mut inner_none_index = 5;
        let mut wrapper_none_metas: [MaybeUninit<PinocchioAccountMeta<'_>>; 6] =
            std::array::from_fn(|_| MaybeUninit::uninit());
        let mut inner_none_metas: [MaybeUninit<PinocchioAccountMeta<'_>>; 6] =
            std::array::from_fn(|_| MaybeUninit::uninit());

        CpiConstWrapper::<Option<()>, 3>::write_account_metas(
            &program_id,
            &wrapper_none_accounts,
            &mut wrapper_none_index,
            &mut wrapper_none_metas,
        );
        <Option<()> as CpiAccountSet>::write_account_metas(
            &program_id,
            &inner_none_accounts,
            &mut inner_none_index,
            &mut inner_none_metas,
        );

        assert_eq!(wrapper_none_index, inner_none_index);
        assert_eq!(wrapper_none_index, 6);

        let inner_some = Some(());
        let wrapper_some = CpiConstWrapper::<Option<()>, 4>(inner_some);
        let wrapper_some_accounts = wrapper_some.to_cpi_accounts();
        let inner_some_accounts = inner_some.to_cpi_accounts();
        let mut wrapper_some_index = 7;
        let mut inner_some_index = 7;
        let mut wrapper_some_metas: [MaybeUninit<PinocchioAccountMeta<'_>>; 8] =
            std::array::from_fn(|_| MaybeUninit::uninit());
        let mut inner_some_metas: [MaybeUninit<PinocchioAccountMeta<'_>>; 8] =
            std::array::from_fn(|_| MaybeUninit::uninit());

        CpiConstWrapper::<Option<()>, 4>::write_account_metas(
            &program_id,
            &wrapper_some_accounts,
            &mut wrapper_some_index,
            &mut wrapper_some_metas,
        );
        <Option<()> as CpiAccountSet>::write_account_metas(
            &program_id,
            &inner_some_accounts,
            &mut inner_some_index,
            &mut inner_some_metas,
        );

        assert_eq!(wrapper_some_index, inner_some_index);
        assert_eq!(wrapper_some_index, 7);
    }

    #[test]
    fn cpi_const_wrapper_structural_delegation_guard() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/account_set/mod.rs"
        ));
        let start_anchor = "unsafe impl<T, const N: usize> CpiAccountSet for CpiConstWrapper<T, N>";
        let end_anchor = "pub trait ClientAccountSet {";
        let impl_start = source
            .find(start_anchor)
            .expect("CpiConstWrapper impl start anchor must exist");
        let impl_end = source[impl_start..]
            .find(end_anchor)
            .map(|offset| impl_start + offset)
            .expect("CpiConstWrapper impl end anchor must exist");
        let normalized_impl: String = source[impl_start..impl_end].split_whitespace().collect();

        let unimplemented_token = concat!("unimplemented", "!(");
        assert!(!normalized_impl.contains(unimplemented_token));
    }

    #[test]
    fn test_validate() {
        let mut vec = Vec::new();
        let mut ctx = Context::default();
        let mut set = AccountSet123 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![1, 2, 3]);

        vec.clear();
        let mut set = AccountSet213 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![2, 1, 3]);

        vec.clear();
        let mut set = AccountSet312 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![3, 1, 2]);

        vec.clear();
        let mut set = AccountSet231 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![2, 3, 1]);
    }
}
