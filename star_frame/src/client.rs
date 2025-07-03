use crate::account_set::{
    discriminant::AccountDiscriminant, GetSeeds, HasOwnerProgram, HasSeeds, ProgramAccount,
};

use crate::instruction::{InstructionDiscriminant, InstructionSet, StarFrameInstruction};
use crate::prelude::{Context, UnsizedInit};
use crate::program::StarFrameProgram;
use crate::unsize::{FromOwned, UnsizedType};
use crate::Result;
use borsh::{object_length, BorshSerialize};
use bytemuck::bytes_of;
use pinocchio::account_info::AccountInfo;
use solana_instruction::{AccountMeta, Instruction as SolanaInstruction};
use solana_pubkey::Pubkey;
use std::fmt::Debug;

pub trait CpiAccountSet {
    type CpiAccounts: Clone + Debug;
    /// The minimum number of accounts this CPI might use
    const MIN_LEN: usize;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts;
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> Result<()>;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    );
}

pub trait ClientAccountSet {
    type ClientAccounts: Clone + Debug;
    /// The minimum number of accounts this CPI might use
    const MIN_LEN: usize;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    );
}

pub fn star_frame_instruction_data<S, I>(data: &I) -> Result<Vec<u8>>
where
    S: InstructionSet,
    I: InstructionDiscriminant<S> + BorshSerialize,
{
    let data_len = std::mem::size_of::<S::Discriminant>() + object_length(data)?;
    let mut ix_data = Vec::with_capacity(data_len);
    ix_data.extend_from_slice(bytes_of(&I::DISCRIMINANT));
    BorshSerialize::serialize(data, &mut ix_data)?;
    Ok(ix_data)
}

#[must_use = "Did you forget to invoke the builder?"]
#[derive(derive_more::Debug, Clone)]
pub struct CpiBuilder {
    pub instruction: SolanaInstruction,
    #[debug("{} accounts", self.accounts.len())]
    pub accounts: Vec<AccountInfo>,
}

impl CpiBuilder {
    #[inline]
    pub fn invoke(&self) -> Result<()> {
        crate::cpi::invoke(&self.instruction, &self.accounts)
    }

    #[inline]
    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        crate::cpi::invoke_signed(&self.instruction, &self.accounts, signer_seeds)
    }
}

pub trait MakeCpi: StarFrameProgram {
    fn cpi<I, A>(data: &I, accounts: A::CpiAccounts, ctx: &Context) -> Result<CpiBuilder>
    where
        I: StarFrameInstruction<Accounts<'static, 'static> = A>
            + InstructionDiscriminant<Self::InstructionSet>
            + BorshSerialize,
        A: CpiAccountSet,
    {
        CpiBuilder::new::<Self::InstructionSet, I, A>(Self::ID, data, accounts, ctx)
    }
}

impl<T> MakeCpi for T where T: StarFrameProgram + ?Sized {}

impl CpiBuilder {
    pub fn new<S, I, A>(
        program_id: Pubkey,
        data: &I,
        accounts: A::CpiAccounts,
        ctx: &Context,
    ) -> Result<Self>
    where
        S: InstructionSet,
        I: StarFrameInstruction<Accounts<'static, 'static> = A>
            + InstructionDiscriminant<S>
            + BorshSerialize,
        A: CpiAccountSet,
    {
        let mut metas = Vec::with_capacity(A::MIN_LEN);
        A::extend_account_metas(&program_id, &accounts, &mut metas);
        let mut infos = Vec::with_capacity(A::MIN_LEN);
        A::extend_account_infos(&program_id, accounts, &mut infos, ctx)?;
        let data = star_frame_instruction_data::<S, I>(data)?;
        Ok(Self {
            instruction: SolanaInstruction {
                program_id,
                accounts: metas,
                data,
            },
            accounts: infos,
        })
    }
}

pub trait MakeInstruction: StarFrameProgram {
    fn instruction<I, A>(data: &I, accounts: A::ClientAccounts) -> Result<SolanaInstruction>
    where
        I: StarFrameInstruction<Accounts<'static, 'static> = A>
            + InstructionDiscriminant<Self::InstructionSet>
            + BorshSerialize,
        A: ClientAccountSet,
    {
        let mut metas = Vec::with_capacity(A::MIN_LEN);
        A::extend_account_metas(&Self::ID, &accounts, &mut metas);
        let data = star_frame_instruction_data::<Self::InstructionSet, I>(data)?;
        Ok(SolanaInstruction {
            program_id: Self::ID,
            accounts: metas,
            data,
        })
    }
}

impl<T> MakeInstruction for T where T: StarFrameProgram + ?Sized {}

pub trait FindProgramAddress: HasSeeds + HasOwnerProgram {
    fn find_program_address(seeds: &Self::Seeds) -> (Pubkey, u8) {
        Pubkey::find_program_address(&seeds.seeds(), &Self::OwnerProgram::ID)
    }

    fn create_program_address(seeds: &Self::Seeds, bump: u8) -> Result<Pubkey> {
        let mut seeds = seeds.seeds();
        let bump = &[bump];
        seeds.push(bump);
        Ok(Pubkey::create_program_address(
            &seeds,
            &Self::OwnerProgram::ID,
        )?)
    }
}

impl<T> FindProgramAddress for T where T: HasSeeds + HasOwnerProgram {}

pub trait DeserializeType: UnsizedType {
    fn deserialize_type(data: &[u8]) -> Result<Self::Owned> {
        Self::owned(data)
    }
}

impl<T: UnsizedType + ?Sized> DeserializeType for T {}

pub trait SerializeType: UnsizedType {
    fn serialize_type(owned: Self::Owned) -> Result<Vec<u8>>
    where
        Self: FromOwned,
    {
        let byte_size = Self::byte_size(&owned);
        let mut bytes = vec![0u8; byte_size];
        Self::from_owned(owned, &mut bytes.as_mut_slice())?;
        Ok(bytes)
    }

    fn serialize_type_from_init<I>(init_arg: I) -> Result<Vec<u8>>
    where
        Self: UnsizedInit<I>,
    {
        let mut bytes = vec![0u8; <Self as UnsizedInit<I>>::INIT_BYTES];
        let data = &mut &mut bytes[..];
        unsafe { <Self as UnsizedInit<I>>::init(data, init_arg)? };
        Ok(bytes)
    }
}

impl<T: UnsizedType + ?Sized> SerializeType for T {}

pub trait DeserializeAccount: UnsizedType + ProgramAccount {
    fn deserialize_account(data: &[u8]) -> Result<Self::Owned> {
        <AccountDiscriminant<Self> as DeserializeType>::deserialize_type(data)
    }
}

impl<T: UnsizedType + ProgramAccount + ?Sized> DeserializeAccount for T {}

pub trait SerializeAccount: UnsizedType + ProgramAccount {
    #[inline]
    fn serialize_account(owned: Self::Owned) -> Result<Vec<u8>>
    where
        Self: FromOwned,
    {
        <AccountDiscriminant<Self> as SerializeType>::serialize_type(owned)
    }

    #[inline]
    fn serialize_account_from_init<I>(init_arg: I) -> Result<Vec<u8>>
    where
        AccountDiscriminant<Self>: UnsizedInit<I>,
    {
        <AccountDiscriminant<Self> as SerializeType>::serialize_type_from_init(init_arg)
    }
}

impl<T> SerializeAccount for T where T: UnsizedType + ProgramAccount + ?Sized {}
