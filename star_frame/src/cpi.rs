//! Cross program invocation (CPI) builders and utilities.
use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    account_set::{CpiAccountSet, DynamicCpiAccountSetLen},
    client::star_frame_instruction_data,
    instruction::InstructionDiscriminant,
    prelude::*,
    program::system::{SystemInstructionSet, Transfer, TransferAccounts, TransferCpiAccounts},
    SolanaInstruction,
};
use borsh::object_length;
use bytemuck::bytes_of;
use itertools::Itertools;
use pinocchio::{
    account_info::AccountInfo,
    cpi::slice_invoke_signed,
    instruction::{
        AccountMeta as PinocchioAccountMeta, Instruction as PinocchioInstruction,
        Seed as PinocchioSeed, Signer as PinocchioSigner,
    },
    ProgramResult,
};
use typenum::{False, IsGreater, True};

/// A builder for creating a CPI instruction for a [`StarFrameProgram`].
///
/// Returned from [`MakeCpi::cpi`], and can be invoked with [`CpiBuilder::invoke`] or [`CpiBuilder::invoke_signed`].
#[must_use = "Did you forget to invoke the builder?"]
#[derive(derive_more::Debug, Clone)]
pub struct CpiBuilder {
    pub instruction: SolanaInstruction,
    #[debug("{} accounts", self.accounts.len())]
    pub accounts: Vec<AccountInfo>,
}

impl CpiBuilder {
    /// Invokes the CPI with no PDA signers.
    #[inline]
    pub fn invoke(&self) -> Result<()> {
        crate::cpi::invoke(&self.instruction, &self.accounts)
    }

    /// Invokes the CPI with seeds for PDA signers.
    #[inline]
    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        crate::cpi::invoke_signed(&self.instruction, &self.accounts, signer_seeds)
    }
}

/// Used to create a `CpiBuilder` for a [`StarFrameProgram`].
pub trait MakeCpi: StarFrameProgram {
    // /// Creates a `CpiBuilder` with a `StarFrameInstruction`.
    // ///
    // /// # Example
    // /// ```ignore
    // /// MyProgram::cpi(&MyInstruction { .. }, MyInstructionCpiAccounts { .. }, &ctx)?.invoke()?;
    // /// ```
    // fn cpi<I, A>(data: &I, accounts: A::CpiAccounts, ctx: &Context) -> Result<CpiBuilder>
    // where
    //     I: StarFrameInstruction<Accounts<'static, 'static> = A>
    //         + InstructionDiscriminant<Self::InstructionSet>
    //         + BorshSerialize,
    //     A: CpiAccountSet,
    // {
    //     // CpiBuilder::new::<Self::InstructionSet, I, A>(Self::ID, data, accounts, ctx)
    // }
}

impl<T> MakeCpi for T where T: StarFrameProgram + ?Sized {}

#[inline(never)]
fn invoke_signed_never_inline(
    instruction: &PinocchioInstruction,
    accounts: &[&AccountInfo],
    signers: &[PinocchioSigner],
) -> ProgramResult {
    slice_invoke_signed(instruction, accounts, signers)
}

#[inline]
fn convert_account_metas(instruction: &SolanaInstruction) -> Vec<PinocchioAccountMeta<'_>> {
    instruction
        .accounts
        .iter()
        .map(|meta| PinocchioAccountMeta {
            pubkey: meta.pubkey.as_array(),
            is_writable: meta.is_writable,
            is_signer: meta.is_signer,
        })
        .collect_vec()
}

#[inline]
fn convert_instruction<'a>(
    instruction: &'a SolanaInstruction,
    metas: &'a [PinocchioAccountMeta<'a>],
) -> PinocchioInstruction<'a, 'a, 'a, 'a> {
    PinocchioInstruction {
        program_id: instruction.program_id.as_array(),
        data: instruction.data.as_slice(),
        accounts: metas,
    }
}

