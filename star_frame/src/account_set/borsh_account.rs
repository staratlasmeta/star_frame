//! A [`ProgramAccount`] that is serialized and deserialized using [`borsh`].

use borsh::object_length;
use derive_more::Debug;

use crate::{
    account_set::{
        modifiers::{
            CanInitAccount, HasInnerType, HasOwnerProgram, HasSeeds, OwnerProgramDiscriminant,
        },
        AccountSetDecode, CanAddLamports, CanFundRent, CanSystemCreateAccount as _,
    },
    errors::ErrorCode,
    prelude::*,
};

/// A [`ProgramAccount`] that is serialized and deserialized using [`BorshSerialize`] and [`BorshDeserialize`].
///
/// This is much less effecient than using [`Account`] because this is not zero-copy.
/// Access is explicit and fallible through [`Self::inner`] and [`Self::inner_mut`].
///
/// Calls [`ProgramAccount::validate_account_info`] during validation to ensure the owner and discriminant match, and writes back the
/// updated `T` to the account info when the account is writable during `AccountSetCleanup`
#[derive(AccountSet, Debug, Clone)]
#[account_set(skip_default_decode, skip_default_idl)]
#[cfg_attr(feature = "aggressive_inline",
    validate(inline_always, extra_validation = T::validate_account_info(self.info))
)]
#[cfg_attr(not(feature = "aggressive_inline"),
    validate(extra_validation = T::validate_account_info(self.info))
)]
#[cleanup(generics = [], extra_cleanup = {
    self.serialize()?;
    self.check_cleanup(ctx)
})]
#[cleanup(
    id = "normalize_rent",
    generics = [<'a, Funder> where Funder: CanFundRent],
    arg = NormalizeRent<&'a Funder>,
    extra_cleanup = {
        self.serialize()?;
        self.normalize_rent(arg.0, ctx)
    }
)]
#[cleanup(
    id = "normalize_rent_cached",
    arg = NormalizeRent<()>,
    generics = [],
    extra_cleanup = {
        self.serialize()?;
        let funder = ctx.get_funder().ok_or_else(|| error!(ErrorCode::EmptyFunderCache, "Missing `funder` in cache for `NormalizeRent`"))?;
        self.normalize_rent(funder, ctx)
    },
)]
#[cleanup(
    id = "receive_rent",
    generics = [<'a, Funder> where Funder: CanFundRent],
    arg = ReceiveRent<&'a Funder>,
    extra_cleanup = {
        self.serialize()?;
        self.receive_rent(arg.0, ctx)
    }
)]
#[cleanup(
    id = "receive_rent_cached",
    arg = ReceiveRent<()>,
    generics = [],
    extra_cleanup = {
        let funder = ctx.get_funder().ok_or_else(|| error!(ErrorCode::EmptyFunderCache, "Missing `funder` in cache for `ReceiveRent`"))?;
        self.serialize()?;
        self.receive_rent(funder, ctx)
    }
)]
#[cleanup(
    id = "refund_rent",
    generics = [<'a, Recipient> where Recipient: CanAddLamports],
    arg = RefundRent<&'a Recipient>,
    extra_cleanup = {
        self.serialize()?;
        self.refund_rent(arg.0, ctx)
    }
)]
#[cleanup(
    id = "refund_rent_cached",
    arg = RefundRent<()>,
    generics = [],
    extra_cleanup = {
        let recipient = ctx.get_recipient().ok_or_else(|| error!(ErrorCode::EmptyRecipientCache, "Missing `recipient` in cache for `RefundRent`"))?;
        self.serialize()?;
        self.refund_rent(recipient, ctx)
    }
)]
#[cleanup(
    id = "close_account",
    generics = [<'a, Recipient> where Recipient: CanAddLamports],
    arg = CloseAccount<&'a Recipient>,
    extra_cleanup = {
        // We don't serialize here because we are about to close the account!
        self.close_account(arg.0)
    }
)]
#[cleanup(
    id = "close_account_cached",
    arg = CloseAccount<()>,
    generics = [],
    extra_cleanup = {
        // We don't serialize here because we are about to close the account!
        let recipient = ctx.get_recipient().ok_or_else(|| error!(ErrorCode::EmptyRecipientCache, "Missing `recipient` in cache for `CloseAccount`"))?;
        self.close_account(recipient)
    }
)]
pub struct BorshAccount<T: ProgramAccount + BorshSerialize + BorshDeserialize> {
    #[single_account_set(
        skip_has_inner_type,
        skip_can_init_account,
        skip_has_seeds,
        skip_has_owner_program
    )]
    info: AccountInfo,
    #[account_set(skip = )]
    data: Option<T>,
}

