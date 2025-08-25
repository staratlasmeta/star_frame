//! Cross program invocation (CPI) builders and utilities.
use crate::{
    account_set::CpiAccountSet, client::star_frame_instruction_data,
    instruction::InstructionDiscriminant, prelude::*, SolanaInstruction,
};
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
    /// Creates a `CpiBuilder` with a `StarFrameInstruction`.
    ///
    /// # Example
    /// ```ignore
    /// MyProgram::cpi(&MyInstruction { .. }, MyInstructionCpiAccounts { .. }, &ctx)?.invoke()?;
    /// ```
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
    /// Manually creates a `CpiBuilder` with a program id override. Prefer [`MakeCpi::cpi`] for a more convenient interface.
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
