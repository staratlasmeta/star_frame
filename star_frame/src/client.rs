//! Client-side utilities for working with Star Frame programs.
//!
//! This module provides convenient traits and functions for client applications to interact
//! with Star Frame programs. It includes utilities for creating Solana instructions,
//! working with program-derived addresses (PDAs), and serializing/deserializing data types
//! and program accounts.

use crate::{
    account_set::{
        account::discriminant::AccountDiscriminant,
        modifiers::{HasOwnerProgram, HasSeeds},
        ClientAccountSet,
    },
    instruction::InstructionDiscriminant,
    prelude::*,
    unsize::{init::UnsizedInit, FromOwned},
};

use borsh::{object_length, BorshSerialize};
use bytemuck::bytes_of;
use solana_instruction::Instruction as SolanaInstruction;

#[doc(hidden)]
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
        <Self as UnsizedInit<I>>::init(data, init_arg)?;
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
