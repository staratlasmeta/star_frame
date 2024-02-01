use crate::instruction::InstructionSet;
use crate::program::{ProgramIds, StarFrameProgram};
use crate::util::Network;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
pub fn try_star_frame_entrypoint<T: StarFrameProgram>(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    network: Network,
) -> Result<()> {
    let len = size_of::<T::InstructionDiscriminant>();
    let disc_bytes = instruction_data.get(0..len).ok_or_else(|| {
        msg!("Instruction data too short");
        ProgramError::InvalidInstructionData
    })?;

    let mut syscalls = crate::sys_calls::solana_runtime::SolanaRuntime {
        program_id,
        network,
    };

    let ix_set: T::InstructionSet<'_> = todo!();

    // todo: actually deserialize the instruction set
    ix_set.handle_ix(program_id, accounts, &mut syscalls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use star_frame_proc::{program, pubkey};
    // struct Stuff;
    impl StarFrameProgram for Stuff {
        type InstructionSet<'a> = ();
        type InstructionDiscriminant = ();
        type AccountDiscriminant = ();
        const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
        const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
            (
                Network::Mainnet,
                &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
            ),
            (
                Network::Custom("atlasnet"),
                &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
            ),
        ]);
    }

    use star_frame::util::Network;

    #[program(Network::Mainnet)]
    pub struct Stuff;
}