impl<'a, T> AccountSetDecode<'a, ()> for BorshAccount<T>
where
    T: BorshDeserialize + BorshSerialize + ProgramAccount + Default,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        ctx: &mut Context,
    ) -> Result<Self> {
        let info = <AccountInfo as AccountSetDecode<'a, ()>>::decode_accounts(accounts, (), ctx)?;
        let data = if info.data_len() > size_of::<OwnerProgramDiscriminant<T>>() {
            Some(T::try_from_slice(
                &info.account_data()?[size_of::<OwnerProgramDiscriminant<T>>()..],
            )?)
        } else {
            None
        };
        Ok(Self { info, data })
    }
}

impl<T: ProgramAccount + BorshSerialize + BorshDeserialize> BorshAccount<T> {
    fn missing_inner_error(pubkey: &Pubkey) -> Error {
        error!(
            ProgramError::InvalidAccountData,
            "Accessing BorshAccount `{}` data before it is initialized", pubkey
        )
    }

    /// Returns an immutable reference to the deserialized account data.
    ///
    /// Returns [`ProgramError::InvalidAccountData`] when data has not been initialized yet.
    pub fn inner(&self) -> Result<&T> {
        self.data
            .as_ref()
            .ok_or_else(|| Self::missing_inner_error(self.info.pubkey()))
    }

    /// Returns a mutable reference to the deserialized account data.
    ///
    /// Returns [`ProgramError::AccountBorrowFailed`] when the account is not writable, and
    /// [`ProgramError::InvalidAccountData`] when data has not been initialized yet.
    pub fn inner_mut(&mut self) -> Result<&mut T> {
        if !self.is_writable() {
            bail!(
                ProgramError::AccountBorrowFailed,
                "Tried to borrow mutably from BorshAccount `{}` which is not writable",
                self.pubkey()
            );
        }
        let pubkey = *self.info.pubkey();
        match self.data.as_mut() {
            Some(data) => Ok(data),
            None => Err(Self::missing_inner_error(&pubkey)),
        }
    }

    /// Serializes the inner data `T` back to the account info if the account is writable, still owned by this program, and not closed.
    ///
    /// This is called during `AccountSetCleanup` and can be useful to call manually if you need the data to be serialized prior to a CPI.
    pub fn serialize(&mut self) -> Result<()> {
        if self.is_writable()
            && self.info.data_len() > size_of::<OwnerProgramDiscriminant<T>>()
            && self.owner_pubkey() == T::OwnerProgram::ID
        {
            let new_size = size_of::<OwnerProgramDiscriminant<T>>() + object_length(&self.data)?;
            self.info.resize(new_size)?;
            let mut account_data = self.info.account_data_mut()?;
            self.data
                .serialize(&mut &mut account_data[size_of::<OwnerProgramDiscriminant<T>>()..])?;
        }
        Ok(())
    }

    /// Reloads the account data from the account info.
    ///
    /// This is useful if the account data has been modified by another program through a CPI, which won't update
    /// `Self`'s deserialized data.
    pub fn reload(&mut self) -> Result<()> {
        self.data = Some(T::try_from_slice(
            &self.info.account_data()?[size_of::<OwnerProgramDiscriminant<T>>()..],
        )?);
        Ok(())
    }

