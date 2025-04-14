use crate::account_set::{
    discriminant::AccountDiscriminant, GetSeeds, HasOwnerProgram, HasSeeds, ProgramAccount,
};
use crate::instruction::{InstructionDiscriminant, InstructionSet, StarFrameInstruction};
use crate::prelude::UnsizedInit;
use crate::program::StarFrameProgram;
use crate::syscalls::SyscallInvoke;
use crate::unsize::{FromOwned, UnsizedType};
use crate::Result;
use crate::SolanaInstruction;
use borsh::{object_length, BorshSerialize};
use bytemuck::bytes_of;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

pub trait CpiAccountSet<'info> {
    type CpiAccounts: Debug + Clone;
    /// The minimum number of accounts this CPI might use
    const MIN_LEN: usize;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts;
    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo<'info>>);
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    );
}

pub trait ClientAccountSet {
    type ClientAccounts: Debug + Clone;
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
#[derive(Debug, Clone)]
pub struct CpiBuilder<'info> {
    pub instruction: SolanaInstruction,
    pub accounts: Vec<AccountInfo<'info>>,
}

impl<'info> CpiBuilder<'info> {
    #[inline]
    pub fn invoke(&self, syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        syscalls.invoke(&self.instruction, &self.accounts)
    }

    #[inline]
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        syscalls.invoke_signed(&self.instruction, &self.accounts, signer_seeds)
    }
}

pub trait MakeCpi<'info>: StarFrameProgram {
    fn cpi<I, A>(data: &I, accounts: A::CpiAccounts) -> Result<CpiBuilder<'info>>
    where
        I: StarFrameInstruction<Accounts<'static, 'static, 'info> = A>
            + InstructionDiscriminant<Self::InstructionSet>
            + BorshSerialize,
        A: CpiAccountSet<'info>,
    {
        CpiBuilder::new::<Self::InstructionSet, I, A>(Self::ID, data, accounts)
    }
}

impl<T> MakeCpi<'_> for T where T: StarFrameProgram + ?Sized {}

impl<'info> CpiBuilder<'info> {
    pub fn new<S, I, A>(program_id: Pubkey, data: &I, accounts: A::CpiAccounts) -> Result<Self>
    where
        S: InstructionSet,
        I: StarFrameInstruction<Accounts<'static, 'static, 'info> = A>
            + InstructionDiscriminant<S>
            + BorshSerialize,
        A: CpiAccountSet<'info>,
    {
        let mut metas = Vec::with_capacity(A::MIN_LEN);
        A::extend_account_metas(&program_id, &accounts, &mut metas);
        let mut infos = Vec::with_capacity(A::MIN_LEN);
        A::extend_account_infos(accounts, &mut infos);
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

pub trait MakeInstruction<'info>: StarFrameProgram {
    fn instruction<I, A>(data: &I, accounts: A::ClientAccounts) -> Result<SolanaInstruction>
    where
        I: StarFrameInstruction<Accounts<'static, 'static, 'info> = A>
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

impl<T> MakeInstruction<'_> for T where T: StarFrameProgram + ?Sized {}

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
