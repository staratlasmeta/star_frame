//! Client-side utilities for working with Star Frame programs.
//!
//! This module provides convenient traits and functions for client applications to interact
//! with Star Frame programs. It includes utilities for creating Solana instructions,
//! working with program-derived addresses (PDAs), and serializing/deserializing data types
//! and program accounts.

use crate::{
    account_set::{
        account::discriminant::AccountDiscriminant,
        modifiers::{HasOwnerProgram, HasSeeds, OwnerProgramDiscriminant},
    },
    instruction::InstructionDiscriminant,
    prelude::*,
    unsize::{init::UnsizedInit, FromOwned},
    ErrorCode,
};

use borsh::{object_length, BorshSerialize};
use bytemuck::bytes_of;

#[doc(hidden)]
pub fn star_frame_instruction_data<S, I>(data: &I) -> Result<Vec<u8>>
where
    S: InstructionSet,
    I: InstructionDiscriminant<S> + BorshSerialize,
{
    let data_len = core::mem::size_of::<S::Discriminant>() + object_length(data)?;
    let mut ix_data = Vec::with_capacity(data_len);
    ix_data.extend_from_slice(bytes_of(&I::DISCRIMINANT));
    BorshSerialize::serialize(data, &mut ix_data)?;
    Ok(ix_data)
}

#[cfg(not(target_os = "solana"))]
mod instruction_builder {
    use super::*;
    use crate::account_set::ClientAccountSet;
    use solana_instruction::Instruction as SolanaInstruction;
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
}

#[cfg(not(target_os = "solana"))]
pub use instruction_builder::MakeInstruction;

pub trait FindProgramAddress: HasSeeds + HasOwnerProgram {
    fn find_program_address(seeds: &Self::Seeds) -> (Address, u8) {
        Address::find_program_address(&seeds.seeds(), &Self::OwnerProgram::ID)
    }

    fn create_program_address(seeds: &Self::Seeds, bump: u8) -> Result<Address> {
        let mut seeds = seeds.seeds();
        let bump = &[bump];
        seeds.push(bump);
        Ok(Address::create_program_address(
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

impl<T> SerializeType for T where T: UnsizedType + ?Sized {}

#[inline]
fn check_discriminant<T: ProgramAccount + ?Sized>(data: &[u8]) -> Result<()> {
    let discriminant_bytes = data.get(0..size_of_val(&T::DISCRIMINANT)).ok_or_else(|| {
        error!(
            ErrorCode::DiscriminantMismatch,
            "Not enough bytes for the discriminant"
        )
    })?;
    let expected_discriminant = &T::DISCRIMINANT;
    ensure_eq!(
        discriminant_bytes,
        bytes_of(expected_discriminant),
        ErrorCode::DiscriminantMismatch,
    );
    Ok(())
}

pub trait DeserializeAccount: UnsizedType + ProgramAccount {
    fn deserialize_account(data: &[u8]) -> Result<Self::Owned> {
        check_discriminant::<Self>(data)
            .ctx("Failed to validate the discriminant in DeserializeAccount")?;
        <AccountDiscriminant<Self> as DeserializeType>::deserialize_type(data)
    }
}

impl<T> DeserializeAccount for T where T: UnsizedType + ProgramAccount + ?Sized {}

pub trait DeserializeBorshAccount: BorshDeserialize + ProgramAccount {
    fn deserialize_account(data: &[u8]) -> Result<Self> {
        check_discriminant::<Self>(data)
            .ctx("Failed to validate the discriminant in DeserializeBorshAccount")?;
        let data = &data[size_of::<OwnerProgramDiscriminant<Self>>()..];
        BorshDeserialize::try_from_slice(data).map_err(Into::into)
    }
}

impl<T> DeserializeBorshAccount for T where T: BorshDeserialize + ProgramAccount {}

/// A trait that provides logic for serializing [`ProgramAccount`]s that are [`UnsizedType`]s.
///
/// This matches the deserialization logic for the [`Account`] account set.
pub trait SerializeAccount: UnsizedType + ProgramAccount {
    /// Serializes the [`Account`] data from an owned value using the [`FromOwned`] trait.
    ///
    /// Writes the discriminant to the beginning of the serialized data.
    #[inline]
    fn serialize_account(owned: Self::Owned) -> Result<Vec<u8>>
    where
        Self: FromOwned,
    {
        <AccountDiscriminant<Self> as SerializeType>::serialize_type(owned)
    }

    /// Serializes the [`Account`] data from an initialization argument using the [`UnsizedInit`] trait.
    ///
    /// Writes the discriminant to the beginning of the serialized data.
    #[inline]
    fn serialize_account_from_init<I>(init_arg: I) -> Result<Vec<u8>>
    where
        AccountDiscriminant<Self>: UnsizedInit<I>,
    {
        <AccountDiscriminant<Self> as SerializeType>::serialize_type_from_init(init_arg)
    }
}

impl<T> SerializeAccount for T where T: UnsizedType + ProgramAccount + ?Sized {}

/// A trait that provides logic for serializing [`ProgramAccount`]s that are [`BorshDeserialize`]s.
///
/// This matches the deserialization logic for the [`BorshAccount`] account set.
pub trait SerializeBorshAccount: BorshSerialize + ProgramAccount {
    /// Serializes the [`BorshAccount`] data from a reference using the [`BorshSerialize`] trait.
    ///
    /// Writes the discriminant to the beginning of the serialized data.
    fn serialize_account(data: &Self) -> Result<Vec<u8>> {
        let mut bytes =
            vec![0u8; size_of::<OwnerProgramDiscriminant<Self>>() + object_length(data)?];
        let (discriminant_bytes, mut data_bytes) =
            bytes.split_at_mut(size_of::<OwnerProgramDiscriminant<Self>>());
        discriminant_bytes.copy_from_slice(bytes_of(&Self::DISCRIMINANT));
        BorshSerialize::serialize(data, &mut data_bytes)?;
        Ok(bytes)
    }
}

impl<T> SerializeBorshAccount for T where T: BorshSerialize + ProgramAccount + ?Sized {}