    /// Sets the inner data `T`.
    ///
    /// To mutate existing data, use [`Self::inner_mut`]. This method replaces the entire
    /// deserialized value.
    ///
    /// Returns an error if the account is not writable.
    pub fn set_inner(&mut self, data: T) -> Result<()> {
        ensure!(
            self.is_writable(),
            ErrorCode::ExpectedWritable,
            "BorshAccount {} is not writable",
            self.pubkey()
        );
        self.data = Some(data);
        Ok(())
    }
}

impl<T> HasSeeds for BorshAccount<T>
where
    T: ProgramAccount + BorshDeserialize + BorshSerialize + HasSeeds,
{
    type Seeds = T::Seeds;
}

impl<T> HasOwnerProgram for BorshAccount<T>
where
    T: ProgramAccount + BorshDeserialize + BorshSerialize,
{
    type OwnerProgram = T::OwnerProgram;
}

impl<T> HasInnerType for BorshAccount<T>
where
    T: ProgramAccount + BorshDeserialize + BorshSerialize + 'static,
{
    type Inner = T;
}

impl<T> CanInitAccount<()> for BorshAccount<T>
where
    T: BorshDeserialize + BorshSerialize + ProgramAccount + Default,
{
    #[inline]
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        _arg: (),
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool> {
        self.init_account::<IF_NEEDED>(|| Default::default(), account_seeds, ctx)
    }
}

impl<T, InitFn> CanInitAccount<InitFn> for BorshAccount<T>
where
    InitFn: FnOnce() -> T,
    T: BorshDeserialize + BorshSerialize + ProgramAccount,
{
    #[inline]
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: InitFn,
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool> {
        let funder = ctx.get_funder().ok_or_else(|| {
            error!(
                ErrorCode::EmptyFunderCache,
                "Missing tagged `funder` for Account `init_account`"
            )
        })?;
        self.init_account::<IF_NEEDED>((arg, funder), account_seeds, ctx)
    }
}

impl<T, Funder> CanInitAccount<(&Funder,)> for BorshAccount<T>
where
    T: BorshDeserialize + BorshSerialize + ProgramAccount + Default,
    Funder: CanFundRent + ?Sized,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (&Funder,),
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool> {
        self.init_account::<IF_NEEDED>((|| Default::default(), arg.0), account_seeds, ctx)
    }
}

impl<T, Funder, InitValue> CanInitAccount<(InitValue, &Funder)> for BorshAccount<T>
where
    InitValue: FnOnce() -> T,
    T: BorshDeserialize + BorshSerialize + ProgramAccount,
    Funder: CanFundRent + ?Sized,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitValue, &Funder),
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool> {
        if IF_NEEDED {
            let needs_init = self.account_info().owner().fast_eq(&System::ID)
                || self.account_data()?[..size_of::<OwnerProgramDiscriminant<T>>()]
                    .iter()
                    .all(|x| *x == 0);
            if !needs_init {
                return Ok(false);
            }
        }
        self.check_writable()?;
        let (init_value, funder) = arg;
        let data = init_value();
        let space = size_of::<OwnerProgramDiscriminant<T>>() + object_length(&data)?;
        self.system_create_account(funder, T::OwnerProgram::ID, space, account_seeds, ctx)
            .ctx("system_create_account failed")?;
        self.account_data_mut()?[..size_of::<OwnerProgramDiscriminant<T>>()]
            .copy_from_slice(bytemuck::bytes_of(&T::DISCRIMINANT));
        // TODO: Should we serialize this now, or wait until cleanup?
        self.data = Some(data);
        Ok(true)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<T, A> AccountSetToIdl<A> for BorshAccount<T>
    where
        AccountInfo: AccountSetToIdl<A>,
        T: BorshDeserialize + BorshSerialize + ProgramAccount + AccountToIdl,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let mut set = <AccountInfo>::account_set_to_idl(idl_definition, arg)?;
            set.single()?
                .program_accounts
                .push(T::account_to_idl(idl_definition)?);
            Ok(set)
        }
    }
}