pub fn invoke(instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> Result<()> {
    invoke_signed(instruction, accounts, &[])
}

pub fn invoke_signed(
    instruction: &SolanaInstruction,
    accounts: &[AccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> Result<()> {
    let metas = convert_account_metas(instruction);
    let pinocchio_ix = convert_instruction(instruction, &metas);
    let accounts = accounts.iter().collect_vec();

    let nested_seeds: Vec<Vec<PinocchioSeed>> = signers_seeds
        .iter()
        .map(|seeds| {
            seeds
                .iter()
                .map(|seed| PinocchioSeed::from(*seed))
                .collect_vec()
        })
        .collect_vec();
    let signers = nested_seeds
        .iter()
        .map(|seeds| seeds.as_slice().into())
        .collect_vec();

    // TODO: Make this log the errors better
    invoke_signed_never_inline(&pinocchio_ix, &accounts, &signers)?;
    Ok(())
}

#[must_use = "Did you forget to invoke the builder?"]
#[derive(derive_more::Debug)]
pub struct CpiBuilder2<'program_id, 'args, 'accounts, P, Ix, Accounts> {
    pub program_id: &'program_id Pubkey,
    pub data: &'args Ix,
    pub accounts: &'accounts Accounts,
    pub program: PhantomData<P>,
}

#[derive(Debug)]
pub struct ByteLen<const N: usize>([u8; N]);

pub trait ConstLen {
    type Bytes;
}

pub trait Invoke {
    fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> Result<()>;
    fn invoke(&self) -> Result<()> {
        self.invoke_signed(&[])
    }
}

impl<P, Ix, A, Acc> Invoke for CpiBuilder2<'_, '_, '_, P, Ix, Acc>
where
    P: StarFrameProgram,
    Ix: BorshSerialize
        + ConstLen
        + StarFrameInstruction<Accounts<'static, 'static> = A>
        + InstructionDiscriminant<P::InstructionSet>,
    A: CpiAccountSet<CpiAccounts = Acc, AccountLen: HandleCpiArray>,
{
    fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> Result<()> {
        let mut buffer = MaybeUninit::<<Ix as ConstLen>::Bytes>::zeroed();
        let data: &mut [u8] = unsafe {
            &mut *ptr_meta::from_raw_parts_mut(
                buffer.as_mut_ptr().cast::<()>(),
                size_of::<<Ix as ConstLen>::Bytes>(),
            )
        };
        let (discriminator, mut ix_data) = data.split_at_mut(size_of::<
            <<P as StarFrameProgram>::InstructionSet as InstructionSet>::Discriminant,
        >());
        discriminator.copy_from_slice(bytes_of(
            &<Ix as InstructionDiscriminant<P::InstructionSet>>::DISCRIMINANT,
        ));
        self.data.serialize(&mut ix_data)?;

        let mut infos_index = 0;
        let mut infos_arr = <<A as CpiAccountSet>::AccountLen as HandleCpiArray>::uninit_infos();
        A::write_account_infos(None, self.accounts, &mut infos_index, infos_arr.as_mut())?;

        let mut metas_index = 0;
        let mut metas_arr = <<A as CpiAccountSet>::AccountLen as HandleCpiArray>::uninit_metas();
        A::write_account_metas(
            self.program_id,
            self.accounts,
            &mut metas_index,
            metas_arr.as_mut(),
        );

        let nested_seeds: Vec<Vec<PinocchioSeed>> = signers_seeds
            .iter()
            .map(|seeds| {
                seeds
                    .iter()
                    .map(|seed| PinocchioSeed::from(*seed))
                    .collect_vec()
            })
            .collect_vec();
        let signers = nested_seeds
            .iter()
            .map(|seeds| seeds.as_slice().into())
            .collect_vec();

        <A as CpiAccountSet>::AccountLen::invoke_signed(
            self.program_id,
            data,
            infos_arr,
            infos_index,
            metas_arr,
            metas_index,
            &signers,
        )?;

        Ok(())
    }
}

trait HandleCpiArray {
    type Arr<T>: AsMut<[MaybeUninit<T>]>;
    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo>;
    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>>;
    fn invoke_signed<'a>(
        program_id: &Pubkey,
        data: &[u8],
        infos: Self::Arr<&'a AccountInfo>,
        infos_index: usize,
        metas: Self::Arr<PinocchioAccountMeta<'a>>,
        metas_index: usize,
        signers: &[PinocchioSigner],
    ) -> Result<()>;
}

macro_rules! impl_handle_cpi_array {
    ($($n:tt)*) => {
        $(
            paste::paste! {
                impl HandleCpiArray for typenum::[<U $n>] {
                    type Arr<T> = [MaybeUninit<T>; $n];
                    #[inline]
                    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo> {
                        unsafe { MaybeUninit::uninit().assume_init() }
                    }
                    #[inline]
                    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>> {
                        unsafe { MaybeUninit::uninit().assume_init() }
                    }
                    #[inline(always)]
                    fn invoke_signed<'a>(
                        program_id: &Pubkey,
                        data: &[u8],
                        infos: Self::Arr<&'a AccountInfo>,
                        infos_index: usize,
                        metas: Self::Arr<PinocchioAccountMeta<'a>>,
                        metas_index: usize,
                        signers: &[PinocchioSigner],
                    ) -> Result<()> {
                        assert_eq!(infos_index, infos.len());
                        assert_eq!(metas_index, metas.len());
                        // SAFETY:
                        // Tranmuting an array of uninits to an init array is safe
                        let metas = unsafe {
                            core::mem::transmute::<
                                [MaybeUninit<PinocchioAccountMeta>; $n],
                                MaybeUninit<[PinocchioAccountMeta; $n]>,
                            >(metas)
                        };

                        // SAFETY:
                        // Tranmuting an array of uninits to an uninit array is safe
                        let infos = unsafe {
                            core::mem::transmute::<
                                [MaybeUninit<&AccountInfo>; $n],
                                MaybeUninit<[&AccountInfo; $n]>,
                            >(infos)
                        };
                        pinocchio::cpi::invoke_signed(
                            &PinocchioInstruction {
                                program_id: program_id.as_array(),
                                data,
                                accounts:
                                // SAFETY:
                                // CpiAccountSet's safety requirements ensure this has been initialized
                                &unsafe { metas.assume_init() }
                            },
                            &unsafe { infos.assume_init() },
                            signers,
                        )?;
                        Ok(())
                    }
                }
            }
        )*
    };
}

