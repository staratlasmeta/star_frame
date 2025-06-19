use crate::{Result, SolanaInstruction};
use itertools::Itertools;
use pinocchio::account_info::AccountInfo;
use pinocchio::cpi::slice_invoke_signed;
use pinocchio::instruction::{
    AccountMeta as PinocchioAccountMeta, Instruction as PinocchioInstruction,
    Seed as PinocchioSeed, Signer as PinocchioSigner,
};
use pinocchio::ProgramResult;

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