impl_handle_cpi_array!(
    0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

impl HandleCpiArray for DynamicCpiAccountSetLen {
    type Arr<T> = [MaybeUninit<T>; 64];

    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo> {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>> {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    fn invoke_signed<'a>(
        program_id: &Pubkey,
        data: &[u8],
        infos: Self::Arr<&'a AccountInfo>,
        infos_index: usize,
        metas: Self::Arr<PinocchioAccountMeta<'a>>,
        metas_index: usize,
        signers: &[PinocchioSigner],
    ) -> Result<()> {
        assert_eq!(infos_index, metas_index);

        let metas_slice = &metas[..metas_index];
        // SAFETY:
        // We can turn a slice of uninits to a slice of inits (we can assume up to the index is initialized)
        let metas_slice = unsafe {
            &*(std::ptr::from_ref::<[MaybeUninit<PinocchioAccountMeta<'a>>]>(metas_slice)
                as *const [PinocchioAccountMeta<'a>])
        };

        let infos_slice = &infos[..infos_index];
        // SAFETY:
        // We can turn a slice of uninits to a slice of inits (we can assume up to the index is initialized)
        let infos_slice = unsafe {
            &*std::ptr::from_ref::<[MaybeUninit<&AccountInfo>]>(infos_slice)
                .cast::<[&AccountInfo; 64]>()
        };

        pinocchio::cpi::slice_invoke_signed(
            &PinocchioInstruction {
                program_id: program_id.as_array(),
                data,
                accounts: metas_slice,
            },
            infos_slice,
            signers,
        )?;
        Ok(())
    }
}
